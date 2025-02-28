use axum::{
    Router,
    extract::State,
    response::IntoResponse,
    routing::{delete, get, patch, post},
};
use axum_extra::extract::{PrivateCookieJar, cookie::Cookie};
use futures::{StreamExt, stream::FuturesUnordered};

use std::fmt;

use crate::{
    C, S,
    api::{ApiRouter, ApiState},
    api_error::ApiError,
    argon::ArgonHash,
    database::{
        ModelTwoFA, ModelTwoFABackup, ModelUser, ModelUserAgentIp, RedisSession, RedisTwoFASetup,
    },
    define_routes,
    emailer::{Email, EmailTemplate},
    helpers::{self, gen_random_hex},
    servers::{Outgoing, authentication, get_cookie_uuid, ij, oj},
};

define_routes! {
    UserRoutes,
    "/user",
    Base => "",
    Signout => "/signout",
    Password => "/password",
    SetupTwoFA => "/setup/twofa",
    TwoFA => "/twofa"
}

// This is shared, should put elsewhere?
enum UserResponse {
    UnsafePassword,
    SetupTwoFA,
    TwoFANotEnabled,
}

impl fmt::Display for UserResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let disp = match self {
            Self::UnsafePassword => S!("unsafe password"),
            Self::SetupTwoFA => S!("Two FA setup already started or enabled"),
            Self::TwoFANotEnabled => S!("Two FA not enabled"),
        };
        write!(f, "{disp}")
    }
}

pub struct UserRouter;

impl ApiRouter for UserRouter {
    fn create_router(_state: &ApiState) -> Router<ApiState> {
        Router::new()
            .route(&UserRoutes::Base.addr(), get(Self::user_get))
            .route(&UserRoutes::Signout.addr(), post(Self::signout_post))
            .route(&UserRoutes::Password.addr(), patch(Self::password_patch))
            .route(
                &UserRoutes::SetupTwoFA.addr(),
                delete(Self::setup_two_fa_delete)
                    .get(Self::setup_two_fa_get)
                    .patch(Self::setup_two_fa_patch)
                    .post(Self::setup_two_fa_post),
            )
            .route(
                &UserRoutes::TwoFA.addr(),
                delete(Self::two_fa_delete)
                    .post(Self::two_fa_post)
                    .patch(Self::two_fa_patch)
                    .put(Self::two_fa_put),
            )
    }
}

impl UserRouter {
    /// Return a user object
    #[expect(clippy::unused_async)]
    async fn user_get(user: ModelUser) -> Outgoing<oj::AuthenticatedUser> {
        (
            axum::http::StatusCode::OK,
            oj::OutgoingJson::new(oj::AuthenticatedUser::from(user)),
        )
    }

    /// Sign out user, by removing session from redis
    async fn signout_post(
        jar: PrivateCookieJar,
        State(state): State<ApiState>,
    ) -> Result<impl IntoResponse, ApiError> {
        if let Some(uuid) = get_cookie_uuid(&state, &jar) {
            RedisSession::delete(&state.redis, &uuid).await?;
        }
        Ok((
            axum::http::StatusCode::OK,
            jar.remove(Cookie::from(C!(state.cookie_name))),
        ))
    }

    /// remove token from redis - used in 2fa setup process,
    async fn setup_two_fa_delete(
        user: ModelUser,
        State(state): State<ApiState>,
    ) -> Result<axum::http::StatusCode, ApiError> {
        RedisTwoFASetup::delete(&state.redis, &user).await?;
        Ok(axum::http::StatusCode::OK)
    }

    /// Get a new secret, store in redis until user returns valid token response
    async fn setup_two_fa_get(
        user: ModelUser,
        State(state): State<ApiState>,
    ) -> Result<Outgoing<oj::TwoFASetup>, ApiError> {
        // If setup process has already started, or user has two_fa already enabled, return conflict error
        if RedisTwoFASetup::exists(&state.redis, &user).await? || user.two_fa_secret.is_some() {
            return Err(ApiError::Conflict(UserResponse::SetupTwoFA.to_string()));
        }

        let secret = gen_random_hex(32);
        let totp = authentication::totp_from_secret(&secret)?;

        RedisTwoFASetup::new(&secret)
            .insert(&state.redis, &user)
            .await?;

        Ok((
            axum::http::StatusCode::OK,
            oj::OutgoingJson::new(oj::TwoFASetup {
                secret: totp.get_secret_base32(),
            }),
        ))
    }

    /// Check that incoming token is valid to the redis key, and insert into postgres
    async fn setup_two_fa_post(
        State(state): State<ApiState>,
        user: ModelUser,
        useragent_ip: ModelUserAgentIp,
        ij::IncomingJson(body): ij::IncomingJson<ij::TwoFA>,
    ) -> Result<axum::http::StatusCode, ApiError> {
        let err = || Err(ApiError::InvalidValue(S!("invalid token")));
        if let Some(two_fa_setup) = RedisTwoFASetup::get(&state.redis, &user).await? {
            match body.token {
                ij::Token::Totp(token) => {
                    let known_totp = authentication::totp_from_secret(two_fa_setup.value())?;

                    if let Ok(valid_token) = known_totp.check_current(&token) {
                        if valid_token {
                            RedisTwoFASetup::delete(&state.redis, &user).await?;
                            ModelTwoFA::insert(&state.postgres, two_fa_setup, useragent_ip, &user)
                                .await?;

                            Email::new(
                                &user.full_name,
                                &user.email,
                                EmailTemplate::TwoFAEnabled,
                                &state.email_env,
                            )
                            .send();
                            return Ok(axum::http::StatusCode::OK);
                        }
                    }
                }
                ij::Token::Backup(_) => return err(),
            };
        }
        err()
    }

    /// Enable, or disable, two_fa_always_required
    async fn setup_two_fa_patch(
        State(state): State<ApiState>,
        user: ModelUser,
        ij::IncomingJson(body): ij::IncomingJson<ij::TwoFAAlwaysRequired>,
    ) -> Result<axum::http::StatusCode, ApiError> {
        if user.two_fa_secret.is_none() {
            return Err(ApiError::Conflict(
                UserResponse::TwoFANotEnabled.to_string(),
            ));
        }

        if body.always_required {
            if user.two_fa_always_required {
                return Err(ApiError::Conflict(
                    UserResponse::TwoFANotEnabled.to_string(),
                ));
            }
            ModelTwoFA::update_always_required(&state.postgres, body.always_required, &user)
                .await?;
            return Ok(axum::http::StatusCode::OK);
        } else if !user.two_fa_always_required {
            return Err(ApiError::Conflict(
                UserResponse::TwoFANotEnabled.to_string(),
            ));
        }
        if body.password.is_none() || body.token.is_none() {
            return Err(ApiError::InvalidValue(S!("password or token")));
        }
        if !authentication::authenticate_password_token(
            &user,
            &body.password.unwrap_or_default(),
            body.token,
            &state.postgres,
        )
        .await?
        {
            return Err(ApiError::Authorization);
        }
        ModelTwoFA::update_always_required(&state.postgres, body.always_required, &user).await?;
        Ok(axum::http::StatusCode::OK)
    }

    /// Remove two_fa complete
    /// remove all backups, then secret
    async fn two_fa_delete(
        State(state): State<ApiState>,
        user: ModelUser,
        ij::IncomingJson(body): ij::IncomingJson<ij::PasswordToken>,
    ) -> Result<axum::http::StatusCode, ApiError> {
        if user.two_fa_secret.is_none() {
            return Err(ApiError::Conflict(
                UserResponse::TwoFANotEnabled.to_string(),
            ));
        }

        if !authentication::authenticate_password_token(
            &user,
            &body.password,
            body.token,
            &state.postgres,
        )
        .await?
        {
            return Err(ApiError::Authorization);
        }
        tokio::try_join!(
            ModelTwoFABackup::delete_all(&state.postgres, &user),
            ModelTwoFA::delete(&state.postgres, &user)
        )?;

        Email::new(
            &user.full_name,
            &user.email,
            EmailTemplate::TwoFADisabled,
            &state.email_env,
        )
        .send();

        Ok(axum::http::StatusCode::OK)
    }

    /// Create backup codes, and matching argon hashes
    async fn gen_backup_codes() -> Result<(Vec<String>, Vec<ArgonHash>), ApiError> {
        let backup_count = 10;
        let mut backup_codes = Vec::with_capacity(backup_count);
        let mut vec_futures = FuturesUnordered::new();
        let mut backup_hashes = vec![];

        for _ in 0..backup_count {
            backup_codes.push(gen_random_hex(16));
        }

        for fut in &backup_codes {
            vec_futures.push(ArgonHash::new(C!(fut)));
        }

        while let Some(result) = vec_futures.next().await {
            backup_hashes.push(result?);
        }
        Ok((backup_codes, backup_hashes))
    }

    /// insert two_fa_backup_code
    async fn two_fa_post(
        user: ModelUser,
        useragent_ip: ModelUserAgentIp,
        State(state): State<ApiState>,
    ) -> Result<Outgoing<oj::TwoFaBackup>, ApiError> {
        if user.two_fa_secret.is_none() || user.two_fa_backup_count != 0 {
            return Err(ApiError::Conflict(
                UserResponse::TwoFANotEnabled.to_string(),
            ));
        }

        let (backup, hashes) = Self::gen_backup_codes().await?;
        ModelTwoFABackup::insert(&state.postgres, &user, &useragent_ip, hashes).await?;
        Email::new(
            &user.full_name,
            &user.email,
            EmailTemplate::TwoFABackupEnabled,
            &state.email_env,
        )
        .send();

        Ok((
            axum::http::StatusCode::OK,
            oj::OutgoingJson::new(oj::TwoFaBackup { backups: backup }),
        ))
    }

    /// Delete any crrent abckup codes, and insert 10 new ones
    async fn two_fa_patch(
        user: ModelUser,
        useragent_ip: ModelUserAgentIp,
        State(state): State<ApiState>,
    ) -> Result<Outgoing<oj::TwoFaBackup>, ApiError> {
        if user.two_fa_secret.is_none() {
            return Err(ApiError::Conflict(
                UserResponse::TwoFANotEnabled.to_string(),
            ));
        }

        ModelTwoFABackup::delete_all(&state.postgres, &user).await?;

        let (backups, hashes) = Self::gen_backup_codes().await?;
        ModelTwoFABackup::insert(&state.postgres, &user, &useragent_ip, hashes).await?;

        Email::new(
            &user.full_name,
            &user.email,
            EmailTemplate::TwoFABackupEnabled,
            &state.email_env,
        )
        .send();

        Ok((
            axum::http::StatusCode::OK,
            oj::OutgoingJson::new(oj::TwoFaBackup { backups }),
        ))
    }

    /// Delete all backup codes
    async fn two_fa_put(
        State(state): State<ApiState>,
        user: ModelUser,
        ij::IncomingJson(body): ij::IncomingJson<ij::PasswordToken>,
    ) -> Result<axum::http::StatusCode, ApiError> {
        if !authentication::authenticate_password_token(
            &user,
            &body.password,
            body.token,
            &state.postgres,
        )
        .await?
        {
            return Err(ApiError::Authorization);
        }
        ModelTwoFABackup::delete_all(&state.postgres, &user).await?;

        Email::new(
            &user.full_name,
            &user.email,
            EmailTemplate::TwoFABackupDisabled,
            &state.email_env,
        )
        .send();

        Ok(axum::http::StatusCode::OK)
    }

    /// Update user password
    async fn password_patch(
        user: ModelUser,
        State(state): State<ApiState>,
        ij::IncomingJson(body): ij::IncomingJson<ij::PatchPassword>,
    ) -> Result<axum::http::StatusCode, ApiError> {
        if !authentication::authenticate_password_token(
            &user,
            &body.current_password,
            body.token,
            &state.postgres,
        )
        .await?
        {
            return Err(ApiError::Authorization);
        }

        // Check if password is exposed in HIBP, that new password doesn't contain user email address, that new password doesn't contain old password and also that new password != old_password
        if body.new_password.contains(&body.current_password)
            || body
                .new_password
                .to_lowercase()
                .contains(&user.email.to_lowercase())
            || helpers::pwned_password(&body.new_password).await?
        {
            return Err(ApiError::InvalidValue(
                UserResponse::UnsafePassword.to_string(),
            ));
        }

        let new_password_hash = ArgonHash::new(C!(body.new_password)).await?;
        ModelUser::update_password(&state.postgres, user.registered_user_id, new_password_hash)
            .await?;

        // TODO remove all sessions except current session?

        Email::new(
            &user.full_name,
            &user.email,
            EmailTemplate::PasswordChanged,
            &state.email_env,
        )
        .send();

        Ok(axum::http::StatusCode::OK)
    }
}

/// Use reqwest to test against real server
/// cargo watch -q -c -w src/ -x 'test api_router_user -- --test-threads=1 --nocapture'
#[cfg(test)]
#[expect(clippy::pedantic, clippy::unwrap_used)]
mod tests {

    use super::UserRoutes;
    use crate::database::{ModelTwoFA, ModelUser, RedisTwoFASetup};
    use crate::helpers::gen_random_hex;
    use crate::servers::api_tests::{
        Response, TEST_EMAIL, TEST_PASSWORD, TestSetup, base_url, get_keys, start_both_servers,
    };
    use crate::tmp_file;

    use fred::interfaces::{HashesInterface, KeysInterface, SetsInterface};

    use reqwest::StatusCode;
    use serde::Serialize;
    use std::collections::HashMap;

    #[tokio::test]
    /// Unauthenticated user unable to access /user route
    async fn api_router_user_get_user_unauthenticated() {
        let test_setup = start_both_servers().await;
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            UserRoutes::Base.addr()
        );
        let resp = reqwest::get(url).await.unwrap();

        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
        let result = resp.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");
    }

    #[tokio::test]
    /// Authenticated user gets correct user object
    async fn api_router_user_get_user_authenticated() {
        let mut test_setup = start_both_servers().await;
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            UserRoutes::Base.addr()
        );
        let client = reqwest::Client::new();

        let authed_cookie = test_setup.authed_user_cookie().await;

        let result = client
            .get(url)
            .header("cookie", authed_cookie)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::OK);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result["admin"], false);
        assert_eq!(result["email"], TEST_EMAIL);
        assert_eq!(result["two_fa_active"], false);
        assert_eq!(result["two_fa_always_required"], false);
        assert_eq!(result["two_fa_count"], 0);
    }

    #[tokio::test]
    /// Authenticated, with 2fa enabled, user gets correct user object
    async fn api_router_user_get_user_authenticated_with_two_fa() {
        let mut test_setup = start_both_servers().await;
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            UserRoutes::Base.addr()
        );
        let client = reqwest::Client::new();

        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.insert_two_fa().await;

        let result = client
            .get(url)
            .header("cookie", authed_cookie)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::OK);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result["admin"], false);
        assert_eq!(result["email"], TEST_EMAIL);
        assert_eq!(result["two_fa_active"], true);
        assert_eq!(result["two_fa_always_required"], false);
        assert_eq!(result["two_fa_count"], 0);
    }

    #[tokio::test]
    /// Unuthenticated user signout just returns 200
    async fn api_router_user_get_signout_unauthenticated() {
        let test_setup = start_both_servers().await;
        let client = reqwest::Client::new();
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            UserRoutes::Signout.addr()
        );

        let result = client.post(&url).send().await.unwrap();

        assert_eq!(result.status(), StatusCode::OK);
    }

    #[tokio::test]
    /// Authenticated user signout removes session, next request invalid
    async fn api_router_user_get_signout_authenticated() {
        let mut test_setup = start_both_servers().await;
        let client = reqwest::Client::new();
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            UserRoutes::Signout.addr()
        );

        let authed_cookie = test_setup.authed_user_cookie().await;

        let result = client
            .post(&url)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::OK);

        // assert redis has zero session keys in it
        let session_vec = get_keys(&test_setup.redis, "session::*").await;
        assert_eq!(session_vec.len(), 0);

        let key = format!(
            "session_set::user::{}",
            test_setup.model_user.unwrap().registered_user_id
        );
        let redis_set: Vec<String> = test_setup.redis.smembers(key).await.unwrap();
        assert!(redis_set.is_empty());

        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            UserRoutes::Base.addr()
        );
        let result = client
            .get(url)
            .header("cookie", authed_cookie)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");
    }

    #[tokio::test]
    //
    async fn api_router_user_password_patch_unauthenticated() {
        let test_setup = start_both_servers().await;
        let client = reqwest::Client::new();
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            UserRoutes::Password.addr()
        );
        let body: HashMap<String, String> = HashMap::new();

        let result = client.patch(&url).json(&body).send().await.unwrap();

        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");
    }

    // Could refactor these with a closure?
    #[tokio::test]
    async fn api_router_user_password_patch_authenticated_short_password() {
        let mut test_setup = start_both_servers().await;
        let client = reqwest::Client::new();
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            UserRoutes::Password.addr()
        );

        let authed_cookie = test_setup.authed_user_cookie().await;

        let new_password = gen_random_hex(11);

        let body = HashMap::from([
            ("current_password", TEST_PASSWORD),
            ("new_password", new_password.as_str()),
        ]);

        let result = client
            .patch(url)
            .header("cookie", authed_cookie)
            .json(&body)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::BAD_REQUEST);

        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "password");

        let post_user = ModelUser::get(&test_setup.postgres, TEST_EMAIL)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(
            test_setup.model_user.unwrap().get_password_hash().0,
            post_user.get_password_hash().0
        );
    }

    #[tokio::test]
    async fn api_router_user_password_patch_authenticated_unsafe_password() {
        let mut test_setup = start_both_servers().await;
        let client = reqwest::Client::new();
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            UserRoutes::Password.addr()
        );
        let authed_cookie = test_setup.authed_user_cookie().await;

        let password = format!("new_password{}", TEST_EMAIL.to_uppercase());
        let body = HashMap::from([
            ("current_password", TEST_PASSWORD),
            ("new_password", &password),
        ]);

        let result = client
            .patch(&url)
            .header("cookie", &authed_cookie)
            .json(&body)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::BAD_REQUEST);

        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "unsafe password");
        let post_user = ModelUser::get(&test_setup.postgres, TEST_EMAIL)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(
            test_setup
                .model_user
                .as_ref()
                .unwrap()
                .get_password_hash()
                .0,
            post_user.get_password_hash().0
        );

        let body = HashMap::from([
            ("current_password", TEST_PASSWORD),
            ("new_password", TEST_PASSWORD),
        ]);

        let result = client
            .patch(&url)
            .header("cookie", &authed_cookie)
            .json(&body)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::BAD_REQUEST);

        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "unsafe password");
        let post_user = ModelUser::get(&test_setup.postgres, TEST_EMAIL)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(
            test_setup
                .model_user
                .as_ref()
                .unwrap()
                .get_password_hash()
                .0,
            post_user.get_password_hash().0
        );

        let body = HashMap::from([
            ("current_password", TEST_PASSWORD),
            ("new_password", "iloveyou1234"),
        ]);

        let result = client
            .patch(url)
            .header("cookie", authed_cookie)
            .json(&body)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::BAD_REQUEST);

        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "unsafe password");
        let post_user = ModelUser::get(&test_setup.postgres, TEST_EMAIL)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(
            test_setup
                .model_user
                .as_ref()
                .unwrap()
                .get_password_hash()
                .0,
            post_user.get_password_hash().0
        );
    }

    #[tokio::test]
    async fn api_router_user_password_patch_authenticated_invalid_current_password() {
        let mut test_setup = start_both_servers().await;
        let client = reqwest::Client::new();
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            UserRoutes::Password.addr()
        );

        let authed_cookie = test_setup.authed_user_cookie().await;

        let current_password = gen_random_hex(64);
        let new_password = gen_random_hex(64);

        let body = HashMap::from([
            ("current_password", &current_password),
            ("new_password", &new_password),
        ]);

        let result = client
            .patch(url)
            .header("cookie", authed_cookie)
            .json(&body)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::UNAUTHORIZED);

        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid email address and/or password and/or token");

        let post_user = ModelUser::get(&test_setup.postgres, TEST_EMAIL)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(
            test_setup
                .model_user
                .as_ref()
                .unwrap()
                .get_password_hash()
                .0,
            post_user.get_password_hash().0
        );
    }

    #[tokio::test]
    async fn api_router_user_password_patch_authenticated_invalid_token() {
        let mut test_setup = start_both_servers().await;
        let client = reqwest::Client::new();
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            UserRoutes::Password.addr()
        );

        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.insert_two_fa().await;
        test_setup.two_fa_always_required(true).await;
        let invalid_token = test_setup.get_invalid_token();

        let new_password = gen_random_hex(64);

        let body = HashMap::from([
            ("current_password", TEST_PASSWORD),
            ("new_password", &new_password),
            ("token", &invalid_token),
        ]);
        let result = client
            .patch(url)
            .header("cookie", authed_cookie)
            .json(&body)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::UNAUTHORIZED);

        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid email address and/or password and/or token");

        let post_user = ModelUser::get(&test_setup.postgres, TEST_EMAIL)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(
            test_setup
                .model_user
                .as_ref()
                .unwrap()
                .get_password_hash()
                .0,
            post_user.get_password_hash().0
        );
    }

    #[tokio::test]
    async fn api_router_user_password_patch_authenticated_password_match() {
        let mut test_setup = start_both_servers().await;
        let client = reqwest::Client::new();
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            UserRoutes::Password.addr()
        );

        let authed_cookie = test_setup.authed_user_cookie().await;

        let body = HashMap::from([
            ("current_password", TEST_PASSWORD),
            ("new_password", TEST_PASSWORD),
        ]);

        let result = client
            .patch(url)
            .header("cookie", authed_cookie)
            .json(&body)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::BAD_REQUEST);

        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "unsafe password");

        let post_user = ModelUser::get(&test_setup.postgres, TEST_EMAIL)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(
            test_setup
                .model_user
                .as_ref()
                .unwrap()
                .get_password_hash()
                .0,
            post_user.get_password_hash().0
        );
    }

    #[tokio::test]
    async fn api_router_user_password_patch_authenticated_password_in_hbip() {
        let mut test_setup = start_both_servers().await;
        let client = reqwest::Client::new();
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            UserRoutes::Password.addr()
        );

        let authed_cookie = test_setup.authed_user_cookie().await;

        let body = HashMap::from([
            ("current_password", TEST_PASSWORD),
            ("new_password", "iloveyou1234"),
        ]);

        let result = client
            .patch(url)
            .header("cookie", authed_cookie)
            .json(&body)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::BAD_REQUEST);

        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "unsafe password");

        let post_user = ModelUser::get(&test_setup.postgres, TEST_EMAIL)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(
            test_setup
                .model_user
                .as_ref()
                .unwrap()
                .get_password_hash()
                .0,
            post_user.get_password_hash().0
        );
    }

    #[tokio::test]
    async fn api_router_user_password_patch_authenticated_valid() {
        let mut test_setup = start_both_servers().await;
        let client = reqwest::Client::new();
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            UserRoutes::Password.addr()
        );

        let authed_cookie = test_setup.authed_user_cookie().await;

        let new_password = gen_random_hex(64);
        let body = HashMap::from([
            ("current_password", TEST_PASSWORD),
            ("new_password", new_password.as_str()),
        ]);

        let result = client
            .patch(url)
            .header("cookie", authed_cookie)
            .json(&body)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::OK);

        let post_user = ModelUser::get(&test_setup.postgres, TEST_EMAIL)
            .await
            .unwrap()
            .unwrap();

        // check that pre_user.0 != post_user.0
        assert_ne!(
            test_setup
                .model_user
                .as_ref()
                .unwrap()
                .get_password_hash()
                .0,
            post_user.get_password_hash().0
        );

        // email sent - written to disk when testing
        assert!(std::fs::exists(tmp_file!("email_headers.txt")).unwrap_or_default());
        assert!(std::fs::exists(tmp_file!("email_body.txt")).unwrap_or_default());
        assert!(
            std::fs::read_to_string(tmp_file!("email_body.txt"))
                .unwrap()
                .contains("The password for your Meal Pedant account has been changed")
        );

        assert!(
            std::fs::read_to_string(tmp_file!("email_headers.txt"))
                .unwrap()
                .contains("Password Changed")
        );

        let signin_url = format!("{}/incognito/signin", base_url(&test_setup.app_env));

        let old_password_body = TestSetup::gen_signin_body(None, None, None, None);

        let result = client
            .post(&signin_url)
            .json(&old_password_body)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            result.json::<Response>().await.unwrap().response,
            "Invalid email address and/or password and/or token"
        );

        let new_password_body = TestSetup::gen_signin_body(None, Some(new_password), None, None);
        let result = client
            .post(&signin_url)
            .json(&new_password_body)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn api_router_user_password_patch_authenticated_valid_with_two_fa() {
        let mut test_setup = start_both_servers().await;
        let client = reqwest::Client::new();
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            UserRoutes::Password.addr()
        );

        let authed_cookie = test_setup.authed_user_cookie().await;

        test_setup.insert_two_fa().await;
        let valid_token = test_setup.get_valid_token();

        let new_password = gen_random_hex(64);
        let body = HashMap::from([
            ("current_password", TEST_PASSWORD),
            ("new_password", new_password.as_str()),
            ("token", &valid_token),
        ]);

        let result = client
            .patch(url)
            .header("cookie", authed_cookie)
            .json(&body)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::OK);

        let post_user = ModelUser::get(&test_setup.postgres, TEST_EMAIL)
            .await
            .unwrap()
            .unwrap();

        // check that pre_user.0 != post_user.0
        assert_ne!(
            test_setup
                .model_user
                .as_ref()
                .unwrap()
                .get_password_hash()
                .0,
            post_user.get_password_hash().0
        );

        // email sent - written to disk when testing
        assert!(std::fs::exists(tmp_file!("email_headers.txt")).unwrap_or_default());
        assert!(std::fs::exists(tmp_file!("email_body.txt")).unwrap_or_default());
        assert!(
            std::fs::read_to_string(tmp_file!("email_body.txt"))
                .unwrap()
                .contains("The password for your Meal Pedant account has been changed")
        );

        assert!(
            std::fs::read_to_string(tmp_file!("email_headers.txt"))
                .unwrap()
                .contains("Password Changed")
        );
    }

    #[tokio::test]
    /// TwoFaSetup route for authed users only
    async fn api_router_user_setup_two_fa_unauthenticated() {
        let test_setup = start_both_servers().await;
        let client = reqwest::Client::new();
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            UserRoutes::SetupTwoFA.addr()
        );

        let result = client.get(&url).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");

        let result = client.delete(&url).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");

        let result = client.patch(&url).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");

        let result = client.post(&url).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");
    }

    #[tokio::test]
    async fn api_router_user_setup_two_fa_get_valid() {
        let mut test_setup = start_both_servers().await;
        let client = reqwest::Client::new();
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            UserRoutes::SetupTwoFA.addr()
        );

        let authed_cookie = test_setup.authed_user_cookie().await;

        let result = client
            .get(url)
            .header("cookie", authed_cookie)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::OK);

        let response = result.json::<Response>().await.unwrap().response;

        let key = format!(
            "two_fa_setup::{}",
            test_setup.model_user.as_ref().unwrap().registered_user_id
        );

        let redis_secret: Option<RedisTwoFASetup> =
            test_setup.redis.hget(&key, "data").await.unwrap();

        assert!(redis_secret.is_some());

        let totp = crate::servers::authentication::totp_from_secret(redis_secret.unwrap().value());
        assert!(totp.is_ok());
        let redis_totp = totp.unwrap().get_secret_base32();

        assert_eq!(redis_totp, response["secret"]);

        let secret_ttl: usize = test_setup.redis.ttl(&key).await.unwrap();

        assert_eq!(secret_ttl, 120);
    }

    #[tokio::test]
    async fn api_router_user_get_two_fa_already_setup() {
        let mut test_setup = start_both_servers().await;
        let client = reqwest::Client::new();
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            UserRoutes::SetupTwoFA.addr()
        );

        let authed_cookie = test_setup.authed_user_cookie().await;

        test_setup.insert_two_fa().await;

        let result = client
            .get(url)
            .header("cookie", authed_cookie)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::CONFLICT);
        assert_eq!(
            "Two FA setup already started or enabled",
            result.json::<Response>().await.unwrap().response
        );
    }

    #[tokio::test]
    async fn api_router_user_setup_two_fa_get_already_in_progress() {
        let mut test_setup = start_both_servers().await;
        let client = reqwest::Client::new();
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            UserRoutes::SetupTwoFA.addr()
        );

        let authed_cookie = test_setup.authed_user_cookie().await;

        client
            .get(&url)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();

        let result = client
            .get(url)
            .header("cookie", authed_cookie)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::CONFLICT);
        assert_eq!(
            "Two FA setup already started or enabled",
            result.json::<Response>().await.unwrap().response
        );
    }

    #[tokio::test]
    async fn api_router_user_delete_setup_two_fa_valid() {
        let mut test_setup = start_both_servers().await;
        let client = reqwest::Client::new();
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            UserRoutes::SetupTwoFA.addr()
        );

        let authed_cookie = test_setup.authed_user_cookie().await;

        client
            .get(&url)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();

        let result = client
            .delete(url)
            .header("cookie", authed_cookie)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::OK);

        // Check key has been removed from redis
        let key = format!(
            "two_fa_setup::{}",
            test_setup.model_user.as_ref().unwrap().registered_user_id
        );

        let redis_secret: Option<String> = test_setup.redis.get(&key).await.unwrap();

        assert!(redis_secret.is_none());
    }

    #[tokio::test]
    async fn api_router_user_post_setup_two_fa_invalid_token() {
        let mut test_setup = start_both_servers().await;
        let client = reqwest::Client::new();
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            UserRoutes::SetupTwoFA.addr()
        );

        let authed_cookie = test_setup.authed_user_cookie().await;

        client
            .get(&url)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();

        let key = format!(
            "two_fa_setup::{}",
            test_setup.model_user.as_ref().unwrap().registered_user_id
        );
        let twofa_setup: RedisTwoFASetup = test_setup.redis.hget(key, "data").await.unwrap();

        let invalid_token = crate::servers::authentication::totp_from_secret(twofa_setup.value())
            .unwrap()
            .generate(123_456_789);

        let body = HashMap::from([("token", &invalid_token)]);

        let result = client
            .post(&url)
            .json(&body)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            "invalid token",
            result.json::<Response>().await.unwrap().response
        );
    }

    #[tokio::test]
    async fn api_router_user_post_setup_two_fa_valid_token() {
        let mut test_setup = start_both_servers().await;
        let client = reqwest::Client::new();
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            UserRoutes::SetupTwoFA.addr()
        );

        let authed_cookie = test_setup.authed_user_cookie().await;

        client
            .get(&url)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();

        let key = format!(
            "two_fa_setup::{}",
            test_setup.model_user.as_ref().unwrap().registered_user_id
        );
        let twofa_setup: RedisTwoFASetup = test_setup.redis.hget(key, "data").await.unwrap();
        let valid_token = crate::servers::authentication::totp_from_secret(twofa_setup.value())
            .unwrap()
            .generate_current()
            .unwrap();

        let body = HashMap::from([("token", &valid_token)]);

        let result = client
            .post(&url)
            .json(&body)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::OK);

        let user = test_setup.get_model_user().await.unwrap();

        // // This will be invalid, as the value sent to the front end is rfc thing
        // assert!(redis_secret.is_some());

        // let totp = totp_from_secret(redis_secret.unwrap().value());
        // assert!(totp.is_ok());
        // let redis_totp = totp.unwrap().get_secret_base32();

        // assert_eq!(redis_totp, response["secret"]);

        assert_eq!(user.two_fa_secret, Some(twofa_setup.value().to_owned()));

        // check email sent - well written to disk
        assert!(std::fs::exists(tmp_file!("email_headers.txt")).unwrap_or_default());
        assert!(std::fs::exists(tmp_file!("email_body.txt")).unwrap_or_default());
        let link = format!(
            "href=\"https://www.{}/user/settings/",
            test_setup.app_env.domain
        );
        assert!(
            std::fs::read_to_string(tmp_file!("email_body.txt"))
                .unwrap()
                .contains(&link)
        );
    }

    #[tokio::test]
    /// Conflict response if two_fa not enabled
    async fn api_router_user_setup_two_patch_two_fa_not_enabled() {
        let mut test_setup = start_both_servers().await;
        let client = reqwest::Client::new();
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            UserRoutes::SetupTwoFA.addr()
        );

        let authed_cookie = test_setup.authed_user_cookie().await;
        let body = HashMap::from([("always_required", true)]);

        let result = client
            .patch(&url)
            .json(&body)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::CONFLICT);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Two FA not enabled");
    }

    #[tokio::test]
    /// Set always_required to true
    async fn api_router_user_setup_two_patch_enabled_valid() {
        let mut test_setup = start_both_servers().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.insert_two_fa().await;

        let client = reqwest::Client::new();
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            UserRoutes::SetupTwoFA.addr()
        );

        let body = HashMap::from([("always_required", true)]);

        let result = client
            .patch(&url)
            .json(&body)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::OK);
        let user = test_setup.get_model_user().await.unwrap();
        assert!(user.two_fa_always_required);
    }

    #[tokio::test]
    /// Conflict response if trying to enabling two_fa_always_required & it is already enabled
    async fn api_router_user_setup_two_patch_enabled_already_enabled() {
        let mut test_setup = start_both_servers().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.insert_two_fa().await;
        let user = test_setup.get_model_user().await.unwrap();
        ModelTwoFA::update_always_required(&test_setup.postgres, true, &user)
            .await
            .unwrap();

        let client = reqwest::Client::new();
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            UserRoutes::SetupTwoFA.addr()
        );

        let body = HashMap::from([("always_required", true)]);

        let result = client
            .patch(&url)
            .json(&body)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::CONFLICT);
    }

    #[derive(Debug, Serialize)]
    struct TestAlwaysRequiredBody {
        always_required: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        password: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        token: Option<String>,
    }

    #[tokio::test]
    /// when trying to disable and no password & token provided error response
    async fn api_router_user_setup_two_patch_disabled_no_password_or_token() {
        let mut test_setup = start_both_servers().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.insert_two_fa().await;
        let user = test_setup.get_model_user().await.unwrap();
        ModelTwoFA::update_always_required(&test_setup.postgres, true, &user)
            .await
            .unwrap();

        let client = reqwest::Client::new();
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            UserRoutes::SetupTwoFA.addr()
        );
        // Missing token
        let body = TestAlwaysRequiredBody {
            always_required: false,
            password: Some(TEST_PASSWORD.to_owned()),
            token: None,
        };

        let result = client
            .patch(&url)
            .json(&body)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::BAD_REQUEST);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "password or token");

        // Missing password
        let body = TestAlwaysRequiredBody {
            always_required: false,
            password: None,
            token: Some(test_setup.get_valid_token()),
        };

        let result = client
            .patch(&url)
            .json(&body)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::BAD_REQUEST);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "password or token");
    }

    #[tokio::test]
    /// Remove two_fa_always required with a valid request which includes password & token
    async fn api_router_user_setup_two_patch_always_required_removed() {
        let mut test_setup = start_both_servers().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.insert_two_fa().await;
        let user = test_setup.get_model_user().await.unwrap();
        ModelTwoFA::update_always_required(&test_setup.postgres, true, &user)
            .await
            .unwrap();

        let client = reqwest::Client::new();
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            UserRoutes::SetupTwoFA.addr()
        );

        let body = TestAlwaysRequiredBody {
            always_required: false,
            password: Some(TEST_PASSWORD.to_owned()),
            token: Some(test_setup.get_valid_token()),
        };

        let result = client
            .patch(&url)
            .json(&body)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::OK);

        let user = test_setup.get_model_user().await.unwrap();
        assert!(!user.two_fa_always_required);
    }

    #[tokio::test]
    /// TwoFa route for authed users only
    async fn api_router_user_two_fa_unauthenticated() {
        let test_setup = start_both_servers().await;
        let client = reqwest::Client::new();
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            UserRoutes::TwoFA.addr()
        );

        let result = client.patch(&url).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");

        let result = client.post(&url).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");

        let result = client.put(&url).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");
    }

    #[tokio::test]
    /// Conflict response if two_fa not enabled
    async fn api_router_user_two_delete_two_fa_not_enabled() {
        let mut test_setup = start_both_servers().await;
        let client = reqwest::Client::new();
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            UserRoutes::TwoFA.addr()
        );

        let authed_cookie = test_setup.authed_user_cookie().await;
        let body = HashMap::from([("password", TEST_PASSWORD), ("token", "012345")]);

        let result = client
            .delete(&url)
            .json(&body)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::CONFLICT);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Two FA not enabled");
    }

    #[tokio::test]
    /// Delete two_fa_secret & all/any backups, also email user
    async fn api_router_user_two_delete_valid() {
        let mut test_setup = start_both_servers().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.insert_two_fa().await;
        let user = test_setup.get_model_user().await.unwrap();
        ModelTwoFA::update_always_required(&test_setup.postgres, true, &user)
            .await
            .unwrap();

        let client = reqwest::Client::new();
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            UserRoutes::TwoFA.addr()
        );

        let valid_token = test_setup.get_valid_token();
        let body = HashMap::from([("password", TEST_PASSWORD), ("token", &valid_token)]);

        let result = client
            .delete(&url)
            .json(&body)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::OK);

        let user = test_setup.get_model_user().await.unwrap();

        assert!(!user.two_fa_always_required);
        assert!(user.two_fa_secret.is_none());
        assert_eq!(user.two_fa_backup_count, 0);

        // email sent - written to disk when testing
        assert!(std::fs::exists(tmp_file!("email_headers.txt")).unwrap_or_default());

        assert!(std::fs::exists(tmp_file!("email_body.txt")).unwrap_or_default());

        assert!(
            std::fs::read_to_string(tmp_file!("email_body.txt"))
                .unwrap()
                .contains(
                    "You have disabled Two-Factor Authentication for your Meal Pedant account"
                )
        );

        assert!(
            std::fs::read_to_string(tmp_file!("email_headers.txt"))
                .unwrap()
                .contains("Two-Factor Disabled")
        );
    }

    #[tokio::test]
    /// Insert 10 backup codes, as hashes, and return 10 backup codes, as strings, to user
    // expect email to have been sent
    async fn api_router_user_two_post_valid() {
        let mut test_setup = start_both_servers().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.insert_two_fa().await;
        let user = test_setup.get_model_user().await.unwrap();
        ModelTwoFA::update_always_required(&test_setup.postgres, true, &user)
            .await
            .unwrap();

        let client = reqwest::Client::new();
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            UserRoutes::TwoFA.addr()
        );

        let result = client
            .post(&url)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::OK);

        let user = test_setup.get_model_user().await.unwrap();
        assert_eq!(user.two_fa_backup_count, 10);

        let result = result.json::<Response>().await.unwrap().response;
        assert!(result["backups"].is_array());

        assert_eq!(result["backups"].as_array().unwrap().len(), 10);

        assert_eq!(
            result["backups"]
                .as_array()
                .unwrap()
                .first()
                .unwrap()
                .as_str()
                .unwrap()
                .chars()
                .count(),
            16
        );

        // email sent - written to disk when testing
        assert!(std::fs::exists(tmp_file!("email_headers.txt")).unwrap_or_default());

        assert!(std::fs::exists(tmp_file!("email_body.txt")).unwrap_or_default());

        assert!(std::fs::read_to_string(tmp_file!("email_body.txt"))
                .unwrap()
                .contains("You have created Two-Factor Authentication backup codes for your Meal Pedant account. The codes should be stored somewhere secure"));

        assert!(
            std::fs::read_to_string(tmp_file!("email_headers.txt"))
                .unwrap()
                .contains("Two-Factor Backup Enabled")
        );
    }

    #[tokio::test]
    /// Conflict response if two_fa not enabled
    async fn api_router_user_two_patch_two_fa_not_enabled() {
        let mut test_setup = start_both_servers().await;
        let client = reqwest::Client::new();
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            UserRoutes::TwoFA.addr()
        );

        let authed_cookie = test_setup.authed_user_cookie().await;

        let result = client
            .patch(&url)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::CONFLICT);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Two FA not enabled");
    }

    #[tokio::test]
    /// Old backup codes removed, 10 new codes inserted, as hashes, and return 10 backup codes, as strings, to user
    // expect email to have been sent
    async fn api_router_user_two_patch_valid() {
        let mut test_setup = start_both_servers().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.insert_two_fa().await;
        let user = test_setup.get_model_user().await.unwrap();
        ModelTwoFA::update_always_required(&test_setup.postgres, true, &user)
            .await
            .unwrap();

        let client = reqwest::Client::new();
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            UserRoutes::TwoFA.addr()
        );

        let result = client
            .post(&url)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();

        let result = result.json::<Response>().await.unwrap().response;

        let pre_first_code = result["backups"]
            .as_array()
            .unwrap()
            .first()
            .unwrap()
            .as_str();

        let result = client
            .patch(&url)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::OK);
        let result = result.json::<Response>().await.unwrap().response;
        let user = test_setup.get_model_user().await.unwrap();
        assert_eq!(user.two_fa_backup_count, 10);

        assert!(result["backups"].is_array());
        assert_eq!(result["backups"].as_array().unwrap().len(), 10);
        assert_eq!(
            result["backups"]
                .as_array()
                .unwrap()
                .first()
                .unwrap()
                .as_str()
                .unwrap()
                .chars()
                .count(),
            16
        );

        let post_first_code = result["backups"]
            .as_array()
            .unwrap()
            .first()
            .unwrap()
            .as_str();

        assert_ne!(pre_first_code, post_first_code);

        // email sent - written to disk when testing
        assert!(std::fs::exists(tmp_file!("email_headers.txt")).unwrap_or_default());

        assert!(std::fs::exists(tmp_file!("email_body.txt")).unwrap_or_default());

        assert!(std::fs::read_to_string(tmp_file!("email_body.txt"))
                .unwrap()
                .contains("You have created Two-Factor Authentication backup codes for your Meal Pedant account. The codes should be stored somewhere secure"));

        assert!(
            std::fs::read_to_string(tmp_file!("email_headers.txt"))
                .unwrap()
                .contains("Two-Factor Backup Enabled")
        );
    }

    #[tokio::test]
    /// Delete all backup codes
    async fn api_router_user_two_put_valid() {
        let mut test_setup = start_both_servers().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.insert_two_fa().await;
        let user = test_setup.get_model_user().await.unwrap();
        ModelTwoFA::update_always_required(&test_setup.postgres, true, &user)
            .await
            .unwrap();

        let client = reqwest::Client::new();
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            UserRoutes::TwoFA.addr()
        );

        client
            .post(&url)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();

        let valid_token = test_setup.get_valid_token();
        let body = HashMap::from([("password", TEST_PASSWORD), ("token", &valid_token)]);

        let result = client
            .put(&url)
            .json(&body)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::OK);
        let user = test_setup.get_model_user().await.unwrap();
        assert_eq!(user.two_fa_backup_count, 0);

        // email sent - written to disk when testing
        assert!(std::fs::exists(tmp_file!("email_headers.txt")).unwrap_or_default());

        assert!(std::fs::exists(tmp_file!("email_body.txt")).unwrap_or_default());

        assert!(std::fs::read_to_string(tmp_file!("email_body.txt"))
                .unwrap()
                .contains("You have removed the Two-Factor Authentication backup codes for your Meal Pedant account. New backup codes can be created at any time from the user settings page."));

        assert!(
            std::fs::read_to_string(tmp_file!("email_headers.txt"))
                .unwrap()
                .contains("Two-Factor Backup Disabled")
        );
    }
}
