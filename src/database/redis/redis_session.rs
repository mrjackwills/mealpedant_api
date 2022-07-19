use std::sync::Arc;

use cookie::time::Duration;
use redis::{aio::Connection, AsyncCommands, FromRedisValue, RedisResult, Value};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{api_error::ApiError, database::ModelUser};

use super::RedisKey;

impl FromRedisValue for RedisSession {
    fn from_redis_value(v: &Value) -> RedisResult<Self> {
        super::string_to_struct::<Self>(v)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RedisSession {
    pub registered_user_id: i64,
    pub email: String,
}

impl RedisSession {
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
        let key = RedisKey::Session(&uuid);
        let session = serde_json::to_string(&self)?;
        let session_set_key = RedisKey::SessionSet(self.registered_user_id);
        let ttl = ttl.as_seconds_f32() as usize;
        redis.lock().await.set(key.to_string(), session).await?;
        redis
            .lock()
            .await
            .sadd(session_set_key.to_string(), key.to_string())
            .await?;
        redis
            .lock()
            .await
            .expire(session_set_key.to_string(), ttl)
            .await?;
        redis.lock().await.expire(key.to_string(), ttl).await?;
        Ok(())
    }

    // On any setting change, need to make sure to update session
    // pub async fn update(&self, redis: &Arc<Mutex<Connection>>, uuid: Uuid) -> Result<(), ApiError> {
    //     let key = RedisKey::Session(&uuid);
    //     let session = serde_json::to_string(&self)?;
    //     redis.lock().await.set(key.to_string(), session).await?;
    //     Ok(())
    // }

    /// Delete session
    pub async fn delete(redis: &Arc<Mutex<Connection>>, uuid: &Uuid) -> Result<(), ApiError> {
        let key = RedisKey::Session(uuid);
        let op_session: Option<Self> = redis.lock().await.get(key.to_string()).await?;
        if let Some(session) = op_session {
            let session_set_key = RedisKey::SessionSet(session.registered_user_id);
            redis
                .lock()
                .await
                .srem(session_set_key.to_string(), key.to_string())
                .await?;

            // Need to test this!
            let set_count: Vec<String> = redis
                .lock()
                .await
                .smembers(session_set_key.to_string())
                .await?;

            if set_count.is_empty() {
                redis.lock().await.del(session_set_key.to_string()).await?;
            }
        }
        redis.lock().await.del(key.to_string()).await?;
        Ok(())
    }

    /// Delete all sessions for a single user - used when setting a user active status to false
    pub async fn delete_all(
        redis: &Arc<Mutex<Connection>>,
        registered_user_id: i64,
    ) -> Result<(), ApiError> {
        let session_set_key = RedisKey::SessionSet(registered_user_id);
        let session_set: Vec<String> = redis
            .lock()
            .await
            .smembers(session_set_key.to_string())
            .await?;
        for i in session_set {
            redis.lock().await.del(i).await?;
        }
        redis.lock().await.del(session_set_key.to_string()).await?;
        Ok(())
    }

    /// Convert a session into a ModelUser object
    pub async fn get(
        redis: &Arc<Mutex<Connection>>,
        postgres: &PgPool,
        uuid: &Uuid,
    ) -> Result<Option<ModelUser>, ApiError> {
        let key = RedisKey::Session(uuid);
        let op_session: Option<Self> = redis.lock().await.get(key.to_string()).await?;
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
        let key = RedisKey::Session(uuid);
        let session: Option<Self> = redis.lock().await.get(key.to_string()).await?;
        Ok(session)
    }
}
