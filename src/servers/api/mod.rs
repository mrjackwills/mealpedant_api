// use std::net::ToSocketAddrs;

// use axum::{extract::OriginalUri, http::HeaderValue, middleware, Extension, Router};

use axum::{Extension, Router, extract::OriginalUri, http::HeaderValue, middleware};
use std::net::SocketAddr;

use fred::prelude::Pool;
use sqlx::PgPool;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
mod routers;

use crate::{
    C, S,
    api_error::ApiError,
    parse_env::{AppEnv, RunMode},
    servers::{get_addr, rate_limiting, shutdown_signal},
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

/// TODO is there any reason for this?
pub trait ApiRouter {
    fn create_router(state: &ApiState) -> Router<ApiState>;
}

/// Serve the application
pub async fn serve(app_env: AppEnv, postgres: PgPool, redis: Pool) -> Result<(), ApiError> {
    let prefix = get_api_version();

    // Not sure about this, might need to remove the wwww.
    let cors_url = match app_env.run_mode {
        RunMode::Development => S!("http://127.0.0.1:8002"),
        RunMode::Production => format!("https://www.{}", app_env.domain),
    };

    let cors = CorsLayer::new()
        .allow_methods([
            axum::http::Method::DELETE,
            axum::http::Method::GET,
            axum::http::Method::OPTIONS,
            axum::http::Method::PATCH,
            axum::http::Method::POST,
            axum::http::Method::PUT,
        ])
        .allow_credentials(true)
        .allow_headers(vec![
            axum::http::header::ACCEPT,
            axum::http::header::ACCEPT_LANGUAGE,
            axum::http::header::ACCESS_CONTROL_ALLOW_ORIGIN,
            axum::http::header::AUTHORIZATION,
            axum::http::header::CACHE_CONTROL,
            axum::http::header::CONTENT_LANGUAGE,
            axum::http::header::CONTENT_TYPE,
        ])
        .allow_origin(
            cors_url
                .parse::<HeaderValue>()
                .map_err(|i| ApiError::Internal(i.to_string()))?,
        );

    let application_state = ApiState::new(&app_env, postgres, redis);

    let key = C!(application_state.cookie_key);

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
                .layer(cors)
                .layer(Extension(key))
                .layer(middleware::from_fn_with_state(
                    application_state,
                    rate_limiting,
                )),
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
