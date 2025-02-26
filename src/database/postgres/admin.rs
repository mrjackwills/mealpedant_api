pub mod admin_queries {
    use std::net::IpAddr;

    use fred::{
        clients::Pool,
        interfaces::{KeysInterface, SetsInterface},
    };
    use jiff::ToSpan;
    use serde::Serialize;
    use sqlx::PgPool;

    use crate::{
        S,
        api_error::ApiError,
        database::{ModelUser, redis::RedisKey},
        helpers::now_utc,
    };

    #[derive(sqlx::FromRow, Serialize, Debug, Clone, PartialEq, Eq)]
    pub struct AllUsers {
        pub full_name: String,
        pub email: String,
        pub active: bool,
        pub timestamp: String,
        pub user_creation_ip: IpAddr,
        pub login_attempt_number: Option<i64>,
        pub password_reset_id: Option<i64>,
        pub reset_string: Option<String>,
        pub password_reset_date: Option<String>,
        pub password_reset_creation_ip: Option<IpAddr>,
        pub password_reset_consumed: Option<bool>,
        pub login_ip: Option<IpAddr>,
        pub login_success: Option<bool>,
        pub login_date: Option<String>,
        pub user_agent_string: Option<String>,
        pub admin: bool,
        pub two_fa_active: bool,
    }

    impl AllUsers {
        pub async fn get(postgres: &PgPool) -> Result<Vec<Self>, ApiError> {
            let query = r#"
SELECT
	ru.full_name,
	ru.email,
	ru.active,
	ru.timestamp :: text,
	CASE
		WHEN la.login_attempt_number IS NULL THEN 0
		ELSE la.login_attempt_number
	END,
	ip.ip AS user_creation_ip,
	pr.password_reset_id,
	pr.reset_string,
	pr.timestamp :: text as "password_reset_date",
	pr.password_reset_creation_ip,
	pr.consumed as "password_reset_consumed",
	lh.login_ip,
	lh.success as "login_success",
	lh.timestamp :: text AS login_date,
	lh.user_agent_string,
	CASE
		WHEN au.admin IS null THEN false
		ELSE CASE
		WHEN au.admin IS true THEN true
		ELSE false
	END
	END AS admin,
	CASE
		WHEN tfa.two_fa_secret IS NOT null THEN true
		ELSE false
		END as "two_fa_active"
FROM
	registered_user ru
	LEFT JOIN ip_address ip USING(ip_id)
	LEFT JOIN login_attempt la USING(registered_user_id)
	LEFT JOIN admin_user au USING(registered_user_id)
	LEFT JOIN two_fa_secret tfa USING(registered_user_id)
	LEFT JOIN (
		SELECT
			pr.registered_user_id,
			pr.password_reset_id,
			pr.timestamp,
			pr.reset_string,
			pr.consumed,
			ip.ip AS password_reset_creation_ip
		FROM
			password_reset pr
			JOIN ip_address ip USING(ip_id)
		WHERE
			NOW () <= pr.timestamp + INTERVAL '1 hour'
			AND pr.consumed = false
	) pr USING(registered_user_id)
	LEFT JOIN LATERAL (
		SELECT
			lh.registered_user_id,
			lh.timestamp,
			lh.login_history_id,
			lh.success,
			ua.user_agent_string,
			ip.ip AS login_ip
		FROM
			login_history lh
			JOIN ip_address ip USING(ip_id)
			JOIN user_agent ua USING(user_agent_id)
		WHERE
			lh.registered_user_id = ru.registered_user_id
		ORDER BY
			timestamp DESC
		limit
			1
	) lh USING(registered_user_id)
ORDER BY
	ru.timestamp
	"#;
            Ok(sqlx::query_as::<_, Self>(query).fetch_all(postgres).await?)
        }
    }

    #[derive(sqlx::FromRow, Debug, Clone, PartialEq, Eq)]
    pub struct User {
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

    impl User {
        pub async fn get(db: &PgPool, email: &str) -> Result<Option<Self>, ApiError> {
            let query = "
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
LEFT JOIN two_fa_secret tfs USING(registered_user_id)
LEFT JOIN login_attempt la USING(registered_user_id)
LEFT JOIN admin_user au USING(registered_user_id)
WHERE
	ru.email = $1";
            Ok(sqlx::query_as::<_, Self>(query)
                .bind(email.to_lowercase())
                .fetch_optional(db)
                .await?)
        }
    }

    pub async fn update_active(
        postgres: &PgPool,
        active: bool,
        registered_user_id: i64,
    ) -> Result<(), ApiError> {
        let query = "UPDATE registered_user SET active = $1 WHERE registered_user_id = $2";
        sqlx::query(query)
            .bind(active)
            .bind(registered_user_id)
            .execute(postgres)
            .await?;
        Ok(())
    }

    pub async fn update_login_attempt(
        postgres: &PgPool,
        registered_user_id: i64,
    ) -> Result<(), ApiError> {
        let query =
            "UPDATE login_attempt SET login_attempt_number = 0 WHERE registered_user_id = $1";
        sqlx::query(query)
            .bind(registered_user_id)
            .execute(postgres)
            .await?;
        Ok(())
    }

    pub async fn consume_password_reset(
        postgres: &PgPool,
        password_reset_id: i64,
    ) -> Result<(), ApiError> {
        let query = "UPDATE password_reset SET consumed = true WHERE password_reset_id = $1";
        sqlx::query(query)
            .bind(password_reset_id)
            .execute(postgres)
            .await?;
        Ok(())
    }

    pub async fn disable_two_fa(
        postgres: &PgPool,
        registered_user_id: i64,
    ) -> Result<(), ApiError> {
        let mut transaction = postgres.begin().await?;
        let query = "DELETE from two_fa_backup WHERE registered_user_id = $1";
        sqlx::query(query)
            .bind(registered_user_id)
            .execute(&mut *transaction)
            .await?;
        let query = "DELETE from two_fa_secret WHERE registered_user_id = $1";
        sqlx::query(query)
            .bind(registered_user_id)
            .execute(&mut *transaction)
            .await?;
        Ok(transaction.commit().await?)
    }

    #[derive(sqlx::FromRow, Serialize, Debug, Clone, PartialEq, Eq)]
    pub struct Session {
        pub user_agent: String,
        pub ip: IpAddr,
        pub login_date: String,
        pub end_date: String,
        pub uuid: String,
        pub current: bool,
    }

    impl Session {
        pub async fn get(
            email: &str,
            redis: &Pool,
            postgres: &PgPool,
            current_session_uuid: Option<String>,
        ) -> Result<Vec<Self>, ApiError> {
            match ModelUser::get(postgres, email).await? {
                Some(user) => {
                    let session_key = RedisKey::SessionSet(user.registered_user_id);
                    let current_sessions: Vec<String> =
                        redis.smembers(session_key.to_string()).await?;
                    let now = now_utc();
                    let mut output = vec![];
                    for session in current_sessions {
                        let ttl: i64 = redis.ttl(&session).await?;
                        let end_date = now.saturating_add(ttl.seconds()).to_string();
                        // OffsetDateTime::from_unix_timestamp(now.unix_timestamp() + ttl)
                        // .unwrap_or(now)
                        // .to_string();
                        let uuid = session.split("::").skip(1).take(1).collect::<String>();

                        let current = current_session_uuid.as_ref() == Some(&uuid);

                        let query = "
SELECT
	ua.user_agent_string AS user_agent,
	ip.ip,
	lh.timestamp::text AS login_date,
	session_name AS uuid,
	$2 as end_date,
	$3 as current
FROM
	login_history lh
JOIN user_agent ua USING(user_agent_id)
JOIN ip_address ip USING(ip_id)
WHERE
lh.session_name = $1";
                        output.push(
                            sqlx::query_as::<_, Self>(query)
                                .bind(uuid)
                                .bind(end_date)
                                .bind(current)
                                .fetch_one(postgres)
                                .await?,
                        );
                    }
                    Ok(output)
                }
                _ => Err(ApiError::InvalidValue(S!("unknown user"))),
            }
        }
    }

    #[derive(sqlx::FromRow, Serialize, Debug, Clone, PartialEq, Eq)]
    pub struct ActiveEmail {
        pub email: String,
    }

    impl ActiveEmail {
        pub async fn get(postgres: &PgPool) -> Result<Vec<String>, ApiError> {
            let query = "SELECT email FROM registered_user WHERE active = true";
            Ok(sqlx::query_as::<_, Self>(query)
                .fetch_all(postgres)
                .await?
                .into_iter()
                .map(|i| i.email)
                .collect::<Vec<_>>())
        }
    }
}
