use std::sync::Arc;

use cookie::time::Duration;
use redis::{aio::Connection, AsyncCommands, FromRedisValue, RedisResult, Value};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{api_error::ApiError, database::ModelUser};

use super::{RedisKey, HASH_FIELD};

impl FromRedisValue for RedisSession {
    fn from_redis_value(v: &Value) -> RedisResult<Self> {
        super::string_to_struct::<Self>(v)
    }
}

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
        redis: &Arc<Mutex<Connection>>,
        ttl: Duration,
        uuid: Uuid,
    ) -> Result<(), ApiError> {
        let key_uuid = Self::key_uuid(&uuid);
        let session_set_key = Self::key_set(self.registered_user_id);
        let mut redis = redis.lock().await;

        let session = serde_json::to_string(&self)?;
        let ttl = usize::try_from(ttl.whole_seconds()).unwrap_or(60);
        redis.hset(&key_uuid, HASH_FIELD, session).await?;
        redis.sadd(&session_set_key, &key_uuid).await?;
        redis.expire(session_set_key, ttl).await?;
        redis.expire(&key_uuid, ttl).await?;
        Ok(())
    }

    /// Delete session
    pub async fn delete(redis: &Arc<Mutex<Connection>>, uuid: &Uuid) -> Result<(), ApiError> {
        let key_uuid = Self::key_uuid(uuid);
        let mut redis = redis.lock().await;

        if let Some(session) = redis
            .hget::<'_, &str, &str, Option<Self>>(&key_uuid, HASH_FIELD)
            .await?
        {
            let session_set_key = Self::key_set(session.registered_user_id);

            redis.srem(&session_set_key, &key_uuid).await?;

            if redis
                .smembers::<'_, &str, Vec<String>>(&session_set_key)
                .await?
                .is_empty()
            {
                redis.del(&session_set_key).await?;
            }
        }
        redis.del(key_uuid).await?;
        Ok(())
    }

    /// Delete all sessions for a single user - used when setting a user active status to false
    pub async fn delete_all(
        redis: &Arc<Mutex<Connection>>,
        registered_user_id: i64,
    ) -> Result<(), ApiError> {
        let session_set_key = Self::key_set(registered_user_id);
        let mut redis = redis.lock().await;

        for key in redis
            .smembers::<'_, &str, Vec<String>>(&session_set_key)
            .await?
        {
            redis.del(key).await?;
        }
        redis.del(session_set_key).await?;
        Ok(())
    }

    /// Convert a session into a ModelUser object
    pub async fn get(
        redis: &Arc<Mutex<Connection>>,
        postgres: &PgPool,
        uuid: &Uuid,
    ) -> Result<Option<ModelUser>, ApiError> {
        let op_session = redis
            .lock()
            .await
            .hget::<'_, String, &str, Option<Self>>(Self::key_uuid(uuid), HASH_FIELD)
            .await?;
        if let Some(session) = op_session {
            // If, for some reason, user isn't in postgres, delete session before returning None
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
    pub async fn exists(
        redis: &Arc<Mutex<Connection>>,
        uuid: &Uuid,
    ) -> Result<Option<Self>, ApiError> {
        Ok(redis
            .lock()
            .await
            .hget(Self::key_uuid(uuid), HASH_FIELD)
            .await?)
    }
}
