use time::OffsetDateTime;
use tracing::error;

use crate::{
    database::backup::{create_backup, BackupEnv, BackupType},
    parse_env::AppEnv,
};

pub struct BackupSchedule {
    backup_env: BackupEnv,
}

#[macro_export]
/// Sleep for a given number of milliseconds, is an async fn.
/// If no parameter supplied, defaults to 1000ms
macro_rules! sleep {
    () => {
        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
    };
    ($ms:expr) => {
        tokio::time::sleep(std::time::Duration::from_millis($ms)).await;
    };
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
        sleep!(u64::from(wait_for) * 1000);
        loop {
            let now = OffsetDateTime::now_utc();
            let current = (now.hour(), now.minute());
            match current {
                (4, 0) => {
                    let backup_env = self.backup_env.clone();
                    tokio::spawn(async move {
                        if create_backup(&backup_env, BackupType::Full).await.is_err() {
                            error!("FULL backup");
                        };
                    });
                }
                (4, 5) => {
                    let backup_env = self.backup_env.clone();
                    tokio::spawn(async move {
                        if create_backup(&backup_env, BackupType::SqlOnly)
                            .await
                            .is_err()
                        {
                            error!("SQL_ONLY backup");
                        };
                    });
                }
                _ => (),
            };
            sleep!(60 * 1000);
        }
    }
}
