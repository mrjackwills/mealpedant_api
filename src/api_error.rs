use image::ImageError;
use redis::RedisError;
use thiserror::Error;

use axum::{
    extract::multipart::MultipartError,
    response::{IntoResponse, Response},
};
use tokio::task::JoinError;
use tracing::error;

use crate::api::oj::OutgoingJson;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Internal Server Error")]
    Internal(String),
    #[error("not found")]
    SqlxError(#[from] sqlx::Error),
    #[error("redis error")]
    RedisError(#[from] RedisError),
    #[error("internal error")]
    SerdeJson(#[from] serde_json::Error),
    #[error("rate limited for")]
    RateLimited(usize),
    #[error("thread error")]
    ThreadError(#[from] JoinError),
    #[error("io error")]
    Io(#[from] std::io::Error),
    #[error("uuid error")]
    UUIDError(#[from] uuid::Error),
    #[error("multipart error")]
    Multipart(#[from] MultipartError),
    #[error("image error")]
    ImageError(#[from] ImageError),
    #[error("invalid")]
    InvalidValue(String),
    #[error("conflict")]
    Conflict(String),
    #[error("missing")]
    MissingKey(String),
    #[error("Invalid email address and/or password and/or token")]
    Authorization,
    #[error("Invalid Authentication")]
    Authentication,
    #[error("Axum")]
    AxumBody(#[from] axum::extract::rejection::BodyAlreadyExtracted),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let prefix = self.to_string();
        let (status, body) = match self {
            Self::SqlxError(e) => {
                error!("{:?}", e);
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    OutgoingJson::new(prefix),
                )
            }
            Self::RedisError(e) => {
                error!("{:?}", e);
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    OutgoingJson::new(prefix),
                )
            }
            Self::RateLimited(limit) => (
                axum::http::StatusCode::TOO_MANY_REQUESTS,
                OutgoingJson::new(format!("{} {} seconds", prefix, limit)),
            ),
            Self::Authorization => (
                axum::http::StatusCode::UNAUTHORIZED,
                OutgoingJson::new(prefix),
            ),
            Self::Authentication => (axum::http::StatusCode::FORBIDDEN, OutgoingJson::new(prefix)),
            Self::InvalidValue(value) => (
                axum::http::StatusCode::BAD_REQUEST,
                OutgoingJson::new(value),
            ),
            Self::MissingKey(key) => (
                axum::http::StatusCode::BAD_REQUEST,
                OutgoingJson::new(format!("{} {}", prefix, key)),
            ),
            Self::ImageError(_) | Self::Multipart(_) |  Self::SerdeJson(_) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                OutgoingJson::new(prefix),
            ),
            Self::Conflict(conflict) => (
                axum::http::StatusCode::CONFLICT,
                OutgoingJson::new(conflict),
            ),
            Self::Internal(e) => {
                error!(%e);
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    OutgoingJson::new(prefix),
                )
            }
            Self::Io(e) => {
                error!(%e);
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    OutgoingJson::new(prefix),
                )
            }
            Self::AxumBody(e) => {
                error!(%e);
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    OutgoingJson::new(prefix),
                )
            }
            Self::UUIDError(e) => {
                error!(%e);
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    OutgoingJson::new(prefix),
                )
            }
            Self::ThreadError(e) => {
                error!(%e);
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    OutgoingJson::new(prefix),
                )
            }
        };
        (status, body).into_response()
    }
}
