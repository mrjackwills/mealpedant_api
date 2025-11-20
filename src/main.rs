#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

mod api_error;
mod argon;
mod database;
mod emailer;
mod helpers;
mod macros;
mod parse_env;
mod photo_convertor;
mod scheduler;
mod servers;

use api_error::ApiError;

use fred::prelude::Pool;
use parse_env::AppEnv;
use scheduler::BackupSchedule;
use servers::api;
use sqlx::PgPool;
use tracing_subscriber::{fmt, prelude::__tracing_subscriber_SubscriberExt};

use crate::servers::static_serve::StaticRouter;

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

/// Get postgres & redis connections
async fn get_db(app_env: &AppEnv) -> Result<(PgPool, Pool), ApiError> {
    tokio::try_join!(
        database::db_postgres::db_pool(app_env),
        database::DbRedis::get_pool(app_env)
    )
}

/// Start the backup schedule, the static_server, and the api_server
async fn start(app_env: AppEnv) -> Result<(), ApiError> {
    BackupSchedule::init(&app_env);
    let (api_db, static_db) = tokio::try_join!(get_db(&app_env), get_db(&app_env))?;
    let static_env = C!(app_env);
    tokio::spawn(async move {
        if let Err(e) = StaticRouter::serve(static_env, static_db.0, static_db.1).await {
            tracing::error!("{e}");
        }
    });
    api::serve(app_env, api_db.0, api_db.1).await
}

#[tokio::main]
async fn main() -> Result<(), ()> {
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
    tokio::spawn(start(app_env)).await.ok();
    Ok(())
}
