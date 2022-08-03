use std::sync::Arc;

use redis::{aio::Connection, AsyncCommands, FromRedisValue, RedisResult, Value};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::{api_error::ApiError, argon::ArgonHash, database::ModelUserAgentIp};

use super::{RedisKey, ONE_HOUR};

impl FromRedisValue for RedisNewUser {
    fn from_redis_value(v: &Value) -> RedisResult<Self> {
        super::string_to_struct::<Self>(v)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RedisNewUser {
    pub email: String,
    pub full_name: String,
    pub password_hash: String,
    pub ip_id: i64,
    pub user_agent_id: i64,
}

impl RedisNewUser {
    pub fn new(email: &str, name: &str, password_hash: ArgonHash, req: ModelUserAgentIp) -> Self {
        Self {
            email: email.to_owned(),
            full_name: name.to_owned(),
            password_hash: password_hash.to_string(),
            ip_id: req.ip_id,
            user_agent_id: req.user_agent_id,
        }
    }

    /// On register, insert a new user into redis cache, to be inserted into postgres once verify email responded to
    pub async fn insert(
        &self,
        redis: &Arc<Mutex<Connection>>,
        secret: &str,
    ) -> Result<(), ApiError> {
        let secret_key = RedisKey::VerifySecret(secret);
        let email_key = RedisKey::VerifyEmail(&self.email);
        let ttl = ONE_HOUR;

        redis
            .lock()
            .await
            .set(email_key.to_string(), &secret)
            .await?;
        redis
            .lock()
            .await
            .expire(email_key.to_string(), ttl)
            .await?;

        let new_user_as_string = serde_json::to_string(&self)?;

        redis
            .lock()
            .await
            .set(secret_key.to_string(), &new_user_as_string)
            .await?;
        redis
            .lock()
            .await
            .expire(secret_key.to_string(), ttl)
            .await?;

        Ok(())
    }

    /// Remove both verify keys from redis
    pub async fn delete(
        &self,
        redis: &Arc<Mutex<Connection>>,
        secret: &str,
    ) -> Result<(), ApiError> {
        let secret_key = RedisKey::VerifySecret(secret);
        let email_key = RedisKey::VerifyEmail(&self.email);

        redis.lock().await.del(secret_key.to_string()).await?;
        redis.lock().await.del(email_key.to_string()).await?;
        Ok(())
    }

    /// Just check if a email is in redis cache, so that if a user has register but not yet verified, cannot sign up again
    /// Static method, as want to use before one creates a NewUser struct
    pub async fn exists(redis: &Arc<Mutex<Connection>>, email: &str) -> Result<bool, ApiError> {
        let email_key = RedisKey::VerifyEmail(email);
        Ok(redis.lock().await.exists(email_key.to_string()).await?)
    }

    /// Verify a new account, secret emailed to user, user visits url with secret as a param
    pub async fn get(con: &Arc<Mutex<Connection>>, secret: &str) -> Result<Option<Self>, ApiError> {
        let secret_key = RedisKey::VerifySecret(secret);
        let new_user: Option<Self> = con.lock().await.get(secret_key.to_string()).await?;
        Ok(new_user)
    }
}

/// cargo watch -q -c -w src/ -x 'test redis_mod_newuser -- --test-threads=1 --nocapture'
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {

    use redis::AsyncCommands;

    type R<T> = Result<T, redis::RedisError>;

    use super::RedisNewUser;
    use crate::{
        api::api_tests::{setup, TEST_EMAIL},
        database::redis::RedisKey,
    };

    /// insert new user into redis, 2 keys (email&verify) inserted & both have correct ttl
    #[tokio::test]
    async fn redis_mod_newuser_insert() {
        let test_setup = setup().await;

        let new_user = RedisNewUser {
            email: TEST_EMAIL.to_owned(),
            full_name: String::from("name"),
            password_hash: String::from("password_hash"),
            ip_id: 1,
            user_agent_id: 1,
        };
        let secret = String::from("new_user_secret");

        let result = new_user.insert(&test_setup.redis, &secret).await;
        assert!(result.is_ok());

        let email_key = RedisKey::VerifyEmail(&new_user.email);
        let ttl: R<i32> = test_setup
            .redis
            .lock()
            .await
            .ttl(email_key.to_string())
            .await;

        assert!(ttl.is_ok());
        let ttl = ttl.unwrap();
        assert_eq!(ttl, 3600);

        let secret_key = RedisKey::VerifySecret(&secret);
        let ttl: R<i32> = test_setup
            .redis
            .lock()
            .await
            .ttl(secret_key.to_string())
            .await;
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
            full_name: String::from("name"),
            password_hash: String::from("password_hash"),
            ip_id: 1,
            user_agent_id: 1,
        };
        let secret = String::from("new_user_secret");

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
            full_name: String::from("name"),
            password_hash: String::from("password_hash"),
            ip_id: 1,
            user_agent_id: 1,
        };
        let secret = String::from("secret");

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
            full_name: String::from("name"),
            password_hash: String::from("password_hash"),
            ip_id: 1,
            user_agent_id: 1,
        };
        let secret = String::from("new_user_secret");

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
