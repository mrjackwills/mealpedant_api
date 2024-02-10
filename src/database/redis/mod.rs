use crate::{api_error::ApiError, parse_env::AppEnv};
use fred::{clients::RedisPool, interfaces::ClientLike, types::ReconnectPolicy};
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

const ONE_MINUTE_AS_SEC: i64 = 60;
const ONE_HOUR_AS_SEC: i64 = ONE_MINUTE_AS_SEC * 60;

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

pub struct DbRedis;

impl DbRedis {
    pub async fn get_pool(app_env: &AppEnv) -> Result<RedisPool, ApiError> {
        let redis_url = format!(
            "redis://:{password}@{host}:{port}/{db}",
            password = app_env.redis_password,
            host = app_env.redis_host,
            port = app_env.redis_port,
            db = app_env.redis_database
        );

        let config = fred::types::RedisConfig::from_url(&redis_url)?;
        let pool = fred::types::Builder::from_config(config)
            .with_performance_config(|config| {
                config.auto_pipeline = true;
            })
            // use exponential backoff, starting at 100 ms and doubling on each failed attempt up to 30 sec
            .set_policy(ReconnectPolicy::new_exponential(0, 100, 30_000, 2))
            .build_pool(32)?;
        pool.init().await?;
        Ok(pool)
    }
}

/// cargo watch -q -c -w src/ -x 'test db_redis_mod -- --test-threads=1 --nocapture'
#[cfg(test)]
#[allow(clippy::pedantic, clippy::nursery, clippy::unwrap_used)]
mod tests {

    use crate::parse_env;

    use super::*;

    #[tokio::test]
    async fn db_redis_mod_get_connection_and_ping() {
        let app_env = parse_env::AppEnv::get_env();
        let result = DbRedis::get_pool(&app_env).await;
        assert!(result.is_ok());
        let result = result.unwrap();

        let result = result.ping::<String>().await;

        // let result: Result<String, RedisError> =
        //     cmd("PING").query_async(&result.unwrap()).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "PONG");
    }
}
