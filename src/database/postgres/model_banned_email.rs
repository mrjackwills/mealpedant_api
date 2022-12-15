use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(sqlx::FromRow, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
// TODO change this is a new type ModelBannedEmail(pub String)
pub struct ModelBannedEmail {
    pub domain: String,
}

impl ModelBannedEmail {
    /// Check if a given email address' domain is in the table of banned domains
    pub async fn get(db: &PgPool, email: &str) -> Result<Option<Self>, sqlx::Error> {
        let domain = email.split_once('@').unwrap_or(("", "")).1;
        let query = r#"
SELECT
	*
FROM
	banned_email_domain
WHERE
	domain = $1"#;
        sqlx::query_as::<_, Self>(query)
            .bind(domain.to_lowercase())
            .fetch_optional(db)
            .await
    }
}

/// cargo watch -q -c -w src/ -x 'test db_postgres_model_banned_email -- --test-threads=1 --nocapture'
#[cfg(test)]
#[allow(clippy::pedantic, clippy::nursery, clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::api::api_tests::setup;

    #[tokio::test]
    /// Retuns None for an allowed email address
    async fn db_postgres_model_banned_email_get_none() {
        let test_setup = setup().await;
        let email = "allowed@gmail.com";

        let result = ModelBannedEmail::get(&test_setup.postgres, email).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    /// Retuns Some(domain: str) for a banned email address
    async fn db_postgres_model_banned_email_get_some() {
        let test_setup = setup().await;

        let email = "one@monctl.com";
        let result = ModelBannedEmail::get(&test_setup.postgres, email).await;
        assert!(result.is_ok());
        assert!(result.as_ref().unwrap().is_some());
        assert_eq!(result.unwrap().unwrap().domain, "monctl.com");

        let email = "two@zynana.cf";
        let result = ModelBannedEmail::get(&test_setup.postgres, email).await;
        assert!(result.is_ok());
        assert!(result.as_ref().unwrap().is_some());
        assert_eq!(result.unwrap().unwrap().domain, "zynana.cf");

        let email = "three@cyme.ru";
        let result = ModelBannedEmail::get(&test_setup.postgres, email).await;
        assert!(result.is_ok());
        assert!(result.as_ref().unwrap().is_some());
        assert_eq!(result.unwrap().unwrap().domain, "cyme.ru");
    }
}
