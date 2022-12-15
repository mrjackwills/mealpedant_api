use std::sync::Arc;

use redis::{aio::Connection, AsyncCommands, FromRedisValue, RedisResult, Value};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::{api_error::ApiError, database::ModelUser};

use super::{RedisKey, HASH_FIELD};

impl FromRedisValue for RedisTwoFASetup {
    fn from_redis_value(v: &Value) -> RedisResult<Self> {
        super::string_to_struct::<Self>(v)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RedisTwoFASetup(String);

impl RedisTwoFASetup {
    pub fn new(secret: &str) -> Self {
        Self(secret.to_owned())
    }

	pub fn value(&self) -> String {
		(*self.0).to_string()
	}

    fn key(registered_user_id: i64) -> String {
        RedisKey::TwoFASetup(registered_user_id).to_string()
    }

    // Insert new twofa secret & set ttl od 2 minutes
    pub async fn insert(
        &self,
        redis: &Arc<Mutex<Connection>>,
        user: &ModelUser,
    ) -> Result<&Self, ApiError> {
        let key = Self::key(user.registered_user_id);
        let session = serde_json::to_string(&self)?;
        redis.lock().await.hset(&key, HASH_FIELD, session).await?;
        redis.lock().await.expire(&key, 120).await?;
        Ok(self)
    }

    /// Delete twofa secret
    pub async fn delete(redis: &Arc<Mutex<Connection>>, user: &ModelUser) -> Result<(), ApiError> {
        // let key = RedisKey::TwoFASetup(user.registered_user_id);
        redis
            .lock()
            .await
            .del(Self::key(user.registered_user_id))
            .await?;
        Ok(())
    }

    /// get twofa setup secret
    pub async fn get(
        redis: &Arc<Mutex<Connection>>,
        user: &ModelUser,
    ) -> Result<Option<Self>, ApiError> {
        Ok(redis
            .lock()
            .await
            .hget(Self::key(user.registered_user_id), HASH_FIELD)
            .await?)
    }

    /// Check twofa setup secret is in cache or not
    pub async fn exists(
        redis: &Arc<Mutex<Connection>>,
        user: &ModelUser,
    ) -> Result<bool, ApiError> {
        Ok(redis
            .lock()
            .await
            .hexists(Self::key(user.registered_user_id), HASH_FIELD)
            .await?)
    }
}
