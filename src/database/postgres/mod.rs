mod admin;
mod model_banned_email;
mod model_food;
mod model_ip_user_agent;
mod model_login;
mod model_meal;
mod model_reset_password;
mod model_twofa;
mod model_user;

use std::fmt;

pub use admin::admin_queries;
pub use model_banned_email::ModelBannedEmail;
pub use model_food::{
    IndividualFoodJson, MissingFoodJson, ModelFoodCategory, ModelFoodLastId, ModelIndividualFood,
    ModelMissingFood,
};
pub use model_ip_user_agent::ModelUserAgentIp;
#[cfg(test)]
pub use model_ip_user_agent::ReqUserAgentIp;
pub use model_login::ModelLogin;
pub use model_meal::ModelMeal;
pub use model_reset_password::ModelPasswordReset;
pub use model_twofa::{ModelTwoFA, ModelTwoFABackup};
pub use model_user::ModelUser;
use serde::{Deserialize, Serialize};

use crate::api_error::ApiError;

// generic From Model<T> for X to Item, for Item is *usually* X
pub trait FromModel<T> {
    type Item;
    fn from_model(t: T) -> Result<Self::Item, ApiError>;
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum Person {
    Dave,
    Jack,
}

impl fmt::Display for Person {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let disp = match self {
            Self::Dave => "Dave",
            Self::Jack => "Jack",
        };
        write!(f, "{disp}")
    }
}

impl TryFrom<&str> for Person {
    type Error = ApiError;
    fn try_from(x: &str) -> Result<Self, ApiError> {
        match x {
            "Dave" => Ok(Self::Dave),
            "Jack" => Ok(Self::Jack),
            _ => Err(ApiError::Internal("from person".to_owned())),
        }
    }
}

pub mod db_postgres {

    use crate::{api_error::ApiError, parse_env::AppEnv};
    use sqlx::{postgres::PgPoolOptions, PgPool};

    pub async fn db_pool(app_env: &AppEnv) -> Result<PgPool, ApiError> {
        let options = sqlx::postgres::PgConnectOptions::new_without_pgpass()
            .host(&app_env.pg_host)
            .port(app_env.pg_port)
            .database(&app_env.pg_database)
            .username(&app_env.pg_user)
            .password(&app_env.pg_pass);

        // if app_env.log.is_none() {
        //     options.disable_statement_logging();
        // }

        let acquire_timeout = std::time::Duration::from_secs(5);
        let idle_timeout = std::time::Duration::from_secs(30);

        Ok(PgPoolOptions::new()
            .max_connections(20)
            .idle_timeout(idle_timeout)
            .acquire_timeout(acquire_timeout)
            .connect_with(options)
            .await?)
    }
}

/// cargo watch -q -c -w src/ -x 'test db_postgres_mod -- --test-threads=1 --nocapture'
#[cfg(test)]
#[expect(clippy::pedantic, clippy::unwrap_used)]
mod tests {
    use crate::parse_env;

    use super::*;

    #[tokio::test]
    async fn db_postgres_mod_get_connection() {
        let app_env = parse_env::AppEnv::get_env();

        let result = db_postgres::db_pool(&app_env).await;
        assert!(result.is_ok());

        #[derive(sqlx::FromRow)]
        struct DB {
            current_database: String,
        }

        let result = sqlx::query_as::<_, DB>("SELECT current_database()")
            .fetch_one(&result.unwrap())
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().current_database, "mealpedant");
    }
}
