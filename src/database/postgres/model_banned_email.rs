use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(sqlx::FromRow, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelBannedEmail {
    pub domain: String,
}

impl ModelBannedEmail {
    /// Check if a given email address' domain is in the table of banned domains
    pub async fn get(db: &PgPool, email: &str) -> Result<Option<Self>, sqlx::Error> {
        let domain = email.split_once('@').unwrap_or_default().1;
        let query = "
SELECT
	*
FROM
	banned_email_domain
WHERE
	domain = $1";
        sqlx::query_as::<_, Self>(query)
            .bind(domain.to_lowercase())
            .fetch_optional(db)
            .await
    }
}

/// cargo watch -q -c -w src/ -x 'test db_postgres_model_banned_email -- --test-threads=1 --nocapture'
#[cfg(test)]
#[expect(clippy::pedantic, clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::api::api_tests::setup;

    #[tokio::test]
    /// Returns None for an allowed email address
    async fn db_postgres_model_banned_email_get_none() {
        let test_setup = setup().await;
        let email = "allowed@gmail.com";

        let result = ModelBannedEmail::get(&test_setup.postgres, email).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    /// Returns Some(domain: str) for a banned email address
    async fn db_postgres_model_banned_email_get_some() {
        let test_setup = setup().await;

        let email = "one@0854445.com";
        let result = ModelBannedEmail::get(&test_setup.postgres, email).await;
        assert!(result.is_ok());
        assert!(result.as_ref().unwrap().is_some());
        assert_eq!(result.unwrap().unwrap().domain, "0854445.com");

        let email = "two@wwwnew.eu";
        let result = ModelBannedEmail::get(&test_setup.postgres, email).await;
        assert!(result.is_ok());
        assert!(result.as_ref().unwrap().is_some());
        assert_eq!(result.unwrap().unwrap().domain, "wwwnew.eu");

        let email = "three@carbonia.de";
        let result = ModelBannedEmail::get(&test_setup.postgres, email).await;
        assert!(result.is_ok());
        assert!(result.as_ref().unwrap().is_some());
        assert_eq!(result.unwrap().unwrap().domain, "carbonia.de");
    }
}
