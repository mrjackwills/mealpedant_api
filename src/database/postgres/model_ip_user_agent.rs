use redis::{aio::Connection, AsyncCommands};
use std::{
    net::{IpAddr, SocketAddr},
    sync::Arc,
};

use axum::{
    async_trait,
    extract::{ConnectInfo, FromRequest, RequestParts},
};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres, Transaction};
use tokio::sync::Mutex;

use crate::{
    api::{get_ip, get_state, get_user_agent_header},
    api_error::ApiError,
    database::redis::{RedisKey, HASH_FIELD},
};


#[derive(Debug, Clone)]
pub struct ReqUserAgentIp {
    pub user_agent: String,
    pub ip: IpAddr,
}

#[derive(sqlx::FromRow, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct Ip {
    ip_id: i64,
}

#[derive(sqlx::FromRow, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct Useragent {
    user_agent_id: i64,
}

#[derive(sqlx::FromRow, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelUserAgentIp {
    pub user_agent_id: i64,
    pub user_agent: String,
    pub ip_id: i64,
    pub ip: IpAddr,
}


impl ModelUserAgentIp {

	fn key_ip(ip: IpAddr) -> String{
		RedisKey::CacheIp(ip).to_string()
	}
	
	fn key_useragent(useragent: &str) -> String{
		RedisKey::CacheUseragent(useragent).to_string()
	}

	
    async fn insert_cache(&self, redis: &Arc<Mutex<Connection>>) -> Result<(), ApiError> {

        redis
            .lock()
            .await
            .hset(Self::key_ip(self.ip), HASH_FIELD, self.ip_id)
            .await?;
        redis
            .lock()
            .await
            .hset(Self::key_useragent(&self.user_agent), HASH_FIELD, self.user_agent_id)
            .await?;
        Ok(())
    }

    async fn get_cache(
        redis: &Arc<Mutex<Connection>>,
        ip: IpAddr,
        user_agent: &str,
    ) -> Result<Option<Self>, ApiError> {
        let o_ip_id: Option<i64> = redis.lock().await.hget(Self::key_ip(ip), HASH_FIELD).await?;
        let o_useragent_id: Option<i64> =
            redis.lock().await.hget(Self::key_useragent(user_agent), HASH_FIELD).await?;
        if let (Some(ip_id), Some(user_agent_id)) = (o_ip_id, o_useragent_id) {
            Ok(Some(Self {
                ip,
                user_agent: user_agent.to_owned(),
                ip_id,
                user_agent_id,
            }))
        } else {
            Ok(None)
        }
    }

    /// Have to cast as inet in the query, until sqlx gets updated
    /// get ip_id
    async fn get_ip(
        transaction: &mut Transaction<'_, Postgres>,
        req: &ReqUserAgentIp,
    ) -> Result<Option<Ip>, sqlx::Error> {
        let query = r"SELECT ip_id from ip_address WHERE ip = $1::inet";
        sqlx::query_as::<_, Ip>(query)
            .bind(req.ip.to_string())
            .fetch_optional(&mut *transaction)
            .await
    }

    /// Have to cast as inet in the query, until sqlx gets updated
    /// Insert ip into postgres
    async fn insert_ip(
        transaction: &mut Transaction<'_, Postgres>,
        req: &ReqUserAgentIp,
    ) -> Result<Ip, sqlx::Error> {
        let query = "INSERT INTO ip_address(ip) VALUES ($1::inet) RETURNING ip_id";
        sqlx::query_as::<_, Ip>(query)
            .bind(req.ip.to_string())
            .fetch_one(&mut *transaction)
            .await
    }

    /// get user_agent_id
    async fn get_user_agent(
        transaction: &mut Transaction<'_, Postgres>,
        req: &ReqUserAgentIp,
    ) -> Result<Option<Useragent>, sqlx::Error> {
        let query = r"SELECT user_agent_id from user_agent WHERE user_agent_string = $1";
        sqlx::query_as::<_, Useragent>(query)
            .bind(req.user_agent.clone())
            .fetch_optional(&mut *transaction)
            .await
    }

    /// Insert useragent into postgres
    async fn insert_user_agent(
        transaction: &mut Transaction<'_, Postgres>,
        req: &ReqUserAgentIp,
    ) -> Result<Useragent, sqlx::Error> {
        let query =
            r"INSERT INTO user_agent(user_agent_string) VALUES ($1) RETURNING user_agent_id";
        sqlx::query_as::<_, Useragent>(query)
            .bind(req.user_agent.clone())
            .fetch_one(&mut *transaction)
            .await
    }

    /// get ip_id and user_agent_id
    pub async fn get(
        postgres: &PgPool,
        redis: &Arc<Mutex<Connection>>,
        req: &ReqUserAgentIp,
    ) -> Result<Self, ApiError> {
        if let Some(cache) = Self::get_cache(redis, req.ip, &req.user_agent).await? {
            return Ok(cache);
        }

        let mut transaction = postgres.begin().await?;
        let ip_id = if let Some(ip) = Self::get_ip(&mut transaction, req).await? {
            ip
        } else {
            Self::insert_ip(&mut transaction, req).await?
        };
        let user_agent_id =
            if let Some(user_agent) = Self::get_user_agent(&mut transaction, req).await? {
                user_agent
            } else {
                Self::insert_user_agent(&mut transaction, req).await?
            };
        transaction.commit().await?;

        let output = Self {
            user_agent: req.user_agent.clone(),
            ip: req.ip,
            user_agent_id: user_agent_id.user_agent_id,
            ip_id: ip_id.ip_id,
        };

        output.insert_cache(redis).await?;

        Ok(output)
    }
}

#[async_trait]
impl<B> FromRequest<B> for ModelUserAgentIp
where
    B: Send,
{
    type Rejection = ApiError;

    /// Get, or insert, ip_address & user agent into db, and inject into handler, if so required
    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let state = get_state(req.extensions())?;
        let addr: Option<&ConnectInfo<SocketAddr>> = req.extensions().get();
        let useragent_ip = ReqUserAgentIp {
            user_agent: get_user_agent_header(req.headers()),
            ip: get_ip(req.headers(), addr),
        };
        Ok(Self::get(&state.postgres, &state.redis, &useragent_ip).await?)
    }
}

/// cargo watch -q -c -w src/ -x 'test db_postgres_model_ip_useragent -- --test-threads=1 --nocapture'
#[cfg(test)]
#[allow(clippy::pedantic, clippy::nursery, clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::api::api_tests::{setup, TestSetup};

    #[tokio::test]
    /// Retuns None
    async fn db_postgres_model_ip_useragent_get_ip_transaction() {
        let test_setup = setup().await;
        let mut transaction = test_setup.postgres.begin().await.unwrap();
        let req = TestSetup::gen_req();

        let result = ModelUserAgentIp::get_ip(&mut transaction, &req).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
        transaction.rollback().await.unwrap();
    }

    #[tokio::test]
    /// Insert returns Ok(), and ip_id > 0
    async fn db_postgres_model_ip_useragent_insert_ip_transaction() {
        let test_setup = setup().await;
        let mut transaction = test_setup.postgres.begin().await.unwrap();
        let req = TestSetup::gen_req();

        let result = ModelUserAgentIp::insert_ip(&mut transaction, &req).await;
        assert!(result.is_ok());
        assert!(result.unwrap().ip_id > 0);
        transaction.rollback().await.unwrap();
    }

    #[tokio::test]
    /// Insert ok, and get ok
    async fn db_postgres_model_ip_useragent_insert_get_ip_transaction() {
        let test_setup = setup().await;
        let mut transaction = test_setup.postgres.begin().await.unwrap();
        let req = TestSetup::gen_req();
        let result = ModelUserAgentIp::insert_ip(&mut transaction, &req).await;
        assert!(result.is_ok());

        let result = ModelUserAgentIp::get_ip(&mut transaction, &req).await;
        assert!(result.is_ok());
        assert!(result.as_ref().unwrap().is_some());
        assert!(result.unwrap().unwrap().ip_id > 0);

        transaction.rollback().await.unwrap();
    }

    #[tokio::test]
    /// Retuns None
    async fn db_postgres_model_ip_useragent_get_user_agent_transaction() {
        let test_setup = setup().await;
        let mut transaction = test_setup.postgres.begin().await.unwrap();
        let req = TestSetup::gen_req();

        let result = ModelUserAgentIp::get_user_agent(&mut transaction, &req).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
        transaction.rollback().await.unwrap();
    }

    #[tokio::test]
    /// Insert returns Ok(), and user_agent_id > 0
    async fn db_postgres_model_ip_useragent_insert_user_agent_transaction() {
        let test_setup = setup().await;
        let mut transaction = test_setup.postgres.begin().await.unwrap();
        let req = TestSetup::gen_req();

        let result = ModelUserAgentIp::insert_user_agent(&mut transaction, &req).await;
        assert!(result.is_ok());
        assert!(result.unwrap().user_agent_id > 0);
        transaction.rollback().await.unwrap();
    }

    #[tokio::test]
    /// Insert ok, and get ok
    async fn db_postgres_model_ip_useragent_insert_get_user_agent_transaction() {
        let test_setup = setup().await;
        let mut transaction = test_setup.postgres.begin().await.unwrap();
        let req = TestSetup::gen_req();

        let result = ModelUserAgentIp::insert_user_agent(&mut transaction, &req).await;
        assert!(result.is_ok());

        let result = ModelUserAgentIp::get_user_agent(&mut transaction, &req).await;
        assert!(result.is_ok());
        assert!(result.as_ref().unwrap().is_some());
        assert!(result.unwrap().unwrap().user_agent_id > 0);

        transaction.rollback().await.unwrap();
    }

    #[tokio::test]
    /// Full test of get, will insert new ip & user agents
    async fn db_postgres_model_ip_useragent_get() {
        let test_setup = setup().await;
        let req = TestSetup::gen_req();

        let result = ModelUserAgentIp::get(&test_setup.postgres, &test_setup.redis, &req).await;
        assert!(result.is_ok());

        let result = result.unwrap();
        assert!(result.ip_id > 0);
        assert!(result.user_agent_id > 0);
        assert_eq!(result.ip, req.ip);
        assert_eq!(result.user_agent, req.user_agent);

        let cache: Vec<String> = test_setup.redis.lock().await.keys("*").await.unwrap();


        // Contains cache
        assert!(cache.contains(&"cache::useragent::test_user_agent".to_owned()));
        assert!(cache.contains(&"cache::ip::123.123.123.123".to_owned()));

        let cache: Option<i64> = test_setup
            .redis
            .lock()
            .await
            .hget("cache::useragent::test_user_agent", "data")
            .await
            .unwrap();
        assert!(cache.is_some());

        let cache: Option<i64> = test_setup
            .redis
            .lock()
            .await
            .hget("cache::ip::123.123.123.123", "data")
            .await
            .unwrap();
        assert!(cache.is_some());
    }
}
