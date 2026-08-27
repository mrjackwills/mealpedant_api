use fred::types::scan::Scanner;
use fred::{clients::Pool, interfaces::KeysInterface};
use futures::stream::TryStreamExt;
use std::net::IpAddr;
use ulid::Ulid;

use super::{ONE_MINUTE_AS_SEC, RedisKey, RedisSession};
use crate::api_error::ApiError;
use crate::servers::{ij::LimitKey, oj::Limit};

pub struct RateLimit;

impl RateLimit {
    fn key_ip(ip: IpAddr) -> String {
        RedisKey::RateLimitIp(ip).to_string()
    }

    fn key_email(email: String) -> String {
        RedisKey::RateLimitEmail(email).to_string()
    }

    /// Check an incoming request to see if it is ratelimited or not
    pub async fn check(redis: &Pool, ip: IpAddr, ulid: Option<Ulid>) -> Result<(), ApiError> {
        let (limits, key) = if let Some(ulid) = ulid
            && let Some(session) = RedisSession::exists(redis, &ulid).await?
        {
            // ideally we'd want to check if an admin user here, maybe load that into the session?
            // then would need to removed it when admin user status gets revoked
            ((1000, 500), Self::key_email(session.email))
        } else {
            ((400, 200), Self::key_ip(ip))
        };

        // Atomic: INCR returns the running total for this key in one round-trip,
        // so concurrent requests no longer observe a stale pre-increment count.
        let count = redis.incr::<usize, _>(&key).await?;

        // First request in a window starts the 60s TTL.
        if count == 1 {
            redis.expire::<(), _>(&key, ONE_MINUTE_AS_SEC, None).await?;
            return Ok(());
        }
        // The request that crosses the short limit locks the key for 1 minute.
        if count == limits.1 + 1 {
            redis.expire::<(), _>(&key, ONE_MINUTE_AS_SEC, None).await?;
            return Err(ApiError::RateLimited(ONE_MINUTE_AS_SEC));
        }
        // Over the short limit: escalate to the 5-minute lock once the big limit
        // is reached, otherwise keep rejecting on the remaining TTL.
        if count > limits.1 + 1 {
            if count > limits.0 {
                redis
                    .expire::<(), _>(&key, ONE_MINUTE_AS_SEC * 5, None)
                    .await?;
            }
            return Err(ApiError::RateLimited(redis.ttl::<i64, &str>(&key).await?));
        }

        Ok(())
    }

    /// Get all current rate limits - is either based on user_email or ip address
    /// Used by admin, keys("*") is not a great function to call
    pub async fn get_all(redis: &Pool) -> Result<Vec<Limit>, ApiError> {
        let mut output = vec![];
        let mut scanner = redis.next().scan("ratelimit::*", Some(100), None);
        while let Some(mut page) = scanner.try_next().await? {
            if let Some(page) = page.take_results() {
                for i in page {
                    let key = i.as_str().unwrap_or_default().to_owned();
                    let points = redis.get(&key).await?;
                    let key = key.split("::").skip(2).take(1).collect::<String>();
                    output.push(Limit { key, points });
                }
            }
            page.next();
        }
        Ok(output)
    }

    /// Get all current rate limits - is either based on user_email or ip address
    pub async fn delete(limit_key: LimitKey, redis: &Pool) -> Result<(), ApiError> {
        let key = match limit_key {
            LimitKey::Email(e) => Self::key_email(e),
            LimitKey::Ip(i) => Self::key_ip(i),
        };

        redis.del::<(), _>(key.clone()).await?;
        Ok(())
    }
}
