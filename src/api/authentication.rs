use axum::{extract::State, http::Request, middleware::Next, response::Response};
use axum_extra::extract::PrivateCookieJar;

use google_authenticator::GoogleAuthenticator;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    api_error::ApiError,
    argon::verify_password,
    database::{ModelTwoFABackup, ModelUser, RedisSession},
};

use super::{incoming_json::ij::Token, ApplicationState};

/// Validate an 2fa token
// pub async fn authenticate_token(
//     token: Option<Token>,
//     postgres: &PgPool,
//     two_fa_secret: &str,
//     registered_user_id: i64,
//     two_fa_backup_count: i64,
// ) -> Result<bool, ApiError> {
//     if let Some(token) = token {
//         let auth = GoogleAuthenticator::new();
//         match token {
//             Token::Totp(token_text) => {
//                 return Ok(auth.verify_code(two_fa_secret, &token_text, 0, 0))
//             }
//             Token::Backup(token_text) => {
//                 if two_fa_backup_count > 0 {
//                     let backups = ModelTwoFABackup::get(postgres, registered_user_id).await?;

//                     let mut backup_token_id = None;
//                     for backup_code in backups {
//                         if verify_password(&token_text, backup_code.as_hash()).await? {
//                             backup_token_id = Some(backup_code.two_fa_backup_id);
//                         }
//                     }
//                     // Delete backup code if it's valid
//                     if let Some(id) = backup_token_id {
//                         ModelTwoFABackup::delete_one(postgres, id).await?;
//                     } else {
//                         return Ok(false);
//                     }
//                 }
//             }
//         };
//     }
//     Ok(true)
// }
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
                // SHOULD USE A TRANSACTION!?
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
    }else{
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
pub async fn not_authenticated<B: Send + Sync>(
    State(state): State<ApplicationState>,
    jar: PrivateCookieJar,
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, ApiError> {
    if let Some(data) = jar.get(&state.cookie_name) {
        if RedisSession::exists(&state.redis, &Uuid::parse_str(data.value())?)
            .await?
            .is_some()
        {
            return Err(ApiError::Authentication);
        }
    }
    Ok(next.run(req).await)
}

/// Only allow a request if the client is authenticated
pub async fn is_authenticated<B: std::marker::Send>(
    State(state): State<ApplicationState>,
    jar: PrivateCookieJar,
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, ApiError> {
    if let Some(data) = jar.get(&state.cookie_name) {
        if RedisSession::exists(&state.redis, &Uuid::parse_str(data.value())?)
            .await?
            .is_some()
        {
            return Ok(next.run(req).await);
        }
    }
    Err(ApiError::Authentication)
}

/// Only allow a request if the client is admin
pub async fn is_admin<B: Send + Sync>(
    State(state): State<ApplicationState>,
    jar: PrivateCookieJar,
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, ApiError> {
    if let Some(data) = jar.get(&state.cookie_name) {
        if let Some(session) = RedisSession::get(
            &state.redis,
            &state.postgres,
            &Uuid::parse_str(data.value())?,
        )
        .await?
        {
            if session.admin {
                return Ok(next.run(req).await);
            }
        }
    }
    Err(ApiError::Authentication)
}
