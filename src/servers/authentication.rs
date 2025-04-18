use axum::{extract::State, http::Request, middleware::Next, response::Response};
use axum_extra::extract::PrivateCookieJar;
use totp_rs::{Algorithm, Secret, TOTP};

use sqlx::PgPool;

use crate::{
    S,
    api_error::ApiError,
    argon::verify_password,
    database::{ModelTwoFABackup, ModelUser, RedisSession},
};

use super::{ApiState, get_cookie_ulid, incoming_json::ij::Token};

/// Generate a secret to TOTP from a given secret
pub fn totp_from_secret(secret: &str) -> Result<TOTP, ApiError> {
    if let Ok(secret_as_bytes) = Secret::Raw(secret.as_bytes().to_vec()).to_bytes() {
        if let Ok(totp) = TOTP::new(Algorithm::SHA1, 6, 1, 30, secret_as_bytes) {
            return Ok(totp);
        }
    }
    Err(ApiError::Internal(S!("TOTP ERROR")))
}

// Could make a struct called Authenticated, and then all these are just methods on that struct?

/// Validate an 2fa token
pub async fn authenticate_token(
    token: Option<Token>,
    postgres: &PgPool,
    two_fa_secret: &str,
    registered_user_id: i64,
    two_fa_backup_count: i64,
) -> Result<bool, ApiError> {
    if let Some(token) = token {
        match token {
            Token::Totp(token_text) => {
                let totp = totp_from_secret(two_fa_secret)?;
                return Ok(totp.check_current(&token_text)?);
            }
            Token::Backup(token_text) => {
                if two_fa_backup_count > 0 {
                    let backups = ModelTwoFABackup::get(postgres, registered_user_id).await?;
                    for backup_code in backups {
                        if verify_password(&token_text, backup_code.as_hash()).await? {
                            ModelTwoFABackup::delete_one(postgres, backup_code.two_fa_backup_id)
                                .await?;
                            return Ok(true);
                        }
                    }
                }
            }
        }
    }
    Ok(false)
}

/// Check that a given password, and token, is valid, will check backup tokens as well
/// Split into signin check, and auth check
pub async fn authenticate_signin(
    user: &ModelUser,
    password: &str,
    token: Option<Token>,
    postgres: &PgPool,
) -> Result<bool, ApiError> {
    let valid_password = verify_password(password, user.get_password_hash()).await?;

    if let Some(two_fa_secret) = &user.two_fa_secret {
        let valid_token = authenticate_token(
            token,
            postgres,
            two_fa_secret,
            user.registered_user_id,
            user.two_fa_backup_count,
        )
        .await?;
        Ok(valid_password && valid_token)
    } else {
        Ok(valid_password)
    }
}

/// Check that a given password, and token, is valid, will check backup tokens as well
pub async fn authenticate_password_token(
    user: &ModelUser,
    password: &str,
    token: Option<Token>,
    postgres: &PgPool,
) -> Result<bool, ApiError> {
    let valid_password = verify_password(password, user.get_password_hash()).await?;
    if !valid_password {
        return Ok(false);
    }

    if let Some(two_fa_secret) = &user.two_fa_secret {
        if user.two_fa_always_required {
            if token.is_none() && user.two_fa_always_required {
                return Ok(false);
            }

            let valid_token = authenticate_token(
                token,
                postgres,
                two_fa_secret,
                user.registered_user_id,
                user.two_fa_backup_count,
            )
            .await?;
            return Ok(valid_password && valid_token);
        }
    }
    Ok(valid_password)
}

/// Only allow a request if the client is not authenticated
pub async fn not_authenticated(
    State(state): State<ApiState>,
    jar: PrivateCookieJar,
    req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, ApiError> {
    if let Some(ulid) = get_cookie_ulid(&state, &jar) {
        if RedisSession::exists(&state.redis, &ulid).await?.is_some() {
            return Err(ApiError::Authentication);
        }
    }
    Ok(next.run(req).await)
}

/// Only allow a request if the client is authenticated
pub async fn is_authenticated(
    State(state): State<ApiState>,
    jar: PrivateCookieJar,
    req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, ApiError> {
    if let Some(ulid) = get_cookie_ulid(&state, &jar) {
        if RedisSession::exists(&state.redis, &ulid).await?.is_some() {
            return Ok(next.run(req).await);
        }
    }
    Err(ApiError::Authentication)
}

/// Only allow a request if the client is admin
pub async fn is_admin(
    State(state): State<ApiState>,
    jar: PrivateCookieJar,
    req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, ApiError> {
    if let Some(ulid) = get_cookie_ulid(&state, &jar) {
        if let Some(session) = RedisSession::get(&state.redis, &state.postgres, &ulid).await? {
            if session.admin {
                return Ok(next.run(req).await);
            }
        }
    }
    Err(ApiError::Authentication)
}
