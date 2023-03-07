#![forbid(unsafe_code)]
#![warn(
    clippy::expect_used,
    clippy::nursery,
    clippy::pedantic,
    clippy::todo,
    clippy::unused_async,
    clippy::unwrap_used
)]
#![allow(clippy::module_name_repetitions, clippy::doc_markdown)]

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
use tracing_subscriber::{fmt, prelude::__tracing_subscriber_SubscriberExt};

fn setup_tracing(app_envs: &AppEnv) -> Result<(), ApiError> {
    let logfile = tracing_appender::rolling::never(&app_envs.location_logs, "api.log");

    let log_fmt = fmt::Layer::default().json().with_writer(logfile);

    match tracing::subscriber::set_global_default(
        fmt::Subscriber::builder()
            .with_file(true)
            .with_line_number(true)
            .with_max_level(app_envs.log_level)
            .finish()
            .with(log_fmt),
    ) {
        Ok(_) => Ok(()),
        Err(e) => {
            println!("{e:?}");
            Err(ApiError::Internal("Unable to start tracing".to_owned()))
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), ApiError> {
    let app_env = parse_env::AppEnv::get_env();
    setup_tracing(&app_env)?;
    let postgres = database::db_postgres::db_pool(&app_env).await?;
    let redis = database::DbRedis::get_connection(&app_env).await?;
    BackupSchedule::init(&app_env).await;
    api::serve(app_env, postgres, Arc::new(Mutex::new(redis))).await?;
    Ok(())
}
