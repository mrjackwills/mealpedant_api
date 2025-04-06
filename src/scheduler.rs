use tracing::error;

use crate::{
    C,
    database::backup::{BackupEnv, BackupType, create_backup},
    helpers::now_utc,
    parse_env::AppEnv,
    sleep,
};

pub struct BackupSchedule {
    backup_env: BackupEnv,
}

impl BackupSchedule {
    /// In it's own tokio thread, start a backup schedule loop
    pub fn init(app_env: &AppEnv) {
        let inner = Self {
            backup_env: BackupEnv::new(app_env),
        };
        tokio::spawn(async move { inner.start().await });
    }

    /// the actual loop, check every minute
    async fn start(&self) {
        // Wait until the current time ends in 0 (i.e. exactly on the minute), before starting the loop
        let wait_for = 60 - now_utc().second();
        sleep!(u64::try_from(wait_for).unwrap_or_default() * 1000);
        loop {
            let now = now_utc();
            let current = (now.hour(), now.minute());
            match current {
                (4, 0) => {
                    let backup_env = C!(self.backup_env);
                    tokio::spawn(async move {
                        if create_backup(&backup_env, BackupType::Full).await.is_err() {
                            error!("FULL backup");
                        }
                    });
                }
                (4, 5) => {
                    let backup_env = C!(self.backup_env);
                    tokio::spawn(async move {
                        if create_backup(&backup_env, BackupType::SqlOnly)
                            .await
                            .is_err()
                        {
                            error!("SQL_ONLY backup");
                        }
                    });
                }
                _ => (),
            }
            sleep!(60 * 1000);
        }
    }
}
