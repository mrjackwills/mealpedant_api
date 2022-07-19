use time::OffsetDateTime;
use tracing::error;

use crate::{
    database::backup::{create_backup, BackupEnv, BackupType},
    parse_env::AppEnv,
};

pub struct BackupSchedule {
    backup_env: BackupEnv,
}

impl BackupSchedule {
    /// In it's own tokio thread, start a backup schedule loop
    pub async fn init(app_env: &AppEnv) {
        let inner = Self {
            backup_env: BackupEnv::new(app_env),
        };
        tokio::spawn(async move { inner.start().await });
    }

    /// the actual loop, check every minute
    async fn start(&self) {
        // Wait until the current time ends in 0 (i.e. exactly on the minute), before starting the loop
        let wait_for = 60 - OffsetDateTime::now_utc().second();
        tokio::time::sleep(std::time::Duration::from_secs(wait_for as u64)).await;
        loop {
            let now = OffsetDateTime::now_utc();
            let current = (now.hour(), now.minute());
            match current {
                (4, 0) => {
                    let backup_env = self.backup_env.to_owned();
                    tokio::spawn(async move {
                        if create_backup(&backup_env, BackupType::Full).await.is_err() {
                            error!("FULL backup")
                        };
                    });
                }
                (4, 5) => {
                    let backup_env = self.backup_env.to_owned();
                    tokio::spawn(async move {
                        if create_backup(&backup_env, BackupType::SqlOnly)
                            .await
                            .is_err()
                        {
                            error!("SQL_ONLY backup")
                        };
                    });
                }
                _ => (),
            };
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
        }
    }
}
