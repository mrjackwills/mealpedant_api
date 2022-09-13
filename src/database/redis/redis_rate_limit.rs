use std::{net::IpAddr, sync::Arc};

use redis::{aio::Connection, AsyncCommands};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{api::ij::LimitKey, api_error::ApiError};

use super::{RedisKey, RedisSession};
use crate::api::oj::Limit;

pub struct RateLimit;

const ONE_MINUTE: usize = 60;

impl RateLimit {
    fn key_ip(ip: IpAddr) -> String {
        RedisKey::RateLimitIp(ip).to_string()
    }

    fn key_email(email: String) -> String {
        RedisKey::RateLimitEmail(email).to_string()
    }

    pub async fn check(
        redis: &Arc<Mutex<Connection>>,
        ip: IpAddr,
        op_uuid: Option<Uuid>,
    ) -> Result<(), ApiError> {
        let mut key = Self::key_ip(ip);
        if let Some(uuid) = op_uuid {
            if let Some(session) = RedisSession::exists(redis, &uuid).await? {
                key = Self::key_email(session.email);
            }
        };

        let count: Option<usize> = redis.lock().await.get(&key).await?;
        redis.lock().await.incr(&key, 1).await?;

        // Only increasing ttl if NOT already blocked
        // Has to be -1 of whatever limit you want, as first request doesn't count
        if let Some(i) = count {
            // If bigger than 180, rate limit for 5 minutes
            if i >= 180 {
                redis.lock().await.expire(&key, ONE_MINUTE * 5).await?;
                return Err(ApiError::RateLimited(ONE_MINUTE * 5));
            }
            if i > 90 {
                let ttl: usize = redis.lock().await.ttl(&key).await?;
                return Err(ApiError::RateLimited(ttl));
            };
            if i == 90 {
                redis.lock().await.expire(&key, ONE_MINUTE).await?;
                return Err(ApiError::RateLimited(ONE_MINUTE));
            }
        } else {
            redis.lock().await.expire(&key, ONE_MINUTE).await?;
        }
        Ok(())
    }

    // Get all current rate limits - is either based on user_email or ip address
    pub async fn get_all(redis: &Arc<Mutex<Connection>>) -> Result<Vec<Limit>, ApiError> {
        let mut output = vec![];
        let all_keys: Vec<String> = redis.lock().await.keys("ratelimit::*").await?;

        for key in all_keys {
            let points: u64 = redis.lock().await.get(&key).await?;
            // trim key - so that it's just ip or email
            let key = key.split("::").skip(2).take(1).collect::<String>();
            output.push(Limit { key, points });
        }
        Ok(output)
    }

    // Get all current rate limits - is either based on user_email or ip address
    pub async fn delete(
        limit_key: LimitKey,
        redis: &Arc<Mutex<Connection>>,
    ) -> Result<(), ApiError> {
        let key = match limit_key {
            LimitKey::Email(e) => Self::key_email(e),
            LimitKey::Ip(i) => Self::key_ip(i),
        };

        redis.lock().await.del(key.to_string()).await?;
        Ok(())
    }
}
