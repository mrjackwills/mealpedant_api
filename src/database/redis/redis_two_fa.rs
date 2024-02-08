use redis::{aio::ConnectionManager, AsyncCommands, FromRedisValue, RedisResult, Value};
use serde::{Deserialize, Serialize};

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

    pub fn value(&self) -> &str {
        &self.0
    }

    fn key(registered_user_id: i64) -> String {
        RedisKey::TwoFASetup(registered_user_id).to_string()
    }

    // Insert new twofa secret & set ttl od 2 minutes
    pub async fn insert(
        &self,
        redis: &mut ConnectionManager,
        user: &ModelUser,
    ) -> Result<&Self, ApiError> {
        let key = Self::key(user.registered_user_id);
        let session = serde_json::to_string(&self)?;
        {
            redis.hset(&key, HASH_FIELD, session).await?;
            redis.expire(&key, 120).await?;
        }
        Ok(self)
    }

    /// Delete twofa secret
    pub async fn delete(redis: &mut ConnectionManager, user: &ModelUser) -> Result<(), ApiError> {
        // let key = RedisKey::TwoFASetup(user.registered_user_id);
        redis.del(Self::key(user.registered_user_id)).await?;
        Ok(())
    }

    /// get twofa setup secret
    pub async fn get(
        redis: &mut ConnectionManager,
        user: &ModelUser,
    ) -> Result<Option<Self>, ApiError> {
        Ok(redis
            .hget(Self::key(user.registered_user_id), HASH_FIELD)
            .await?)
    }

    /// Check twofa setup secret is in cache or not
    pub async fn exists(redis: &mut ConnectionManager, user: &ModelUser) -> Result<bool, ApiError> {
        Ok(redis
            .hexists(Self::key(user.registered_user_id), HASH_FIELD)
            .await?)
    }
}
