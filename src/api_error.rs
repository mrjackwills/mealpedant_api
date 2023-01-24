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
    #[error("Invalid Authentication")]
    Authentication,
    #[error("Invalid email address and/or password and/or token")]
    Authorization,
    #[error("Axum")]
    AxumExtension(#[from] axum::extract::rejection::ExtensionRejection),
    #[error("body too large")]
    BodySize(#[from] axum::extract::rejection::LengthLimitError),
    #[error("conflict")]
    Conflict(String),
    #[error("image error")]
    ImageError(#[from] ImageError),
    #[error("Internal Server Error")]
    Internal(String),
    #[error("invalid")]
    InvalidValue(String),
    #[error("io error")]
    Io(#[from] std::io::Error),
    #[error("missing")]
    MissingKey(String),
    #[error("multipart error")]
    Multipart(#[from] MultipartError),
    #[error("reqwest")]
    Reqwest(#[from] reqwest::Error),
    #[error("rate limited for")]
    RateLimited(usize),
    #[error("redis error")]
    RedisError(#[from] RedisError),
    #[error("internal error")]
    SerdeJson(#[from] serde_json::Error),
    #[error("not found")]
    SqlxError(#[from] sqlx::Error),
    #[error("thread error")]
    ThreadError(#[from] JoinError),
    #[error("uuid error")]
    UUIDError(#[from] uuid::Error),
}

// BodySize(#[from] axum::extract::rejection::LengthLimitError),
#[allow(clippy::cognitive_complexity)]
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let prefix = self.to_string();
        let (status, body) = match self {
            Self::Authorization => (
                axum::http::StatusCode::UNAUTHORIZED,
                OutgoingJson::new(prefix),
            ),
            Self::Authentication => (axum::http::StatusCode::FORBIDDEN, OutgoingJson::new(prefix)),
            Self::AxumExtension(e) => {
                error!(%e);
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    OutgoingJson::new(e.to_string()),
                )
            }
            Self::BodySize(e) => {
                error!("body size: {}", e);
                (
                    axum::http::StatusCode::PAYLOAD_TOO_LARGE,
                    OutgoingJson::new(prefix),
                )
            }
            Self::Conflict(conflict) => (
                axum::http::StatusCode::CONFLICT,
                OutgoingJson::new(conflict),
            ),

            Self::ImageError(_) | Self::SerdeJson(_) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                OutgoingJson::new(prefix),
            ),
            Self::Internal(e) => {
                error!(%e);
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    OutgoingJson::new(e),
                )
            }
            Self::InvalidValue(value) => (
                axum::http::StatusCode::BAD_REQUEST,
                OutgoingJson::new(value),
            ),
            Self::Io(e) => {
                error!(%e);
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    OutgoingJson::new(prefix),
                )
            }
            Self::MissingKey(key) => (
                axum::http::StatusCode::BAD_REQUEST,
                OutgoingJson::new(format!("{prefix} {key}")),
            ),
            Self::Multipart(e) => {
                println!("{:?}", e.to_string());
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    OutgoingJson::new(prefix),
                )
            }
            Self::RateLimited(limit) => (
                axum::http::StatusCode::TOO_MANY_REQUESTS,
                OutgoingJson::new(format!("{prefix} {limit} seconds")),
            ),
            Self::RedisError(e) => {
                error!("{:?}", e);
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    OutgoingJson::new(prefix),
                )
            }
            Self::Reqwest(e) => {
                error!("{:?}", e);
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    OutgoingJson::new(prefix),
                )
            }
            Self::SqlxError(e) => {
                error!("{:?}", e);
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
            Self::UUIDError(e) => {
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
