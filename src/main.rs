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

use parse_env::AppEnv;
use scheduler::BackupSchedule;
use servers::{api, static_serve};
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
        Ok(()) => Ok(()),
        Err(e) => {
            println!("{e:?}");
            Err(ApiError::Internal(S!("Unable to start tracing")))
        }
    }
}

async fn spawned_main(app_env: AppEnv) -> Result<(), ApiError> {
    tracing::info!(
        "{} - {} - {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        app_env.run_mode
    );
    let postgres = database::db_postgres::db_pool(&app_env).await?;
    let redis = database::DbRedis::get_pool(&app_env).await?;
    BackupSchedule::init(&app_env);

    let static_data = (C!(app_env), C!(postgres), C!(redis));
    tokio::spawn(async move {
        if let Err(e) =
            static_serve::StaticRouter::serve(static_data.0, static_data.1, static_data.2).await
        {
            tracing::error!("{e}");
        }
    });
    tokio::spawn(api::serve(app_env, postgres, redis))
        .await
        .ok();
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), ()> {
    let app_env = parse_env::AppEnv::get_env();

    if let Err(e) = setup_tracing(&app_env) {
        println!("tracing error: {e}");
        std::process::exit(1);
    }
    tokio::spawn(spawned_main(app_env)).await.ok();
    Ok(())
}
