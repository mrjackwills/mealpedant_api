use axum::{extract::RequestParts, http::Request, middleware::Next, response::Response};
use axum_extra::extract::PrivateCookieJar;
use cookie::Key;
use google_authenticator::GoogleAuthenticator;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    api_error::ApiError,
    argon::verify_password,
    database::{ModelTwoFABackup, ModelUser, RedisSession},
};

use super::{get_state, incoming_json::ij::Token};

/// Validate an 2fa token
pub async fn authenticate_token(
    token: Option<Token>,
    postgres: &PgPool,
    two_fa_secret: &str,
    registered_user_id: i64,
    two_fa_backup_count: i64,
) -> Result<bool, ApiError> {
    if let Some(token) = token {
        let auth = GoogleAuthenticator::new();
        match token {
            Token::Totp(token_text) => {
                return Ok(auth.verify_code(two_fa_secret, &token_text, 0, 0))
            }
            Token::Backup(token_text) => {
                if two_fa_backup_count > 0 {
                    let backups = ModelTwoFABackup::get(postgres, registered_user_id).await?;

                    let mut backup_token_id = None;
                    for backup_code in backups {
                        if verify_password(&token_text, backup_code.as_hash()).await? {
                            backup_token_id = Some(backup_code.two_fa_backup_id);
                        }
                    }
                    // Delete backup code if it's valid
                    if let Some(id) = backup_token_id {
                        ModelTwoFABackup::delete_one(postgres, id).await?;
                    } else {
                        return Ok(false);
                    }
                }
            }
        };
    }
    Ok(true)
}

/// Check that a given password, and token, is valid, will check backup tokens as well
/// Split into signin check, and auth check
pub async fn authenticate_signin(
    user: &ModelUser,
    password: &str,
    token: Option<Token>,
    postgres: &PgPool,
) -> Result<bool, ApiError> {
    if !verify_password(password, user.get_password_hash()).await? {
        return Ok(false);
    }

    if let Some(two_fa_secret) = &user.two_fa_secret {
        return authenticate_token(
            token,
            postgres,
            two_fa_secret,
            user.registered_user_id,
            user.two_fa_backup_count,
        )
        .await;
    }
    Ok(true)
}

/// Check that a given password, and token, is valid, will check backup tokens as well
pub async fn authenticate_password_token(
    user: &ModelUser,
    password: &str,
    token: Option<Token>,
    postgres: &PgPool,
) -> Result<bool, ApiError> {
    if !verify_password(password, user.get_password_hash()).await? {
        return Ok(false);
    }

    if let Some(two_fa_secret) = &user.two_fa_secret {
        if token.is_none() && user.two_fa_always_required {
            return Ok(false);
        }

        return authenticate_token(
            token,
            postgres,
            two_fa_secret,
            user.registered_user_id,
            user.two_fa_backup_count,
        )
        .await;
    }
    Ok(true)
}

/// Only allow a request if the client is not authenticated
pub async fn not_authenticated<B: std::marker::Send>(
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, ApiError> {
    let state = get_state(req.extensions())?;
    let mut parts = RequestParts::new(req);
    if let Ok(jar) = parts.extract::<PrivateCookieJar<Key>>().await {
        if let Some(data) = jar.get(&state.cookie_name) {
            if RedisSession::exists(&state.redis, &Uuid::parse_str(data.value())?)
                .await?
                .is_some()
            {
                return Err(ApiError::Authentication);
            }
        }
        return Ok(next.run(parts.try_into_request()?).await);
    }
    Err(ApiError::Authentication)
}

/// Only allow a request if the client is authenticated
pub async fn is_authenticated<B: std::marker::Send>(
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, ApiError> {
    let state = get_state(req.extensions())?;
    let mut parts = RequestParts::new(req);

    if let Ok(jar) = parts.extract::<PrivateCookieJar<Key>>().await {
        if let Some(data) = jar.get(&state.cookie_name) {
            if RedisSession::exists(&state.redis, &Uuid::parse_str(data.value())?)
                .await?
                .is_some()
            {
                return Ok(next.run(parts.try_into_request()?).await);
            }
        }
    }
    Err(ApiError::Authentication)
}

/// Only allow a request if the client is admin
pub async fn is_admin<B: std::marker::Send>(
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, ApiError> {
    let state = get_state(req.extensions())?;
    let mut parts = RequestParts::new(req);

    if let Ok(jar) = parts.extract::<PrivateCookieJar<Key>>().await {
        if let Some(data) = jar.get(&state.cookie_name) {
            if let Some(session) = RedisSession::get(
                &state.redis,
                &state.postgres,
                &Uuid::parse_str(data.value())?,
            )
            .await?
            {
                if session.admin {
                    return Ok(next.run(parts.try_into_request()?).await);
                }
            }
        }
    }
    Err(ApiError::Authentication)
}
