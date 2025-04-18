pub mod admin_queries {
    use std::net::IpAddr;

    use fred::{
        clients::Pool,
        interfaces::{KeysInterface, SetsInterface},
    };
    use jiff::ToSpan;
    use serde::Serialize;
    use sqlx::{FromRow, PgPool, Row};

    use crate::{
        S,
        api_error::ApiError,
        database::{ModelUser, redis::RedisKey},
        helpers::now_utc,
        servers::ij::PhotoName,
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
    JOIN ip_address ip USING(ip_id)
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
            Ok(sqlx::query_as!(
                Self,
                r#"SELECT
    tfs.two_fa_secret,
    ru.registered_user_id,
    ru.active,
    ru.email,
    ru.password_hash,
    ru.full_name,
    COALESCE(tfs.always_required, false) AS "two_fa_always_required!",
    COALESCE(au.admin, false) AS "admin!",
    COALESCE(la.login_attempt_number, 0) AS "login_attempt_number!",
    (
        SELECT
            COALESCE(COUNT(*),0)
        FROM
            two_fa_backup
        WHERE
            registered_user_id = ru.registered_user_id
    ) AS "two_fa_backup_count!"
FROM
    registered_user ru
LEFT JOIN two_fa_secret tfs USING(registered_user_id)
LEFT JOIN login_attempt la USING(registered_user_id)
LEFT JOIN admin_user au USING(registered_user_id)
WHERE
    ru.email = $1"#,
                email.to_lowercase()
            )
            .fetch_optional(db)
            .await?)
        }
    }

    pub async fn update_active(
        postgres: &PgPool,
        active: bool,
        registered_user_id: i64,
    ) -> Result<(), ApiError> {
        sqlx::query!(
            "UPDATE registered_user SET active = $1 WHERE registered_user_id = $2",
            active,
            registered_user_id
        )
        .execute(postgres)
        .await?;
        Ok(())
    }

    pub async fn update_login_attempt(
        postgres: &PgPool,
        registered_user_id: i64,
    ) -> Result<(), ApiError> {
        sqlx::query!(
            "UPDATE login_attempt SET login_attempt_number = 0 WHERE registered_user_id = $1",
            registered_user_id
        )
        .execute(postgres)
        .await?;
        Ok(())
    }

    pub async fn consume_password_reset(
        postgres: &PgPool,
        password_reset_id: i64,
    ) -> Result<(), ApiError> {
        sqlx::query!(
            "UPDATE password_reset SET consumed = true WHERE password_reset_id = $1",
            password_reset_id
        )
        .execute(postgres)
        .await?;
        Ok(())
    }

    pub async fn disable_two_fa(
        postgres: &PgPool,
        registered_user_id: i64,
    ) -> Result<(), ApiError> {
        let mut transaction = postgres.begin().await?;
        sqlx::query!(
            "DELETE from two_fa_backup WHERE registered_user_id = $1",
            registered_user_id
        )
        .execute(&mut *transaction)
        .await?;
        sqlx::query!(
            "DELETE from two_fa_secret WHERE registered_user_id = $1",
            registered_user_id
        )
        .execute(&mut *transaction)
        .await?;
        transaction.commit().await?;
        Ok(())
    }

    #[derive(sqlx::FromRow, Serialize, Debug, Clone, PartialEq, Eq)]
    pub struct Session {
        pub user_agent: String,
        pub ip: IpAddr,
        pub login_date: String,
        pub end_date: String,
        pub ulid: String,
        pub current: bool,
    }

    impl Session {
        pub async fn get(
            email: &str,
            redis: &Pool,
            postgres: &PgPool,
            current_session_ulid: Option<String>,
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
                        let ulid = session.split("::").skip(1).take(1).collect::<String>();
                        let current = current_session_ulid.as_ref() == Some(&ulid);
                        let query = "
SELECT
    ua.user_agent_string AS user_agent,
    ip.ip,
    lh.timestamp::TEXT AS login_date,
    session_name AS ulid,
    $1 as end_date,
    $2 as current
FROM
    login_history lh
JOIN user_agent ua USING(user_agent_id)
JOIN ip_address ip USING(ip_id)
WHERE
lh.session_name = $3";
                        output.push(
                            sqlx::query_as::<_, Self>(query)
                                .bind(end_date)
                                .bind(current)
                                .bind(ulid)
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
            Ok(sqlx::query_as!(
                Self,
                "SELECT email FROM registered_user WHERE active = true"
            )
            .fetch_all(postgres)
            .await?
            .into_iter()
            .map(|i| i.email)
            .collect::<Vec<_>>())
        }
    }

    #[derive(Debug, Clone, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
    pub struct ActivePhoto {
        pub meal_photo_id: i64,
        pub photo_original: String,
        pub photo_converted: String,
        pub individual_meal_id: Option<i64>,
        pub person: Option<String>,
        pub meal_date: Option<jiff::civil::Date>,
    }

    impl<'r> FromRow<'r, sqlx::postgres::PgRow> for ActivePhoto {
        fn from_row(row: &'r sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
            Ok(Self {
                meal_photo_id: row.try_get("meal_photo_id")?,
                photo_original: row.try_get("photo_original")?,
                photo_converted: row.try_get("photo_converted")?,
                individual_meal_id: row.try_get("individual_meal_id")?,
                person: row.try_get("person")?,
                meal_date: row
                    .try_get::<Option<jiff_sqlx::Date>, &str>("meal_date")?
                    .map(jiff_sqlx::Date::to_jiff),
            })
        }
    }

    impl ActivePhoto {
        /// Check if a given image name is currently attached to any individual meal
        pub async fn in_use(postgres: &PgPool, photoname: &PhotoName) -> Result<bool, ApiError> {
            Ok(sqlx::query!(
                "SELECT individual_meal_id FROM individual_meal im
                LEFT JOIN meal_photo mp USING (meal_photo_id)
                WHERE mp.photo_converted = $1 OR mp.photo_original = $1",
                photoname.to_string()
            )
            .fetch_optional(postgres)
            .await?
            .is_some())
        }

        pub async fn get_all(postgres: &PgPool) -> Result<Vec<Self>, ApiError> {
            let query = "SELECT
    p.meal_photo_id,
    p.photo_original,
    p.photo_converted,
    im.individual_meal_id,
    mp.person,
    md.date_of_meal AS meal_date
FROM
    meal_photo p
JOIN individual_meal im USING(meal_photo_id)
JOIN
    meal_person mp
ON
    im.meal_person_id = mp.meal_person_id
JOIN
    meal_date md
ON
    im.meal_date_id = md.meal_date_id
ORDER BY md.date_of_meal DESC";
            Ok(sqlx::query_as::<_, Self>(query).fetch_all(postgres).await?)
        }
    }
}
