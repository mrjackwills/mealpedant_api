#![forbid(unsafe_code)]
#![warn(clippy::unused_async, clippy::unwrap_used, clippy::expect_used)]
// Wanring - These are indeed pedantic
// #![warn(clippy::pedantic)]
// #![warn(clippy::nursery)]
// #![allow(clippy::module_name_repetitions, clippy::doc_markdown)]

// Only allow when debugging
// #![allow(unused)]


mod api;
mod api_error;
mod argon;
mod database;
mod emailer;
mod helpers;
mod parse_env;
mod photo_convertor;
mod scheduler;

use api_error::ApiError;
use parse_env::AppEnv;
use scheduler::BackupSchedule;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::Level;
use tracing_subscriber::fmt::writer::MakeWriterExt;

fn setup_tracing(app_envs: &AppEnv) {
    let level = if app_envs.log_trace {
        Level::TRACE
    } else if app_envs.log_debug {
        Level::DEBUG
    } else {
        Level::INFO
    };
    let logfile = tracing_appender::rolling::never(&app_envs.location_logs, "api.log");
    let stdout = std::io::stdout.with_max_level(level);

    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_file(true)
        .with_line_number(true)
        .with_writer(logfile.and(stdout))
        .init();
}

#[tokio::main]
async fn main() -> Result<(), ApiError> {
    let app_env = parse_env::AppEnv::get_env();
    setup_tracing(&app_env);
    let postgres = database::db_postgres::db_pool(&app_env).await?;
    let redis = database::DbRedis::get_connection(&app_env).await?;
    BackupSchedule::init(&app_env).await;
    api::serve(app_env, postgres, Arc::new(Mutex::new(redis))).await?;
    Ok(())
}
