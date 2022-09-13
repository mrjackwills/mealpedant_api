use axum::{
    async_trait,
    extract::{FromRequest, RequestParts},
};
use axum_extra::extract::PrivateCookieJar;
use cookie::Key;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    api::get_state,
    api_error::ApiError,
    argon::ArgonHash,
    database::{RedisNewUser, RedisSession},
};

#[derive(sqlx::FromRow, Debug, Clone, PartialEq, Eq)]
pub struct ModelUser {
    pub registered_user_id: i64,
    pub full_name: String,
    pub email: String,
    pub active: bool,
    pub login_attempt_number: i64,
    pub two_fa_secret: Option<String>,
    pub two_fa_always_required: bool,
    pub two_fa_backup_count: i64,
    pub admin: bool,
    password_hash: String,
}

impl ModelUser {
    pub fn get_password_hash(&self) -> ArgonHash {
        ArgonHash {
            password_hash: self.password_hash.clone(),
        }
    }

    pub async fn get(db: &PgPool, email: &str) -> Result<Option<Self>, ApiError> {
        // CASE WHEN (SELECT COUNT(*) FROM two_fa_backup WHERE registered_user_id = ru.registered_user_id) > 0 THEN true ELSE false END AS two_fa_backup,
        let query = r#"
SELECT
	tfs.two_fa_secret,
	ru.registered_user_id, ru.active, ru.email, ru.password_hash, ru.full_name,
	COALESCE(tfs.always_required, false) AS two_fa_always_required,
	COALESCE(au.admin, false) as admin,
	COALESCE(la.login_attempt_number, 0) AS login_attempt_number,
	(
		SELECT
			COALESCE(COUNT(*),0)
		FROM
			two_fa_backup
		WHERE
			registered_user_id = ru.registered_user_id
	) AS two_fa_backup_count
FROM
	registered_user ru
LEFT JOIN
	two_fa_secret tfs
ON
	ru.registered_user_id = tfs.registered_user_id
LEFT JOIN
	login_attempt la
ON
	ru.registered_user_id = la.registered_user_id
LEFT JOIN
	admin_user au
ON
	ru.registered_user_id = au.registered_user_id
WHERE
	ru.email = $1 AND active = true"#;
        Ok(sqlx::query_as::<_, Self>(query)
            .bind(email.to_lowercase())
            .fetch_optional(db)
            .await?)
    }

    pub async fn insert(db: &PgPool, user: &RedisNewUser) -> Result<(), ApiError> {
        let query = r"
		INSERT INTO
			registered_user(full_name, email, password_hash, ip_id, user_agent_id, active)
	 	VALUES
			($1, $2, $3, $4, $5, TRUE)";
        sqlx::query(query)
            .bind(&user.full_name)
            .bind(&user.email)
            .bind(&user.password_hash)
            .bind(user.ip_id)
            .bind(user.user_agent_id)
            .execute(db)
            .await?;
        Ok(())
    }

    // Ideally shoudl use self here!
    // &self,
    pub async fn update_password(
        db: &PgPool,
        registered_user_id: i64,
        password_hash: ArgonHash,
    ) -> Result<(), ApiError> {
        let query = "UPDATE registered_user SET password_hash = $1 WHERE registered_user_id = $2";
        sqlx::query(query)
            .bind(password_hash.to_string())
            .bind(registered_user_id)
            .execute(db)
            .await?;
        Ok(())
    }
}

#[async_trait]
impl<B> FromRequest<B> for ModelUser
where
    B: Send,
{
    type Rejection = ApiError;

    /// Check client is authenticated, and then return model_user object
    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let state = get_state(req.extensions())?;
        if let Ok(jar) = req.extract::<PrivateCookieJar<Key>>().await {
            if let Some(data) = jar.get(&state.cookie_name) {
                let uuid = Uuid::parse_str(data.value())?;
                if let Some(user) = RedisSession::get(&state.redis, &state.postgres, &uuid).await? {
                    return Ok(user);
                }
            }
        }
        Err(ApiError::Authentication)
    }
}

/// cargo watch -q -c -w src/ -x 'test db_postgres_model_user -- --test-threads=1 --nocapture'
#[cfg(test)]
#[allow(clippy::pedantic, clippy::nursery, clippy::unwrap_used)]
mod tests {
    use ::redis::aio::Connection;
    use tokio::sync::Mutex;

    use super::*;
    use crate::api::api_tests::{setup, TestSetup, TEST_EMAIL, TEST_PASSWORD};
    use crate::database::{ModelUserAgentIp, RedisNewUser, ReqUserAgentIp};
    use std::sync::Arc;

    async fn gen_new_user(user_ip: &ModelUserAgentIp) -> RedisNewUser {
        let password_hash = ArgonHash::new(TEST_PASSWORD.to_owned())
            .await
            .unwrap()
            .to_string();
        RedisNewUser {
            email: TEST_EMAIL.to_owned(),
            full_name: String::from("test_user"),
            password_hash,
            ip_id: user_ip.ip_id,
            user_agent_id: user_ip.user_agent_id,
        }
    }

    /// insert useragent/ip into postgres & redis
    async fn get_req(
        db: &PgPool,
        redis: &Arc<Mutex<Connection>>,
        req: &ReqUserAgentIp,
    ) -> ModelUserAgentIp {
        ModelUserAgentIp::get(db, redis, req).await.unwrap()
    }

    #[tokio::test]
    /// Insert result Ok
    async fn db_postgres_model_user_insert() {
        let test_setup = setup().await;

        let req = TestSetup::gen_req();
        let user_ip = get_req(&test_setup.postgres, &test_setup.redis, &req).await;
        let new_user = gen_new_user(&user_ip).await;

        let result = ModelUser::insert(&test_setup.postgres, &new_user).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    /// Second insert, with same user + email, returns error
    async fn db_postgres_model_user_insert_twice_error() {
        let test_setup = setup().await;

        let req = TestSetup::gen_req();
        let user_ip = get_req(&test_setup.postgres, &test_setup.redis, &req).await;
        let new_user = gen_new_user(&user_ip).await;

        let result = ModelUser::insert(&test_setup.postgres, &new_user).await;
        assert!(result.is_ok());
        let result = ModelUser::insert(&test_setup.postgres, &new_user).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    /// Get known email Ok(Some(user))
    async fn db_postgres_model_user_get_user_some() {
        let test_setup = setup().await;

        let req = TestSetup::gen_req();
        let user_ip = get_req(&test_setup.postgres, &test_setup.redis, &req).await;
        let new_user = gen_new_user(&user_ip).await;

        ModelUser::insert(&test_setup.postgres, &new_user)
            .await
            .unwrap();

        let result = ModelUser::get(&test_setup.postgres, &new_user.email).await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.email, new_user.email);
        assert_eq!(result.full_name, new_user.full_name);
        assert!(result.active);
    }

    #[tokio::test]
    /// get unkown email Ok(None)
    async fn db_postgres_model_user_get_user_none() {
        let test_setup = setup().await;

        let req = TestSetup::gen_req();
        let user_ip = get_req(&test_setup.postgres, &test_setup.redis, &req).await;
        let new_user = gen_new_user(&user_ip).await;

        let result = ModelUser::get(&test_setup.postgres, &new_user.email).await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.is_none());
    }
}
