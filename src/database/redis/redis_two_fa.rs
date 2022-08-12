use std::sync::Arc;

use redis::{aio::Connection, AsyncCommands, FromRedisValue, RedisResult, Value};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::{api_error::ApiError, database::ModelUser};

use super::RedisKey;

impl FromRedisValue for RedisTwoFASetup {
    fn from_redis_value(v: &Value) -> RedisResult<Self> {
        super::string_to_struct::<Self>(v)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RedisTwoFASetup {
    pub secret: String,
}

impl RedisTwoFASetup {
    pub fn new(secret: &str) -> Self {
        Self {
            secret: secret.to_owned(),
        }
    }

    // Insert new twofa secret & set ttl od 2 minutes
    pub async fn insert(
        &self,
        redis: &Arc<Mutex<Connection>>,
        user: &ModelUser,
    ) -> Result<&Self, ApiError> {
        let key = RedisKey::TwoFASetup(user.registered_user_id);
        let session = serde_json::to_string(&self)?;
        redis.lock().await.set(key.to_string(), session).await?;
        redis.lock().await.expire(key.to_string(), 120).await?;
        Ok(self)
    }

    /// Delete twofa secret
    pub async fn delete(redis: &Arc<Mutex<Connection>>, user: &ModelUser) -> Result<(), ApiError> {
        let key = RedisKey::TwoFASetup(user.registered_user_id);
        redis.lock().await.del(key.to_string()).await?;
        Ok(())
    }

    /// get twofa setup secret
    pub async fn get(
        redis: &Arc<Mutex<Connection>>,
        user: &ModelUser,
    ) -> Result<Option<Self>, ApiError> {
        let key = RedisKey::TwoFASetup(user.registered_user_id);
        let secret: Option<Self> = redis.lock().await.get(key.to_string()).await?;
        Ok(secret)
    }

    /// Check twofa setup secret is in cache or not
    pub async fn exists(
        redis: &Arc<Mutex<Connection>>,
        user: &ModelUser,
    ) -> Result<bool, ApiError> {
        let key = RedisKey::TwoFASetup(user.registered_user_id);
        let exists: bool = redis.lock().await.exists(key.to_string()).await?;
        Ok(exists)
    }
}
