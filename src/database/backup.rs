use std::{fmt, fs::Permissions, os::unix::fs::PermissionsExt, path::PathBuf, process::ExitStatus};

use time::OffsetDateTime;
use tokio::io::AsyncWriteExt;

use crate::{C, S, api_error::ApiError, helpers::gen_random_hex, parse_env::AppEnv};

#[derive(Debug, Clone)]
pub struct BackupEnv {
    pub location_backup: String,
    pub location_logs: String,
    backup_age: String,
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
            backup_age: C!(app_env.backup_age),
            location_backup: C!(app_env.location_backup),
            location_logs: C!(app_env.location_logs),
            location_redis: C!(app_env.location_redis),
            location_static: C!(app_env.location_static),
            location_temp: C!(app_env.location_temp),
            pg_database: C!(app_env.pg_database),
            pg_host: C!(app_env.pg_host),
            pg_password: C!(app_env.pg_pass),
            pg_port: app_env.pg_port,
            pg_user: C!(app_env.pg_user),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
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
        write!(f, "{disp}")
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
        format!("mealpedant_{date}_{time}_{self}_{suffix}.tar.age")
    }
}
enum Programs {
    Age,
    Find,
    Gzip,
    PgDump,
    Tar,
}

impl fmt::Display for Programs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let disp = match self {
            Self::Age => "age",
            Self::Find => "find",
            Self::Gzip => "gzip",
            Self::PgDump => "pg_dump",
            Self::Tar => "tar",
        };
        write!(f, "{disp}")
    }
}

/// write to ~/.pgpass
/// set chmod 600
async fn write_pgpass(backup_env: &BackupEnv) -> Result<PathBuf, ApiError> {
    let Some(file_path) = directories::BaseDirs::new() else {
        return Err(ApiError::Internal(S!("home_dir")));
    };
    let file_path = file_path.home_dir().join(".pgpass");

    let mut file = tokio::fs::File::create_new(&file_path).await?;
    file.write_all(
        format!(
            "{}:{}:{}:{}:{}",
            backup_env.pg_host,
            backup_env.pg_port,
            backup_env.pg_database,
            backup_env.pg_user,
            backup_env.pg_password
        )
        .as_bytes(),
    )
    .await?;
    file.flush().await?;
    file.set_permissions(Permissions::from_mode(0o600)).await?;
    Ok(file_path)
}

/// Delete the .pgpass
async fn delete_pgpass(file_path: PathBuf) -> Result<(), ApiError> {
    tokio::fs::remove_file(file_path).await?;
    Ok(())
}

/// Use pg_dump to create a .tar backup of the database, then gzip result
async fn pg_dump(backup_env: &BackupEnv, temp_dir: &str) -> Result<ExitStatus, ApiError> {
    let pg_dump_tar = format!("{temp_dir}/pg_dump.tar");
    let pg_dump_args = [
        "-U",
        &backup_env.pg_user,
        "-p",
        &backup_env.pg_port.to_string(),
        "-d",
        &backup_env.pg_database,
        "-h",
        &backup_env.pg_host,
        "--no-owner",
        "-F",
        "t",
        "-f",
        &pg_dump_tar,
    ];
    let pg_pass_file_path = write_pgpass(backup_env).await?;
    let dump = tokio::process::Command::new(Programs::PgDump.to_string())
        .args(pg_dump_args)
        .spawn()?
        .wait()
        .await?;
    delete_pgpass(pg_pass_file_path).await?;

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

/// Use age to encrypt a tar, or tar.gz, file
async fn encrypt_backup(
    backup_env: &BackupEnv,
    final_backup_location: &str,
    combined: &str,
) -> Result<(), ApiError> {
    let age_args = [
        "-r",
        &backup_env.backup_age,
        "-o",
        final_backup_location,
        combined,
    ];

    tokio::process::Command::new(Programs::Age.to_string())
        .args(age_args)
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
        "*.age",
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
    let redis_temp_tar = format!("{temp_dir}/redis.tar");
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
    let log_temp_tar = format!("{temp_dir}/logs.tar");
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
    let static_temp_tar = format!("{temp_dir}/static.tar");
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
    let combined_tar = format!("{temp_dir}/combined.tar");

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
        args.push("static.tar");
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

    let final_backup_location = format!("{}/{final_output_name}", backup_env.location_backup);

    let temp_dir = format!("{}/{}", backup_env.location_temp, gen_random_hex(8));

    tokio::fs::create_dir(&temp_dir).await?;

    let combined = format!("{temp_dir}/combined.tar");

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
#[expect(clippy::pedantic, clippy::unwrap_used)]
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
            assert!(i.as_ref().unwrap().metadata().unwrap().len() > 800_000);
            assert!(i.unwrap().metadata().unwrap().len() < 5_000_000);
        }
    }

    #[tokio::test]
    async fn backup_full() {
        let setup = setup().await;
        let backup_env = BackupEnv::new(&setup.app_env);

        let result = create_backup(&backup_env, BackupType::Full).await;

        assert!(result.is_ok());

        // Assert that only single backup created
        let number_backups = std::fs::read_dir(&setup.app_env.location_backup)
            .unwrap()
            .count();
        assert_eq!(number_backups, 1);

        // Assert is between 400mb and 450mb
        // Need to change these figures as the number of photos grows
        for i in std::fs::read_dir(&setup.app_env.location_backup).unwrap() {
            assert!(i.as_ref().unwrap().metadata().unwrap().len() > 400_000_000);
            assert!(i.unwrap().metadata().unwrap().len() < 450_000_000);
        }
    }
}
