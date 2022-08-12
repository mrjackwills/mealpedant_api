use crate::{api_error::ApiError, parse_env::AppEnv};
use redis::{
    aio::Connection, from_redis_value, ConnectionAddr, ConnectionInfo, RedisConnectionInfo, Value,
};
use serde::de::DeserializeOwned;
use std::{fmt, net::IpAddr, time::Duration};
use uuid::Uuid;

mod redis_new_user;
mod redis_rate_limit;
mod redis_session;
mod redis_two_fa;
pub use redis_new_user::RedisNewUser;
pub use redis_rate_limit::RateLimit;
pub use redis_session::RedisSession;
pub use redis_two_fa::RedisTwoFASetup;

const ONE_HOUR: usize = 60 * 60;

#[derive(Debug, Clone)]
pub enum RedisKey<'a> {
    VerifyEmail(&'a str),
    VerifySecret(&'a str),
    RateLimitIp(IpAddr),
    RateLimitEmail(String),
    CacheIp(IpAddr),
    Session(&'a Uuid),
    SessionSet(i64),
    CacheUseragent(&'a str),
    LastID,
    Category,
    AllMeals,
    TwoFASetup(i64),
}

// Store in a single hash, and put each in it's own field
// remove category, allmeals, lastid, and just have cache::food as key?
// is it worth it? flush cache would only have to remove a single key/value
// when updating, again only update a single key/value?
// impl<'a> RedisKey<'a> {
//     pub fn hash_field(&self) -> Option<String> {
//         match self {
//             Self::Category => Some("category".to_owned()),
//             Self::AllMeals => Some("all_meals".to_owned()),
//             Self::LastID => Some("last_id".to_owned()),
//             _ => None,
//         }
//     }
// }

impl<'a> fmt::Display for RedisKey<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let disp = match self {
            Self::AllMeals => "cache::all_meals".to_owned(),
            Self::CacheIp(ip) => format!("cache::ip::{}", ip),
            Self::CacheUseragent(useragent) => format!("cache::useragent::{}", useragent),
            Self::Category => "cache::category".to_owned(),
            Self::LastID => "cache::last_id".to_owned(),
            Self::RateLimitIp(ip) => format!("ratelimit::ip::{}", ip),
            Self::RateLimitEmail(email) => format!("ratelimit::email::{}", email),
            Self::Session(uuid) => format!("session::{}", uuid),
            Self::SessionSet(id) => format!("session_set::user::{}", id),
            Self::TwoFASetup(id) => format!("two_fa_setup::{}", id),
            Self::VerifyEmail(email) => format!("verify::email::{}", email),
            Self::VerifySecret(secret) => format!("verify::secret::{}", secret),
        };
        write!(f, "{}", disp)
    }
}

/// so struct/models can easily convert from redis strings into the Structs they are modelled on
pub fn string_to_struct<T>(v: &Value) -> Result<T, redis::RedisError>
where
    T: DeserializeOwned,
{
    let json_str: String = from_redis_value(v)?;
    let result: Result<T, serde_json::Error> = serde_json::from_str(&json_str);
    match result {
        Ok(v) => Ok(v),
        Err(_) => Err((redis::ErrorKind::TypeError, "Parse to JSON Failed").into()),
    }
}

// pub struct RateLimitCounts {
//     pub small: u16,
//     pub big: u16,
// }

// impl RateLimitCounts {
//     fn new(small: u16, big: u16) -> Self {
//         Self { small, big }
//     }
// }

// // Not really used anywhere - yet
// impl AuthenticationLevel {
//     pub fn rate_limit_count(&self) -> RateLimitCounts {
//         match self {
//             Self::Incognito => RateLimitCounts::new(30, 60),
//             Self::User => RateLimitCounts::new(80, 160),
//             Self::Admin => RateLimitCounts::new(120, 240),
//         }
//     }
// }

pub struct DbRedis;

impl DbRedis {
    // pub async fn check_rate_limit(
    //     redis: &Arc<Mutex<Connection>>,
    //     ip: IpAddr,
    //     op_uuid: Option<Uuid>,
    // ) -> Result<(), ApiError> {
    //     let mut key = RedisKey::RateLimitIp(ip);
    //     if let Some(uuid) = op_uuid {
    //         if let Some(session) = RedisSession::exists(redis, &uuid).await? {
    //             key = RedisKey::RateLimitUser(session.registered_user_id);
    //         }
    //     };

    //     let count: Option<usize> = redis.lock().await.get(key.to_string()).await?;
    //     redis.lock().await.incr(key.to_string(), 1).await?;

    //     // Only increasing ttl if NOT already blocked
    //     // Has to be -1 of whatever limit you want, as first request doesn't count
    //     if let Some(i) = count {
    //         // If bigger than 180, rate limit for 5 minutes
    //         if i >= 180 {
    //             redis.lock().await.expire(key.to_string(), 60 * 5).await?;
    //             let ttl: usize = redis.lock().await.ttl(key.to_string()).await?;
    //             return Err(ApiError::RateLimited(ttl));
    //         }
    //         if i >= 90 {
    //             let ttl: usize = redis.lock().await.ttl(key.to_string()).await?;
    //             return Err(ApiError::RateLimited(ttl));
    //         };
    //     }
    //     redis.lock().await.expire(key.to_string(), 60).await?;
    //     Ok(())
    // }

    /// Open up a redis connection, to be saved in an Arc<Mutex> in application state
    pub async fn get_connection(app_env: &AppEnv) -> Result<Connection, ApiError> {
        let connection_info = ConnectionInfo {
            redis: RedisConnectionInfo {
                db: i64::from(app_env.redis_database),
                password: Some(app_env.redis_password.clone()),
                username: None,
            },
            addr: ConnectionAddr::Tcp(app_env.redis_host.clone(), app_env.redis_port),
        };
        let client = redis::Client::open(connection_info)?;
        match tokio::time::timeout(Duration::from_secs(10), client.get_async_connection()).await {
            Ok(con) => Ok(con?),
            Err(_) => Err(ApiError::Internal("Unable to connect to redis".to_owned())),
        }
    }
}

/// cargo watch -q -c -w src/ -x 'test db_redis_mod -- --test-threads=1 --nocapture'
#[cfg(test)]
#[allow(clippy::pedantic, clippy::nursery, clippy::unwrap_used)]
mod tests {

    use redis::{cmd, RedisError};

    use crate::parse_env;

    use super::*;

    #[tokio::test]
    async fn db_redis_mod_get_connection_and_ping() {
        let app_env = parse_env::AppEnv::get_env();
        let result = DbRedis::get_connection(&app_env).await;
        assert!(result.is_ok());

        let result: Result<String, RedisError> =
            cmd("PING").query_async(&mut result.unwrap()).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "PONG");
    }
}
