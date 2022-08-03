use std::{fmt, process::ExitStatus};

use time::OffsetDateTime;

use crate::{api_error::ApiError, helpers::gen_random_hex, parse_env::AppEnv};

#[derive(Debug, Clone)]
pub struct BackupEnv {
    backup_gpg: String,
    pub location_backup: String,
    pub location_logs: String,
    location_redis: String,
    location_static: String,
    location_temp: String,
    pg_database: String,
    pg_host: String,
    pg_password: String,
    pg_port: u16,
    pg_user: String,
}

impl BackupEnv {
    pub fn new(app_env: &AppEnv) -> Self {
        Self {
            backup_gpg: app_env.backup_gpg.to_owned(),
            location_backup: app_env.location_backup.to_owned(),
            location_logs: app_env.location_logs.to_owned(),
            location_redis: app_env.location_redis.to_owned(),
            location_static: app_env.location_static.to_owned(),
            location_temp: app_env.location_temp.to_owned(),
            pg_database: app_env.pg_database.to_owned(),
            pg_host: app_env.pg_host.to_owned(),
            pg_password: app_env.pg_pass.to_owned(),
            pg_port: app_env.pg_port.to_owned(),
            pg_user: app_env.pg_user.to_owned(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum BackupType {
    SqlOnly,
    Full,
}

impl fmt::Display for BackupType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let disp = match self {
            Self::Full => "LOGS_PHOTOS_REDIS_SQL",
            Self::SqlOnly => "LOGS_REDIS_SQL",
        };
        write!(f, "{}", disp)
    }
}

impl BackupType {
    /// Generate a filename for the backup
    pub fn gen_name(&self) -> String {
        let date = time::OffsetDateTime::now_utc().date().to_string();
        let suffix = gen_random_hex(8);
        let current_time = OffsetDateTime::now_utc().to_hms();
        let time = format!(
            "{:0>2}.{:0>2}.{:0>2}",
            current_time.0, current_time.1, current_time.2
        );
        format!("mealpedant_{}_{}_{}_{}.tar.gpg", date, time, self, suffix)
    }
}
enum Programs {
    Tar,
    PgDump,
    Find,
    Gzip,
    Gpg,
}

impl fmt::Display for Programs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let disp = match self {
            Self::Tar => "tar",
            Self::PgDump => "pg_dump",
            Self::Find => "find",
            Self::Gzip => "gzip",
            Self::Gpg => "gpg",
        };
        write!(f, "{}", disp)
    }
}

/// Use pg_dump to create a .tar backup of the database, then gzip result
async fn pg_dump(backup_env: &BackupEnv, temp_dir: &str) -> Result<ExitStatus, ApiError> {
    let pg_dump_tar = format!("{}/pg_dump.tar", temp_dir);
    let pg_dump_args = [
        "-U",
        &backup_env.pg_user,
        "-p",
        &backup_env.pg_port.to_string(),
        "-d",
        &backup_env.pg_database,
        "-h",
        &backup_env.pg_host,
        // need to include password
        "--no-owner",
        "-F",
        "t",
        "-f",
        &pg_dump_tar,
    ];
    std::env::set_var("PGPASSWORD", &backup_env.pg_password);
    let dump = tokio::process::Command::new(Programs::PgDump.to_string())
        .args(pg_dump_args)
        .spawn()?
        .wait()
        .await?;
    std::env::set_var("PGPASSWORD", "");

    if dump.success() {
        Ok(tokio::process::Command::new(Programs::Gzip.to_string())
            .arg(&pg_dump_tar)
            .spawn()?
            .wait()
            .await?)
    } else {
        Ok(dump)
    }
}

/// Use gpg to encrypt a tar, or tar.gz, file
async fn encrypt_backup(
    backup_env: &BackupEnv,
    final_backup_location: &str,
    combined: &str,
) -> Result<(), ApiError> {
    let gpg_args = [
        "--output",
        final_backup_location,
        "--batch",
        "--passphrase",
        &backup_env.backup_gpg,
        "-c",
        combined,
    ];
    tokio::process::Command::new(Programs::Gpg.to_string())
        .args(gpg_args)
        .spawn()?
        .wait()
        .await?;
    Ok(())
}

/// Delete any backup files that are older than 6 days
async fn delete_old(backup_env: &BackupEnv) -> Result<(), ApiError> {
    let find_args = [
        &backup_env.location_backup,
        "-type",
        "f",
        "-name",
        "*.gpg",
        "-mtime",
        "+6",
        "-delete",
    ];
    tokio::process::Command::new(Programs::Find.to_string())
        .args(find_args)
        .spawn()?
        .wait()
        .await?;
    Ok(())
}

/// tar & gzip the redis.db file
async fn tar_redis(backup_env: &BackupEnv, temp_dir: &str) -> Result<(), ApiError> {
    let redis_temp_tar = format!("{}/redis.tar", temp_dir);
    let args = [
        "-C",
        &backup_env.location_redis,
        "-cf",
        &redis_temp_tar,
        "dump.rdb",
    ];

    let tar = tokio::process::Command::new(Programs::Tar.to_string())
        .args(args)
        .spawn()?
        .wait()
        .await?;

    if tar.success() {
        tokio::process::Command::new(Programs::Gzip.to_string())
            .arg(&redis_temp_tar)
            .spawn()?
            .wait()
            .await?;
    }
    Ok(())
}

/// tar & gzip the api.log file
async fn tar_log(backup_env: &BackupEnv, temp_dir: &str) -> Result<(), ApiError> {
    let log_temp_tar = format!("{}/logs.tar", temp_dir);
    let args = [
        "-C",
        &backup_env.location_logs,
        "-cf",
        &log_temp_tar,
        "api.log",
    ];

    let tar = tokio::process::Command::new(Programs::Tar.to_string())
        .args(args)
        .spawn()?
        .wait()
        .await?;

    if tar.success() {
        tokio::process::Command::new(Programs::Gzip.to_string())
            .arg(&log_temp_tar)
            .spawn()?
            .wait()
            .await?;
    }
    Ok(())
}

/// tar the redis.db file
async fn tar_static(backup_env: &BackupEnv, temp_dir: &str) -> Result<(), ApiError> {
    let static_temp_tar = format!("{}/static.tar", temp_dir);
    let args = [
        "-C",
        &backup_env.location_static,
        "-cf",
        &static_temp_tar,
        "./",
    ];

    tokio::process::Command::new(Programs::Tar.to_string())
        .args(args)
        .spawn()?
        .wait()
        .await?;
    Ok(())
}

/// Combine files into a single tar, if sql_only then also gzip this output
async fn combine_files(temp_dir: &str, backup_type: BackupType) -> Result<(), ApiError> {
    let combined_tar = format!("{}/combined.tar", temp_dir);

    let mut args = vec![
        "-C",
        temp_dir,
        "-cf",
        &combined_tar,
        "redis.tar.gz",
        "pg_dump.tar.gz",
        "logs.tar.gz",
    ];

    if backup_type == BackupType::Full {
        args.push("static.tar")
    }

    tokio::process::Command::new(Programs::Tar.to_string())
        .args(args)
        .spawn()?
        .wait()
        .await?;
    Ok(())
}
// Combine this and full _backup, only difference should be inclusion of photos, and also the gzip, or not, in the case of full backup
// Return name of new backup?
// TODO this is causing memory issues
pub async fn create_backup(
    backup_env: &BackupEnv,
    backup_type: BackupType,
) -> Result<(), ApiError> {
    let final_output_name = backup_type.gen_name();

    let final_backup_location = format!("{}/{}", backup_env.location_backup, final_output_name);

    let temp_dir = format!("{}/{}", backup_env.location_temp, gen_random_hex(8));

    tokio::fs::create_dir(&temp_dir).await?;

    let combined = format!("{}/combined.tar", temp_dir);

    // handle each individually?
    if backup_type == BackupType::Full {
        tar_static(backup_env, &temp_dir).await?;
    }

    tar_redis(backup_env, &temp_dir).await?;
    tar_log(backup_env, &temp_dir).await?;
    pg_dump(backup_env, &temp_dir).await?;

    combine_files(&temp_dir, backup_type).await?;
    encrypt_backup(backup_env, &final_backup_location, &combined).await?;

    // Remove the tmp location
    // Should always do this? Else can clog up /tmp directory
    // think this always gets called anyway, even is exit code is 1
    tokio::fs::remove_dir_all(&temp_dir).await?;

    delete_old(backup_env).await?;
    Ok(())
}

/// cargo watch -q -c -w src/ -x 'test backup -- --test-threads=1 --nocapture'
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::api::api_tests::setup;

    #[tokio::test]
    async fn backup_sql_only() {
        let setup = setup().await;

        let backup_env = BackupEnv::new(&setup.app_env);
        let result = create_backup(&backup_env, BackupType::SqlOnly).await;
        assert!(result.is_ok());

        // Assert that only single backup created
        let number_backups = std::fs::read_dir(&setup.app_env.location_backup)
            .unwrap()
            .count();
        assert_eq!(number_backups, 1);

        // Assert is between 1mb and 5mb in size
        for i in std::fs::read_dir(&setup.app_env.location_backup).unwrap() {
            assert!(i.as_ref().unwrap().metadata().unwrap().len() > 1000000);
            assert!(i.unwrap().metadata().unwrap().len() < 5000000);
        }
    }

    #[tokio::test]
    async fn backup_full() {
        let setup = setup().await;
        let backup_env = BackupEnv::new(&setup.app_env);

        let result = create_backup(&backup_env, BackupType::Full).await;

        assert!(result.is_ok());

        //  Assert that only single backup created
        let number_backups = std::fs::read_dir(&setup.app_env.location_backup)
            .unwrap()
            .count();
        assert_eq!(number_backups, 1);

        // Assert is between 400mb and 450mb
        // Need to change these figures as the number of photos grows
        for i in std::fs::read_dir(&setup.app_env.location_backup).unwrap() {
            assert!(i.as_ref().unwrap().metadata().unwrap().len() > 400000000);
            assert!(i.unwrap().metadata().unwrap().len() < 450000000);
        }
    }
}
