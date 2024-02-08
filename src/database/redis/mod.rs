use crate::{api_error::ApiError, parse_env::AppEnv};
use redis::{
    aio::ConnectionManager, from_redis_value, ConnectionAddr, ConnectionInfo, RedisConnectionInfo,
    Value,
};
use serde::de::DeserializeOwned;
use std::{fmt, net::IpAddr};
use uuid::Uuid;

mod redis_new_user;
mod redis_rate_limit;
mod redis_session;
mod redis_two_fa;
pub use redis_new_user::RedisNewUser;
pub use redis_rate_limit::RateLimit;
pub use redis_session::RedisSession;
pub use redis_two_fa::RedisTwoFASetup;

const ONE_MINUTE_IN_SEC: i64 = 60;
const ONE_HOUR_IN_SEC: i64 = ONE_MINUTE_IN_SEC * 60;

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

pub const HASH_FIELD: &str = "data";

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
            Self::CacheIp(ip) => format!("cache::ip::{ip}"),
            Self::CacheUseragent(useragent) => format!("cache::useragent::{useragent}"),
            Self::Category => "cache::category".to_owned(),
            Self::LastID => "cache::last_id".to_owned(),
            Self::RateLimitIp(ip) => format!("ratelimit::ip::{ip}"),
            Self::RateLimitEmail(email) => format!("ratelimit::email::{email}"),
            Self::Session(uuid) => format!("session::{uuid}"),
            Self::SessionSet(id) => format!("session_set::user::{id}"),
            Self::TwoFASetup(id) => format!("two_fa_setup::{id}"),
            Self::VerifyEmail(email) => format!("verify::email::{email}"),
            Self::VerifySecret(secret) => format!("verify::secret::{secret}"),
        };
        write!(f, "{disp}")
    }
}

/// so struct/models can easily convert from redis strings into the Structs they are modelled on
pub fn string_to_struct<T>(v: &Value) -> Result<T, redis::RedisError>
where
    T: DeserializeOwned,
{
    let json_str: String = from_redis_value(v)?;
    let result: Result<T, serde_json::Error> = serde_json::from_str(&json_str);
    result.map_or(
        Err((redis::ErrorKind::TypeError, "Parse to JSON Failed").into()),
        |v| Ok(v),
    )
}

pub struct DbRedis;

impl DbRedis {
    /// Open up a redis connection, to be saved in an Arc<Mutex> in application state
    /// Get an async redis connection
    pub async fn get_connection(app_env: &AppEnv) -> Result<ConnectionManager, ApiError> {
        let connection_info = ConnectionInfo {
            redis: RedisConnectionInfo {
                db: i64::from(app_env.redis_database),
                password: Some(app_env.redis_password.clone()),
                username: None,
            },
            addr: ConnectionAddr::Tcp(app_env.redis_host.clone(), app_env.redis_port),
        };

        Ok(redis::aio::ConnectionManager::new(redis::Client::open(connection_info)?).await?)
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
