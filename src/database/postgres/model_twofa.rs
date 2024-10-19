use sqlx::{types::time::OffsetDateTime, PgPool};

use crate::{api_error::ApiError, argon::ArgonHash, database::RedisTwoFASetup, C};

use super::{ModelUser, ModelUserAgentIp};

#[derive(sqlx::FromRow, Debug, Clone, PartialEq, Eq)]
pub struct ModelTwoFA {
    pub two_fa_secret_id: i64,
    pub always_required: bool,
    pub timestamp: OffsetDateTime,
    pub ip_id: i64,
    pub user_agent_id: i64,
    two_fa_secret: String,
}

impl ModelTwoFA {
    pub async fn insert(
        postgres: &PgPool,
        two_fa_setup: RedisTwoFASetup,
        useragent_ip: ModelUserAgentIp,
        user: &ModelUser,
    ) -> Result<(), ApiError> {
        let insert_query = "INSERT INTO two_fa_secret(registered_user_id, ip_id, user_agent_id, two_fa_secret) VALUES($1, $2, $3, $4)";
        sqlx::query(insert_query)
            .bind(user.registered_user_id)
            .bind(useragent_ip.ip_id)
            .bind(useragent_ip.user_agent_id)
            .bind(two_fa_setup.value())
            .execute(postgres)
            .await?;
        Ok(())
    }

    pub async fn update_always_required(
        postgres: &PgPool,
        always_required: bool,
        user: &ModelUser,
    ) -> Result<(), ApiError> {
        let insert_query =
            "UPDATE two_fa_secret SET always_required = $1 WHERE registered_user_id = $2;";
        sqlx::query(insert_query)
            .bind(always_required)
            .bind(user.registered_user_id)
            .execute(postgres)
            .await?;
        Ok(())
    }

    pub async fn delete(postgres: &PgPool, user: &ModelUser) -> Result<(), ApiError> {
        let insert_query = "DELETE FROM two_fa_secret WHERE registered_user_id = $1;";
        sqlx::query(insert_query)
            .bind(user.registered_user_id)
            .execute(postgres)
            .await?;
        Ok(())
    }
}

#[derive(sqlx::FromRow, Debug, Clone, PartialEq, Eq)]
pub struct ModelTwoFABackup {
    pub two_fa_backup_id: i64,
    two_fa_backup_code: String,
}

impl ModelTwoFABackup {
    pub fn as_hash(&self) -> ArgonHash {
        ArgonHash(C!(self.two_fa_backup_code))
    }

    pub async fn get(postgres: &PgPool, registered_user_id: i64) -> Result<Vec<Self>, ApiError> {
        let query = "SELECT two_fa_backup_code, two_fa_backup_id FROM two_fa_backup WHERE registered_user_id = $1";
        Ok(sqlx::query_as::<_, Self>(query)
            .bind(registered_user_id)
            .fetch_all(postgres)
            .await?)
    }

    pub async fn insert(
        postgres: &PgPool,
        user: &ModelUser,
        useragent_ip: &ModelUserAgentIp,
        backup_hashes: Vec<ArgonHash>,
    ) -> Result<(), ApiError> {
        let mut transaction = postgres.begin().await?;

        for hash in backup_hashes {
            let query = "INSERT INTO two_fa_backup(registered_user_id, user_agent_id, ip_id, two_fa_backup_code) VALUES($1, $2, $3, $4)";
            sqlx::query(query)
                .bind(user.registered_user_id)
                .bind(useragent_ip.user_agent_id)
                .bind(useragent_ip.ip_id)
                .bind(hash.to_string())
                .execute(&mut *transaction)
                .await?;
        }
        Ok(transaction.commit().await?)
    }

    pub async fn delete_one(postgres: &PgPool, two_fa_backup_id: i64) -> Result<(), ApiError> {
        let query = "DELETE FROM two_fa_backup WHERE two_fa_backup_id = $1";
        sqlx::query(query)
            .bind(two_fa_backup_id)
            .execute(postgres)
            .await?;
        Ok(())
    }

    pub async fn delete_all(postgres: &PgPool, user: &ModelUser) -> Result<(), ApiError> {
        let query = "DELETE FROM two_fa_backup WHERE registered_user_id = $1";
        sqlx::query(query)
            .bind(user.registered_user_id)
            .execute(postgres)
            .await?;
        Ok(())
    }
}
