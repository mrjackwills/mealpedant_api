use sqlx::{PgPool, types::time::OffsetDateTime};

use crate::api_error::ApiError;

use super::ModelUserAgentIp;

#[derive(sqlx::FromRow, Debug, Clone, PartialEq, Eq)]
pub struct ModelPasswordReset {
    pub registered_user_id: i64,
    pub email: String,
    pub full_name: String,
    pub password_reset_id: i64,
    pub reset_string: String,
    pub timestamp: OffsetDateTime,
    pub two_fa_backup_count: i64,
    pub two_fa_secret: Option<String>,
}

// TEST ME
impl ModelPasswordReset {
    /// Check if a given email address' domain is in the table of banned domains
    pub async fn insert(
        db: &PgPool,
        registered_user_id: i64,
        secret: &str,
        req: ModelUserAgentIp,
    ) -> Result<(), sqlx::Error> {
        let query = "
INSERT INTO
	password_reset (
		registered_user_id,
		reset_string,
		ip_id,
		user_agent_id
	)
VALUES
	($1, $2, $3, $4)";
        sqlx::query(query)
            .bind(registered_user_id)
            .bind(secret)
            .bind(req.ip_id)
            .bind(req.user_agent_id)
            .execute(db)
            .await?;
        Ok(())
    }

    /// Set the password reset as consumed, so that it can't be used again
    pub async fn consume(db: &PgPool, password_reset_id: i64) -> Result<(), ApiError> {
        let query = "
	UPDATE password_reset SET consumed = 'true' WHERE password_reset_id = $1";
        sqlx::query(query)
            .bind(password_reset_id)
            .execute(db)
            .await?;
        Ok(())
    }

    /// Find a valid password reset by email, for when user is submitting their address to create a new one
    pub async fn get_by_email(db: &PgPool, email: &str) -> Result<Option<Self>, ApiError> {
        let query = r"
SELECT
	ru.registered_user_id,
	ru.email,
	ru.full_name,
	pr.timestamp,
	pr.password_reset_id,
	pr.reset_string,
	tfs.two_fa_secret,
	(
		SELECT
			COALESCE(COUNT(*), 0)
		FROM
			two_fa_backup
		WHERE
			registered_user_id = ru.registered_user_id
	) AS two_fa_backup_count
FROM
	password_reset pr
	LEFT JOIN registered_user ru USING(registered_user_id)
	LEFT JOIN two_fa_secret tfs USING(registered_user_id)
WHERE
	ru.email = $1
	AND pr.timestamp >= NOW () - INTERVAL '1 hour'
	AND pr.consumed IS NOT TRUE";

        Ok(sqlx::query_as::<_, Self>(query)
            .bind(email.to_lowercase())
            .fetch_optional(db)
            .await?)
    }

    /// Find a valid password reset by secret, for when user is attempting to follow the secret sent via email
    pub async fn get_by_secret(db: &PgPool, secret: &str) -> Result<Option<Self>, ApiError> {
        let query = r"
SELECT
	ru.registered_user_id,
	ru.email,
	ru.full_name,
	pr.timestamp,
	pr.password_reset_id,
	pr.reset_string,
	tfs.two_fa_secret,
	(
		SELECT
			COALESCE(COUNT(*), 0)
		FROM
			two_fa_backup
		WHERE
			registered_user_id = ru.registered_user_id
	) AS two_fa_backup_count
FROM
	password_reset pr
	LEFT JOIN registered_user ru USING(registered_user_id)
	LEFT JOIN two_fa_secret tfs USING(registered_user_id)
WHERE
	pr.reset_string = $1
	AND pr.timestamp >= NOW () - INTERVAL '1 hour'
	AND pr.consumed IS NOT TRUE";
        Ok(sqlx::query_as::<_, Self>(query)
            .bind(secret)
            .fetch_optional(db)
            .await?)
    }
}
