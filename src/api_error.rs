use std::{process, time::SystemTimeError};

use fred::prelude::ErrorKind;
use image::ImageError;
use thiserror::Error;

use axum::{
    extract::multipart::MultipartError,
    response::{IntoResponse, Response},
};
use tokio::task::JoinError;
use tracing::error;

use crate::servers::oj::OutgoingJson;

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
    RateLimited(i64),
    #[error("redis error")]
    RedisError(#[from] fred::error::Error),
    #[error("internal error")]
    SerdeJson(#[from] serde_json::Error),
    #[error("not found")]
    SqlxError(#[from] sqlx::Error),
    #[error("thread error")]
    ThreadError(#[from] JoinError),
    #[error("time error")]
    TimeError(#[from] SystemTimeError),
}

/// Return the internal server error, with a basic { response: "$prefix" }
macro_rules! internal {
    ($prefix:expr) => {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            OutgoingJson::new($prefix),
        )
    };
}

#[expect(clippy::cognitive_complexity)]
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

            Self::ImageError(_) | Self::SerdeJson(_) => internal!(prefix),
            Self::Internal(e) => {
                error!(%e);
                internal!(prefix)
            }
            Self::InvalidValue(value) => (
                axum::http::StatusCode::BAD_REQUEST,
                OutgoingJson::new(value),
            ),
            Self::Io(e) => {
                error!(%e);
                internal!(prefix)
            }
            Self::MissingKey(key) => (
                axum::http::StatusCode::BAD_REQUEST,
                OutgoingJson::new(format!("{prefix} {key}")),
            ),
            Self::Multipart(e) => {
                error!(%e);
                internal!(prefix)
            }
            Self::RateLimited(limit) => (
                axum::http::StatusCode::TOO_MANY_REQUESTS,
                OutgoingJson::new(format!("{prefix} {limit} seconds")),
            ),
            Self::RedisError(e) => {
                error!("{e:?}");
                if e.kind() == &ErrorKind::IO {
                    process::exit(1);
                }
                internal!(prefix)
            }
            Self::Reqwest(e) => {
                error!("{:?}", e);
                internal!(prefix)
            }
            Self::SqlxError(e) => {
                error!(%e);
                internal!(prefix)
            }
            Self::ThreadError(e) => {
                error!(%e);
                internal!(prefix)
            }
            Self::TimeError(e) => {
                error!(%e);
                internal!(prefix)
            }
        };
        (status, body).into_response()
    }
}
