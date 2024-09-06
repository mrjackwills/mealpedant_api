use cookie::time::Duration;
use fred::{
    clients::RedisPool,
    interfaces::{HashesInterface, KeysInterface, SetsInterface},
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{api_error::ApiError, database::ModelUser, hmap, redis_hash_to_struct};

use super::{RedisKey, HASH_FIELD};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RedisSession {
    pub registered_user_id: i64,
    pub email: String,
}

redis_hash_to_struct!(RedisSession);

impl RedisSession {
    fn key_session(uuid: &Uuid) -> String {
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
        let session_key = Self::key_session(&uuid);
        let session_set_key = Self::key_set(self.registered_user_id);
        let session = serde_json::to_string(&self)?;
        let ttl = ttl.whole_seconds();

        redis.hset::<(), _, _>(&session_key, hmap!(session)).await?;
        redis
            .sadd::<(), _, _>(&session_set_key, &session_key)
            .await?;
        // This won't work as expected, should set TTL to the max at all times
        // redis.expire(&key_session_set, ttl).await?;
        Ok(redis.expire(&session_key, ttl).await?)
    }

    /// Delete session
    pub async fn delete(redis: &RedisPool, uuid: &Uuid) -> Result<(), ApiError> {
        let key_session = Self::key_session(uuid);
        if let Some(session) = redis
            .hget::<Option<Self>, &str, &str>(&key_session, HASH_FIELD)
            .await?
        {
            let key_session_set = Self::key_set(session.registered_user_id);
            redis
                .srem::<(), _, _>(&key_session_set, &key_session)
                .await?;

            // Need to test this!
            if redis
                .smembers::<Vec<String>, &str>(&key_session_set)
                .await?
                .is_empty()
            {
                redis.del::<(), _>(&key_session_set).await?;
            }
        }
        Ok(redis.del(&key_session).await?)
    }

    /// Delete all sessions for a single user - used when setting a user active status to false
    pub async fn delete_all(redis: &RedisPool, registered_user_id: i64) -> Result<(), ApiError> {
        let session_set_key = Self::key_set(registered_user_id);
        let all_keys = redis
            .smembers::<Vec<String>, &str>(&session_set_key)
            .await?;
        for key in all_keys {
            redis.del::<(), _>(key).await?;
        }
        Ok(redis.del(session_set_key).await?)
    }

    /// Convert a session into a ModelUser object
    pub async fn get(
        redis: &RedisPool,
        postgres: &PgPool,
        uuid: &Uuid,
    ) -> Result<Option<ModelUser>, ApiError> {
        if let Some(session) = redis
            .hget::<Option<Self>, &str, &str>(&Self::key_session(uuid), HASH_FIELD)
            .await?
        {
            let user = ModelUser::get(postgres, &session.email).await?;
            // If, for some reason, user isn't in postgres, delete session
            if user.is_none() {
                Self::delete(redis, uuid).await?;
            }
            Ok(user)
        } else {
            Ok(None)
        }
    }

    pub async fn exists(redis: &RedisPool, uuid: &Uuid) -> Result<Option<Self>, ApiError> {
        Ok(redis.hget(Self::key_session(uuid), HASH_FIELD).await?)
    }
}
