// use axum::{extract::{State, FromRequestParts}, http::Request, middleware::Next, response::Response};

use axum::{
    async_trait,
    extract::{FromRef, FromRequest, FromRequestParts, State},
    // headers::Cookie,
    http::request::Parts,
    http::Request,
    middleware::Next,
    response::Response,
    Extension,
    RequestPartsExt,
};
use axum_extra::extract::PrivateCookieJar;
use cookie::Key;

// use axum_extra::extract::PrivateCookieJar;
// use cookie::Key;
// use axum_extra::extract::PrivateCookieJar;
// use cookie::Key;
use google_authenticator::GoogleAuthenticator;
use sqlx::PgPool;
use uuid::Uuid;
// use uuid::Uuid;

use crate::{
    api_error::ApiError,
    argon::verify_password,
    database::{ModelTwoFABackup, ModelUser, RedisSession},
};

use super::{incoming_json::ij::Token, ApplicationState};

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

// ApplicationState: FromRef<S>,
// S: Send + Sync,
/// Only allow a request if the client is not authenticated
pub async fn not_authenticated<B: Send + Sync>(
    State(state): State<ApplicationState>,
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, ApiError> {
    // if let Ok(jar) = PrivateCookieJar::<Key>::from_request_parts(parts, state).await {
    // 	let state = ApplicationState::from_ref(state);
    // 	if let Some(data) = jar.get(&state.cookie_name) {
    // 		let uuid = Uuid::parse_str(data.value())?;
    // 		if let Some(user) = RedisSession::get(&state.redis, &state.postgres, &uuid).await? {
    // 			return Ok(user);
    // 		}
    // 	}
    // }
    // let state = ApplicationState::from_ref(state);
    // let state = get_state(req.extensions())?;
    let (mut parts, mut body) = req.into_parts();
    if let Ok(jar) = PrivateCookieJar::<Key>::from_request_parts(&mut parts, &state).await {
        // let state = ApplicationState::from_ref(state);
        if let Some(data) = jar.get(&state.cookie_name) {
            if RedisSession::exists(&state.redis, &Uuid::parse_str(data.value())?)
                .await?
                .is_some()
            {
                return Err(ApiError::Authentication);
            }
        }

        return Ok(next.run(Request::from_parts(parts, body)).await);
    }
    Err(ApiError::Authentication)
}

/// Only allow a request if the client is authenticated
pub async fn is_authenticated<B: std::marker::Send>(
    State(state): State<ApplicationState>,
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, ApiError> {
    // let state = get_state(req.extensions())?;
    // let mut parts = RequestParts::new(req);

    let (mut parts, mut body) = req.into_parts();
    if let Ok(jar) = PrivateCookieJar::<Key>::from_request_parts(&mut parts, &state).await {
        if let Some(data) = jar.get(&state.cookie_name) {
            if RedisSession::exists(&state.redis, &Uuid::parse_str(data.value())?)
                .await?
                .is_some()
            {
                return Ok(next.run(Request::from_parts(parts, body)).await);
            }
        }
    }

    // if let Ok(jar) = parts.extract::<PrivateCookieJar<Key>>().await {
    //     if let Some(data) = jar.get(&state.cookie_name) {
    //         if RedisSession::exists(&state.redis, &Uuid::parse_str(data.value())?)
    //             .await?
    //             .is_some()
    //         {
    //             return Ok(next.run(parts.try_into_request()?).await);
    //         }
    //     }
    // }
    Err(ApiError::Authentication)
}

/// Limit the users request based on ip address, using redis as mem store
// async fn rate_limiting<B: Send + Sync>(
//     State(state): State<ApplicationState>,
//     req: Request<B>,
//     next: Next<B>,
// ) -> Result<Response, AppError> {
//     let addr: Option<&ConnectInfo<SocketAddr>> = req.extensions().get();
//     let key = RedisKey::RateLimit(get_ip(req.headers(), addr));
//     check_rate_limit(&state.redis, key).await?;
//     Ok(next.run(req).await)
// }

/// Only allow a request if the client is admin
pub async fn is_admin<B: Send + Sync>(
    State(state): State<ApplicationState>,
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, ApiError> {
    let (mut parts, mut body) = req.into_parts();
    if let Ok(jar) = PrivateCookieJar::<Key>::from_request_parts(&mut parts, &state).await {
        if let Some(data) = jar.get(&state.cookie_name) {
            if let Some(session) = RedisSession::get(
                &state.redis,
                &state.postgres,
                &Uuid::parse_str(data.value())?,
            )
            .await?
            {
                if session.admin {
                    return Ok(next.run(Request::from_parts(parts, body)).await);
                }
            }
        }
    }

    // let state = get_state(req.extensions())?;
    // let mut parts = RequestParts::new(req);

    // if let Ok(jar) = parts.extract::<PrivateCookieJar<Key>>().await {
    //     if let Some(data) = jar.get(&state.cookie_name) {
    //         if let Some(session) = RedisSession::get(
    //             &state.redis,
    //             &state.postgres,
    //             &Uuid::parse_str(data.value())?,
    //         )
    //         .await?
    //         {
    //             if session.admin {
    //                 return Ok(next.run(parts.try_into_request()?).await);
    //             }
    //         }
    //     }
    // }
    Err(ApiError::Authentication)
}
