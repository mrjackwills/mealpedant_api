use axum::{Extension, Router, extract::OriginalUri, middleware};
use std::net::SocketAddr;

use fred::prelude::Pool;
use sqlx::PgPool;
use tower::ServiceBuilder;
mod routers;

use crate::{
    C, S,
    api_error::ApiError,
    parse_env::AppEnv,
    servers::{create_cors_layer, get_addr, rate_limiting, shutdown_signal},
};

use super::ApiState;
pub use super::oj::{AsJsonRes, OutgoingJson};

/// Create a /v[x] prefix for all api routes, where x is the current major version
pub fn get_api_version() -> String {
    format!(
        "/v{}",
        env!("CARGO_PKG_VERSION")
            .split('.')
            .take(1)
            .collect::<String>()
    )
}

/// return a unknown endpoint response
pub async fn fallback(
    OriginalUri(original_uri): OriginalUri,
) -> (axum::http::StatusCode, AsJsonRes<String>) {
    (
        axum::http::StatusCode::NOT_FOUND,
        OutgoingJson::new(format!("unknown endpoint: {original_uri}")),
    )
}

pub trait ApiRouter {
    fn create_router(state: &ApiState) -> Router<ApiState>;
}

/// Serve the application
pub async fn serve(app_env: AppEnv, postgres: PgPool, redis: Pool) -> Result<(), ApiError> {
    let prefix = get_api_version();

    let application_state = ApiState::new(&app_env, postgres, redis);

    let cors_layer = create_cors_layer(&app_env)?;

    let api_routes = Router::new()
        .merge(routers::Admin::create_router(&application_state))
        .merge(routers::Food::create_router(&application_state))
        .merge(routers::Incognito::create_router(&application_state))
        .merge(routers::Meal::create_router(&application_state))
        .merge(routers::Photo::create_router(&application_state))
        .merge(routers::User::create_router(&application_state));

    let app = Router::new()
        .nest(&prefix, api_routes)
        .fallback(fallback)
        .with_state(C!(application_state))
        .layer(
            ServiceBuilder::new()
                .layer(Extension(C!(application_state.cookie_key)))
                .layer(middleware::from_fn_with_state(
                    application_state,
                    rate_limiting,
                ))
                .layer(cors_layer),
        );
    let addr = get_addr(&app_env.api_host, app_env.api_port)?;
    tracing::info!("starting api server @ {addr}{prefix}");

    match axum::serve(
        tokio::net::TcpListener::bind(&addr).await?,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    {
        Ok(()) => Ok(()),
        Err(_) => Err(ApiError::Internal(S!("api_server"))),
    }
}
