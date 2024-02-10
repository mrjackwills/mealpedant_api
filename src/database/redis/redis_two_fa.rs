use super::RedisKey;
use crate::{api_error::ApiError, database::ModelUser};
use fred::{clients::RedisPool, interfaces::KeysInterface};
use serde::{Deserialize, Serialize};

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

    // Insert new twofa secret & set ttl of 2 minutes
    pub async fn insert(&self, redis: &RedisPool, user: &ModelUser) -> Result<&Self, ApiError> {
        let key = Self::key(user.registered_user_id);
        redis
            .set(
                &key,
                self.value(),
                Some(fred::types::Expiration::EX(120)),
                None,
                false,
            )
            .await?;
        Ok(self)
    }

    /// Delete twofa secret
    pub async fn delete(redis: &RedisPool, user: &ModelUser) -> Result<(), ApiError> {
        Ok(redis.del(Self::key(user.registered_user_id)).await?)
    }

    /// get twofa setup secret
    pub async fn get(redis: &RedisPool, user: &ModelUser) -> Result<Option<Self>, ApiError> {
        (redis
            .get::<Option<String>, String>(Self::key(user.registered_user_id))
            .await?)
            .map_or_else(|| Ok(None), |x| Ok(Some(Self(x))))
    }

    /// Check twofa setup secret is in cache or not
    pub async fn exists(redis: &RedisPool, user: &ModelUser) -> Result<bool, ApiError> {
        Ok(redis.exists(Self::key(user.registered_user_id)).await?)
    }
}
