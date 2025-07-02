use super::{HASH_FIELD, RedisKey};
use crate::{api_error::ApiError, database::ModelUser, hmap, redis_hash_to_struct};
use fred::{
    clients::Pool,
    interfaces::{HashesInterface, KeysInterface},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RedisTwoFASetup(String);

redis_hash_to_struct!(RedisTwoFASetup);

impl RedisTwoFASetup {
    pub fn new(secret: &str) -> Self {
        Self(secret.to_owned())
    }

    // #[allow(clippy::missing_const_for_fn)]
    pub fn value(&self) -> &str {
        &self.0
    }

    fn key(registered_user_id: i64) -> String {
        RedisKey::TwoFASetup(registered_user_id).to_string()
    }

    /// Insert new twofa secret & set ttl of 2 minutes
    pub async fn insert(&self, redis: &Pool, user: &ModelUser) -> Result<&Self, ApiError> {
        let key = Self::key(user.registered_user_id);
        let session = serde_json::to_string(&self)?;
        redis.hset::<(), _, _>(&key, hmap!(session)).await?;
        redis.expire::<(), _>(&key, 120, None).await?;
        Ok(self)
    }

    /// Delete twofa secret
    pub async fn delete(redis: &Pool, user: &ModelUser) -> Result<(), ApiError> {
        Ok(redis.del(Self::key(user.registered_user_id)).await?)
    }

    /// get twofa setup secret
    pub async fn get(redis: &Pool, user: &ModelUser) -> Result<Option<Self>, ApiError> {
        Ok(redis
            .hget(Self::key(user.registered_user_id), HASH_FIELD)
            .await?)
    }

    /// Check twofa setup secret is in cache or not
    pub async fn exists(redis: &Pool, user: &ModelUser) -> Result<bool, ApiError> {
        Ok(redis
            .hexists::<bool, String, &str>(Self::key(user.registered_user_id), HASH_FIELD)
            .await?)
    }
}
