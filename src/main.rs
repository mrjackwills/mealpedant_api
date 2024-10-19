#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

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
use tracing_subscriber::{fmt, prelude::__tracing_subscriber_SubscriberExt};

/// Simple macro to create a new String, or convert from a &str to a String - basically just gets rid of String::from() / .to_owned() etc
#[macro_export]
macro_rules! S {
    () => {
        String::new()
    };
    ($s:expr) => {
        String::from($s)
    };
}

/// Simple macro to call `.clone()` on whatever is passed in
#[macro_export]
macro_rules! C {
    ($i:expr) => {
        $i.clone()
    };
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
        Ok(()) => Ok(()),
        Err(e) => {
            println!("{e:?}");
            Err(ApiError::Internal(S!("Unable to start tracing")))
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), ApiError> {
    let app_env = parse_env::AppEnv::get_env();

    if let Err(e) = setup_tracing(&app_env) {
        println!("tracing error: {e}");
        std::process::exit(1);
    }

    tracing::info!(
        "{} - {} - {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        app_env.run_mode
    );
    let postgres = database::db_postgres::db_pool(&app_env).await?;
    let redis = database::DbRedis::get_pool(&app_env).await?;
    BackupSchedule::init(&app_env);
    api::serve(app_env, postgres, redis).await
}
