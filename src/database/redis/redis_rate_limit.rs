use fred::types::scan::Scanner;
use fred::{clients::Pool, interfaces::KeysInterface};
use futures::stream::TryStreamExt;
use std::net::IpAddr;
use uuid::Uuid;

use super::{ONE_MINUTE_AS_SEC, RedisKey, RedisSession};
use crate::api::oj::Limit;
use crate::{api::ij::LimitKey, api_error::ApiError};

pub struct RateLimit;

impl RateLimit {
    fn key_ip(ip: IpAddr) -> String {
        RedisKey::RateLimitIp(ip).to_string()
    }

    fn key_email(email: String) -> String {
        RedisKey::RateLimitEmail(email).to_string()
    }

    /// Check an incoming request to see if it is ratelimited or not
    pub async fn check(redis: &Pool, ip: IpAddr, op_uuid: Option<Uuid>) -> Result<(), ApiError> {
        let mut key = Self::key_ip(ip);
        if let Some(uuid) = op_uuid {
            if let Some(session) = RedisSession::exists(redis, &uuid).await? {
                key = Self::key_email(session.email);
            }
        };

        let count = redis.get::<Option<usize>, &str>(&key).await?;
        redis.incr::<(), _>(&key).await?;
        if let Some(count) = count {
            if count >= 180 {
                redis
                    .expire::<(), _>(&key, ONE_MINUTE_AS_SEC * 5, None)
                    .await?;
            }
            if count > 90 {
                return Err(ApiError::RateLimited(redis.ttl::<i64, &str>(&key).await?));
            };
            if count == 90 {
                redis.expire::<(), _>(&key, ONE_MINUTE_AS_SEC, None).await?;
                return Err(ApiError::RateLimited(ONE_MINUTE_AS_SEC));
            }
        } else {
            redis.expire::<(), _>(&key, ONE_MINUTE_AS_SEC, None).await?;
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
                    let points: u64 = redis.get(&key).await?;
                    // trim key - so that it's just ip or email
                    let key = key.split("::").skip(2).take(1).collect::<String>();
                    output.push(Limit { key, points });
                }
            }
            page.next();
        }
        Ok(output)
    }

    // Get all current rate limits - is either based on user_email or ip address
    pub async fn delete(limit_key: LimitKey, redis: &Pool) -> Result<(), ApiError> {
        let key = match limit_key {
            LimitKey::Email(e) => Self::key_email(e),
            LimitKey::Ip(i) => Self::key_ip(i),
        };

        redis.del::<(), _>(key.to_string()).await?;
        Ok(())
    }
}
