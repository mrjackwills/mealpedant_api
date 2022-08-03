use anyhow::Result;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::api_error::ApiError;

use super::ModelUserAgentIp;

#[derive(sqlx::FromRow, Debug, Clone, Deserialize, PartialEq)]
pub struct ModelLogin {
    pub login_attempt_id: i64,
    pub login_attempt_number: i64,
}

impl ModelLogin {
    #[cfg(test)]
    pub async fn get(postgres: &PgPool, registered_user_id: i64) -> Result<Option<Self>, ApiError> {
        let query = r#"SELECT * FROM login_attempt WHERE registered_user_id = $1"#;
        Ok(sqlx::query_as::<_, Self>(query)
            .bind(registered_user_id)
            .fetch_optional(postgres)
            .await?)
    }

    async fn reset(postgres: &PgPool, registered_user_id: i64) -> Result<(), ApiError> {
        let query =
            r#"UPDATE login_attempt SET login_attempt_number = 0 WHERE registered_user_id = $1"#;
        sqlx::query(query)
            .bind(registered_user_id)
            .execute(postgres)
            .await?;
        Ok(())
    }

    async fn increase(postgres: &PgPool, registered_user_id: i64) -> Result<(), ApiError> {
        let query = r#"
INSERT INTO
	login_attempt (login_attempt_number, registered_user_id)
VALUES
	(1, $1)
ON CONFLICT
	(registered_user_id)
DO UPDATE
	SET
		login_attempt_number = login_attempt.login_attempt_number +1"#;
        sqlx::query(query)
            .bind(registered_user_id)
            .execute(postgres)
            .await?;
        Ok(())
    }

    pub async fn insert(
        postgres: &PgPool,
        registered_user_id: i64,
        useragent_ip: ModelUserAgentIp,
        success: bool,
        session_uuid: Option<Uuid>,
    ) -> Result<(), ApiError> {
        let query = r#"
INSERT INTO
	login_history(ip_id, success, session_name, user_agent_id, registered_user_id)
VALUES
	($1, $2, $3, $4, $5)
RETURNING login_history_id"#;

        sqlx::query(query)
            .bind(useragent_ip.ip_id)
            .bind(success)
            .bind(session_uuid.map(|uuid| uuid.to_string()))
            .bind(useragent_ip.user_agent_id)
            .bind(registered_user_id)
            .execute(postgres)
            .await?;

        if success {
            Self::reset(postgres, registered_user_id).await?;
        } else {
            Self::increase(postgres, registered_user_id).await?;
        }
        Ok(())
    }
}
