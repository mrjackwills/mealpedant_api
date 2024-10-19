use super::{RedisKey, HASH_FIELD, ONE_HOUR_AS_SEC};
use crate::{
    api_error::ApiError, argon::ArgonHash, database::ModelUserAgentIp, hmap, redis_hash_to_struct,
};
use fred::{
    clients::RedisPool,
    interfaces::{HashesInterface, KeysInterface},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RedisNewUser {
    pub email: String,
    pub full_name: String,
    pub password_hash: String,
    pub ip_id: i64,
    pub user_agent_id: i64,
}

redis_hash_to_struct!(RedisNewUser);

impl RedisNewUser {
    fn key_email(email: &str) -> String {
        RedisKey::VerifyEmail(email).to_string()
    }

    fn key_secret(secret: &str) -> String {
        RedisKey::VerifySecret(secret).to_string()
    }

    pub fn new(email: &str, name: &str, password_hash: &ArgonHash, req: &ModelUserAgentIp) -> Self {
        Self {
            email: email.to_owned(),
            full_name: name.to_owned(),
            password_hash: password_hash.to_string(),
            ip_id: req.ip_id,
            user_agent_id: req.user_agent_id,
        }
    }

    /// On register, insert a new user into redis cache, to be inserted into postgres once verify email responded to
    pub async fn insert(&self, redis: &RedisPool, secret: &str) -> Result<(), ApiError> {
        let key_secret = Self::key_secret(secret);
        let key_email = Self::key_email(&self.email);

        let new_user_as_string = serde_json::to_string(&self)?;

        redis.hset::<(), _, _>(&key_email, hmap!(secret)).await?;
        redis.expire::<(), _>(key_email, ONE_HOUR_AS_SEC).await?;
        redis
            .hset::<(), _, _>(&key_secret, hmap!(new_user_as_string))
            .await?;
        Ok(redis.expire(key_secret, ONE_HOUR_AS_SEC).await?)
    }

    /// Remove both verify keys from redis
    pub async fn delete(&self, redis: &RedisPool, secret: &str) -> Result<(), ApiError> {
        let _: () = redis.del(Self::key_secret(secret)).await?;
        Ok(redis.del(Self::key_email(&self.email)).await?)
    }

    /// Just check if a email is in redis cache, so that if a user has register but not yet verified, cannot sign up again
    /// Static method, as want to use before one creates a NewUser struct
    pub async fn exists(redis: &RedisPool, email: &str) -> Result<bool, ApiError> {
        Ok(redis.exists(Self::key_email(email)).await?)
    }

    /// Verify a new account, secret emailed to user, user visits url with secret as a param
    pub async fn get(redis: &RedisPool, secret: &str) -> Result<Option<Self>, ApiError> {
        Ok(redis.hget(Self::key_secret(secret), HASH_FIELD).await?)
    }
}

/// cargo watch -q -c -w src/ -x 'test redis_mod_newuser -- --test-threads=1 --nocapture'
#[cfg(test)]
#[expect(clippy::pedantic, clippy::unwrap_used)]
mod tests {

    type R<T> = Result<T, fred::error::RedisError>;

    use fred::interfaces::KeysInterface;

    use super::RedisNewUser;
    use crate::{
        api::api_tests::{setup, TEST_EMAIL},
        database::redis::RedisKey,
        S,
    };

    /// insert new user into redis, 2 keys (email&verify) inserted & both have correct ttl
    #[tokio::test]
    async fn redis_mod_newuser_insert() {
        let test_setup = setup().await;

        let new_user = RedisNewUser {
            email: TEST_EMAIL.to_owned(),
            full_name: S!("name"),
            password_hash: S!("password_hash"),
            ip_id: 1,
            user_agent_id: 1,
        };
        let secret = S!("new_user_secret");

        let result = new_user.insert(&test_setup.redis, &secret).await;
        assert!(result.is_ok());

        let email_key = RedisKey::VerifyEmail(&new_user.email);
        let ttl: R<i32> = test_setup.redis.ttl(email_key.to_string()).await;

        assert!(ttl.is_ok());
        let ttl = ttl.unwrap();
        assert_eq!(ttl, 3600);

        let secret_key = RedisKey::VerifySecret(&secret);
        let ttl: R<i32> = test_setup.redis.ttl(secret_key.to_string()).await;
        assert!(ttl.is_ok());

        let ttl = ttl.unwrap();
        assert_eq!(ttl, 3600);
    }

    /// get_by_secret & get_by_email return Some(new_user)/Some(secret)
    #[tokio::test]
    async fn redis_mod_newuser_get_some() {
        let test_setup = setup().await;
        let new_user = RedisNewUser {
            email: TEST_EMAIL.to_owned(),
            full_name: S!("name"),
            password_hash: S!("password_hash"),
            ip_id: 1,
            user_agent_id: 1,
        };
        let secret = S!("new_user_secret");

        let insert = new_user.insert(&test_setup.redis, &secret).await;
        assert!(insert.is_ok());

        let result = RedisNewUser::get(&test_setup.redis, &secret).await;

        assert!(result.is_ok());
        assert!(result.as_ref().unwrap().is_some());
        assert_eq!(result.unwrap().unwrap(), new_user);

        let result = RedisNewUser::exists(&test_setup.redis, &new_user.email).await;

        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    /// get_by_secret return None of wrong keyname
    #[tokio::test]
    async fn redis_mod_newuser_get_none() {
        let test_setup = setup().await;
        let new_user = RedisNewUser {
            email: TEST_EMAIL.to_owned(),
            full_name: S!("name"),
            password_hash: S!("password_hash"),
            ip_id: 1,
            user_agent_id: 1,
        };
        let secret = S!("secret");

        let insert = new_user.insert(&test_setup.redis, &secret).await;
        assert!(insert.is_ok());

        let result = RedisNewUser::get(&test_setup.redis, "Secret").await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    /// delete removes both keys (verify::email & verify::secret) from redis
    #[tokio::test]
    async fn redis_mod_newuser_delete() {
        let test_setup = setup().await;
        let new_user = RedisNewUser {
            email: TEST_EMAIL.to_owned(),
            full_name: S!("name"),
            password_hash: S!("password_hash"),
            ip_id: 1,
            user_agent_id: 1,
        };
        let secret = S!("new_user_secret");

        let insert = new_user.insert(&test_setup.redis, &secret).await;
        assert!(insert.is_ok());

        let result = RedisNewUser::exists(&test_setup.redis, &new_user.email).await;
        assert!(result.is_ok());
        assert!(result.unwrap());

        let result = RedisNewUser::get(&test_setup.redis, &secret).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());

        let result = new_user.delete(&test_setup.redis, &secret).await;
        assert!(result.is_ok());

        let result = RedisNewUser::exists(&test_setup.redis, &new_user.email).await;
        assert!(result.is_ok());
        assert!(!result.unwrap());

        let result = RedisNewUser::get(&test_setup.redis, &secret).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }
}
