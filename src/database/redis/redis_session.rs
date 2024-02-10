use cookie::time::Duration;
use fred::{
    clients::RedisPool,
    interfaces::{KeysInterface, SetsInterface},
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{api_error::ApiError, database::ModelUser};

use super::RedisKey;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RedisSession {
    pub registered_user_id: i64,
    pub email: String,
}

impl RedisSession {
    fn key_uuid(uuid: &Uuid) -> String {
        RedisKey::Session(uuid).to_string()
    }

    fn key_set(registered_user_id: i64) -> String {
        RedisKey::SessionSet(registered_user_id).to_string()
    }

    pub fn new(registered_user_id: i64, email: &str) -> Self {
        Self {
            registered_user_id,
            email: email.to_owned(),
        }
    }

    // Need to create a set, session::set:id, data: uuid?

    // Insert new session & set ttl
    pub async fn insert(
        &self,
        redis: &RedisPool,
        ttl: Duration,
        uuid: Uuid,
    ) -> Result<(), ApiError> {
        let key_uuid = Self::key_uuid(&uuid);
        let session_set_key = Self::key_set(self.registered_user_id);
        let session = serde_json::to_string(&self)?;
        let ttl = ttl.whole_seconds();
        redis
            .set(
                &key_uuid,
                session,
                Some(fred::types::Expiration::EX(ttl)),
                None,
                false,
            )
            .await?;
        Ok(redis.sadd(&session_set_key, &key_uuid).await?)
    }

    /// Delete session
    pub async fn delete(redis: &RedisPool, uuid: &Uuid) -> Result<(), ApiError> {
        let key_uuid = Self::key_uuid(uuid);

        if let Some(session) = redis.get::<Option<String>, &str>(&key_uuid).await? {
            let session = serde_json::from_str::<Self>(&session)?;
            let session_set_key = Self::key_set(session.registered_user_id);

            redis.srem(&session_set_key, &key_uuid).await?;

            if redis
                .smembers::<Vec<String>, &str>(&session_set_key)
                .await?
                .is_empty()
            {
                redis.del(&session_set_key).await?;
            }
        }
        Ok(redis.del(key_uuid).await?)
    }

    /// Delete all sessions for a single user - used when setting a user active status to false
    pub async fn delete_all(redis: &RedisPool, registered_user_id: i64) -> Result<(), ApiError> {
        let session_set_key = Self::key_set(registered_user_id);
        let all_keys = redis
            .smembers::<Vec<String>, &str>(&session_set_key)
            .await?;
        for key in all_keys {
            redis.del(key).await?;
        }
        Ok(redis.del(session_set_key).await?)
    }

    /// Convert a session into a ModelUser object
    pub async fn get(
        redis: &RedisPool,
        postgres: &PgPool,
        uuid: &Uuid,
    ) -> Result<Option<ModelUser>, ApiError> {
        let op_session = redis
            .get::<Option<String>, String>(Self::key_uuid(uuid))
            .await?;
        if let Some(session) = op_session {
            // If, for some reason, user isn't in postgres, delete session before returning None
            let session = serde_json::from_str::<Self>(&session)?;
            let user = ModelUser::get(postgres, &session.email).await?;
            if user.is_none() {
                Self::delete(redis, uuid).await?;
            }
            Ok(user)
        } else {
            Ok(None)
        }
    }
    /// Check session exists in redis
    pub async fn exists(redis: &RedisPool, uuid: &Uuid) -> Result<Option<Self>, ApiError> {
        if let Some(x) = redis
            .get::<Option<String>, String>(Self::key_uuid(uuid))
            .await?
        {
            Ok(Some(serde_json::from_str::<Self>(&x)?))
        } else {
            Ok(None)
        }
    }
}
