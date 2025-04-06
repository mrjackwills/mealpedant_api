use axum_extra::extract::{
    PrivateCookieJar,
    cookie::{Cookie, SameSite},
};
use cookie::time::Duration;
use sqlx::PgPool;
use std::fmt;
use ulid::Ulid;

use crate::{
    C, S,
    api_error::ApiError,
    argon::ArgonHash,
    database::{
        MealResponse, ModelBannedEmail, ModelLogin, ModelPasswordReset, ModelUser,
        ModelUserAgentIp, RedisNewUser, RedisSession,
    },
    define_routes,
    emailer::{Email, EmailTemplate},
    helpers::{self, calc_uptime, gen_random_hex, xor},
    servers::{
        Outgoing,
        api::{ApiRouter, ApiState},
        authentication::{authenticate_signin, authenticate_token, not_authenticated},
        deserializer::IncomingDeserializer,
        get_cookie_ulid, ij,
        oj::{self, MealInfo},
    },
};
use axum::{
    Router,
    extract::{Path, State},
    middleware,
    response::IntoResponse,
    routing::{get, post},
};

define_routes! {
    IncognitoRoutes,
    "/incognito",
    Online => "/online",
    Register => "/register",
    Reset => "/reset",
    ResetParam => "/reset/{secret}",
    Signin => "/signin",
    VerifyParam => "/verify/{secret}",
    Meals => "/meals",
    MealHash => "/hash"
}

enum IncognitoResponse {
    DomainBanned(String),
    Instructions,
    InviteInvalid,
    UnsafePassword,
    Verified,
    VerifyInvalid,
    ResetPatch,
}

impl fmt::Display for IncognitoResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let disp = match self {
            Self::DomainBanned(domain) => format!("{domain} is a banned domain"),
            Self::InviteInvalid => S!("invite invalid"),
            Self::UnsafePassword => S!("unsafe password"),
            Self::Verified => S!("Account verified, please sign in to continue"),
            Self::VerifyInvalid => S!("Incorrect verification data"),
            Self::Instructions => {
                S!("Instructions have been sent to the email address provided")
            }
            Self::ResetPatch => S!("Password reset complete - please sign in"),
        };
        write!(f, "{disp}")
    }
}

pub struct IncognitoRouter;

impl ApiRouter for IncognitoRouter {
    fn create_router(state: &ApiState) -> Router<ApiState> {
        Router::new()
            .route(&IncognitoRoutes::Register.addr(), post(Self::register_post))
            .route(
                &IncognitoRoutes::ResetParam.addr(),
                get(Self::reset_param_get).patch(Self::reset_param_patch),
            )
            .route(&IncognitoRoutes::Reset.addr(), post(Self::reset_post))
            .route(
                &IncognitoRoutes::VerifyParam.addr(),
                get(Self::verify_param_get),
            )
            .layer(middleware::from_fn_with_state(C!(state), not_authenticated))
            .route(&IncognitoRoutes::Meals.addr(), get(Self::meals_get))
            .route(&IncognitoRoutes::MealHash.addr(), get(Self::hash_get))
            .route(&IncognitoRoutes::Signin.addr(), post(Self::signin_post))
            .route(&IncognitoRoutes::Online.addr(), get(Self::get_online))
    }
}

impl IncognitoRouter {
    /// Return a simple online status response
    #[expect(clippy::unused_async)]
    async fn get_online(State(state): State<ApiState>) -> impl IntoResponse {
        (
            axum::http::StatusCode::OK,
            oj::OutgoingJson::new(oj::Online {
                uptime: calc_uptime(state.start_time),
                api_version: env!("CARGO_PKG_VERSION").into(),
            }),
        )
    }

    async fn meals_get(State(state): State<ApiState>) -> Result<Outgoing<MealInfo>, ApiError> {
        Ok((
            axum::http::StatusCode::OK,
            oj::OutgoingJson::new(
                MealResponse::get_all(&state.postgres, &state.redis, None).await?,
            ),
        ))
    }

    async fn hash_get(State(state): State<ApiState>) -> Result<Outgoing<String>, ApiError> {
        Ok((
            axum::http::StatusCode::OK,
            oj::OutgoingJson::new(
                MealResponse::get_hash(&state.postgres, &state.redis, None).await?,
            ),
        ))
    }

    /// Insert a password reset entry, email user the secret link
    /// Always return same response, even if user/email isn't known in database
    async fn reset_post(
        useragent_ip: ModelUserAgentIp,
        State(state): State<ApiState>,
        ij::IncomingJson(body): ij::IncomingJson<ij::Reset>,
    ) -> Result<Outgoing<String>, ApiError> {
        let (op_reset_in_progress, op_user) = tokio::try_join!(
            ModelPasswordReset::get_by_email(&state.postgres, &body.email),
            ModelUser::get(&state.postgres, &body.email)
        )?;

        if let (Some(user), None) = (op_user, op_reset_in_progress) {
            let secret = gen_random_hex(128);
            ModelPasswordReset::insert(
                &state.postgres,
                user.registered_user_id,
                &secret,
                useragent_ip,
            )
            .await?;
            Email::new(
                &user.full_name,
                &user.email,
                EmailTemplate::PasswordResetRequested(secret),
                &state.email_env,
            )
            .send();
        }
        Ok((
            axum::http::StatusCode::OK,
            oj::OutgoingJson::new(IncognitoResponse::Instructions.to_string()),
        ))
    }

    async fn reset_param_patch(
        Path(secret): Path<String>,
        State(state): State<ApiState>,
        ij::IncomingJson(body): ij::IncomingJson<ij::PasswordToken>,
    ) -> Result<Outgoing<String>, ApiError> {
        match ModelPasswordReset::get_by_secret(&state.postgres, &secret).await? {
            Some(reset_user) => {
                if let Some(two_fa_secret) = reset_user.two_fa_secret {
                    if !authenticate_token(
                        body.token,
                        &state.postgres,
                        &two_fa_secret,
                        reset_user.registered_user_id,
                        reset_user.two_fa_backup_count.unwrap_or_default(),
                    )
                    .await?
                    {
                        return Err(ApiError::Authorization);
                    }
                }

                // Check if password is exposed in HIBP or new_password contains users email address
                if helpers::pwned_password(&body.password).await?
                    || body
                        .password
                        .to_lowercase()
                        .contains(&reset_user.email.to_lowercase())
                {
                    return Err(ApiError::InvalidValue(
                        IncognitoResponse::UnsafePassword.to_string(),
                    ));
                }

                let password_hash = ArgonHash::new(C!(body.password)).await?;

                tokio::try_join!(
                    ModelUser::update_password(
                        &state.postgres,
                        reset_user.registered_user_id,
                        password_hash
                    ),
                    ModelPasswordReset::consume(&state.postgres, reset_user.password_reset_id)
                )?;

                Email::new(
                    &reset_user.full_name,
                    &reset_user.email,
                    EmailTemplate::PasswordChanged,
                    &state.email_env,
                )
                .send();
                Ok((
                    axum::http::StatusCode::OK,
                    oj::OutgoingJson::new(IncognitoResponse::ResetPatch.to_string()),
                ))
            }
            _ => Err(ApiError::InvalidValue(
                IncognitoResponse::VerifyInvalid.to_string(),
            )),
        }
    }

    /// check if a given reset string is still valid, and also the two-fa status of the user
    async fn reset_param_get(
        Path(secret): Path<String>,
        State(state): State<ApiState>,
    ) -> Result<Outgoing<oj::PasswordReset>, ApiError> {
        if !IncomingDeserializer::is_hex(&secret, 128) {
            return Err(ApiError::InvalidValue(
                IncognitoResponse::VerifyInvalid.to_string(),
            ));
        }
        match ModelPasswordReset::get_by_secret(&state.postgres, &secret).await? {
            Some(valid_reset) => {
                let response = oj::PasswordReset {
                    two_fa_active: valid_reset.two_fa_secret.is_some(),
                    two_fa_backup: valid_reset.two_fa_backup_count.is_some_and(|i| i > 0),
                };
                Ok((axum::http::StatusCode::OK, oj::OutgoingJson::new(response)))
            }
            _ => Err(ApiError::InvalidValue(
                IncognitoResponse::VerifyInvalid.to_string(),
            )),
        }
    }

    /// User gets emailed a link when they sign up, they hit this route and it verifies the email address
    /// and insert the new user into postgres
    async fn verify_param_get(
        Path(secret): Path<String>,
        State(state): State<ApiState>,
    ) -> Result<Outgoing<String>, ApiError> {
        if !IncomingDeserializer::is_hex(&secret, 128) {
            return Err(ApiError::InvalidValue(
                IncognitoResponse::VerifyInvalid.to_string(),
            ));
        }

        match RedisNewUser::get(&state.redis, &secret).await? {
            Some(new_user) => {
                ModelUser::insert(&state.postgres, &new_user).await?;
                RedisNewUser::delete(&new_user, &state.redis, &secret).await?;
                Ok((
                    axum::http::StatusCode::OK,
                    oj::OutgoingJson::new(IncognitoResponse::Verified.to_string()),
                ))
            }
            _ => Err(ApiError::InvalidValue(
                IncognitoResponse::VerifyInvalid.to_string(),
            )),
        }
    }

    async fn invalid_signin(
        postgres: &PgPool,
        registered_user_id: i64,
        useragent_ip: ModelUserAgentIp,
    ) -> Result<ApiError, ApiError> {
        ModelLogin::insert(postgres, registered_user_id, useragent_ip, false, None).await?;
        Ok(ApiError::Authorization)
    }

    // this is where one needs to check password, token, create session, create cookie,
    // Redirect to /user, so can get user object?
    async fn signin_post(
        State(state): State<ApiState>,
        useragent_ip: ModelUserAgentIp,
        jar: PrivateCookieJar,
        ij::IncomingJson(body): ij::IncomingJson<ij::Signin>,
    ) -> Result<impl IntoResponse, ApiError> {
        // If front end and back end out of sync, and front end user has an api cookie, but not front-end authed, delete server cookie api session
        if let Some(ulid) = get_cookie_ulid(&state, &jar) {
            RedisSession::delete(&state.redis, &ulid).await?;
        }

        match ModelUser::get(&state.postgres, &body.email).await? {
            Some(user) => {
                // Email user that account is blocked
                if user.login_attempt_number == 19 {
                    Email::new(
                        &user.full_name,
                        &user.email,
                        EmailTemplate::AccountLocked,
                        &state.email_env,
                    )
                    .send();
                }

                // Don't allow blocked accounts to even try to authenticate
                if user.login_attempt_number >= 19 {
                    return Err(Self::invalid_signin(
                        &state.postgres,
                        user.registered_user_id,
                        useragent_ip,
                    )
                    .await?);
                }

                // If twofa token required, but not sent, 202 response - as long as password is valid
                if user.two_fa_secret.is_some() && body.token.is_none() {
                    if crate::argon::verify_password(&body.password, user.get_password_hash())
                        .await?
                    {
                        ModelLogin::insert(
                            &state.postgres,
                            user.registered_user_id,
                            useragent_ip,
                            false,
                            None,
                        )
                        .await?;
                        // So that the function return type can be strict
                        // need to included two_backup as a bool
                        return Ok((
                            axum::http::StatusCode::ACCEPTED,
                            oj::OutgoingJson::new(oj::SigninAccepted {
                                two_fa_backup: user.two_fa_backup_count > 0,
                            }),
                        )
                            .into_response());
                    }
                    return Err(Self::invalid_signin(
                        &state.postgres,
                        user.registered_user_id,
                        useragent_ip,
                    )
                    .await?);
                }

                if !authenticate_signin(&user, &body.password, body.token, &state.postgres).await? {
                    return Err(Self::invalid_signin(
                        &state.postgres,
                        user.registered_user_id,
                        useragent_ip,
                    )
                    .await?);
                }

                let ulid = Ulid::new();
                ModelLogin::insert(
                    &state.postgres,
                    user.registered_user_id,
                    useragent_ip,
                    true,
                    Some(ulid),
                )
                .await?;

                let ttl = if body.remember {
                    Duration::days(7 * 4 * 6)
                } else {
                    Duration::hours(6)
                };

                let mut cookie = Cookie::new(C!(state.cookie_name), ulid.to_string());
                cookie.set_domain(C!(state.domain));
                cookie.set_path("/");
                cookie.set_secure(state.run_mode.is_production());
                cookie.set_same_site(SameSite::Strict);
                cookie.set_http_only(true);
                cookie.set_max_age(ttl);

                RedisSession::new(user.registered_user_id, &user.email)
                    .insert(&state.redis, ttl, ulid)
                    .await?;
                Ok(jar.add(cookie).into_response())
            }
            _ => {
                // No known user
                // Add an artificial delay? Of between 500ms and 1500ms?
                Err(ApiError::Authorization)
            }
        }
    }

    async fn register_post(
        State(state): State<ApiState>,
        useragent_ip: ModelUserAgentIp,
        ij::IncomingJson(body): ij::IncomingJson<ij::Register>,
    ) -> impl IntoResponse {
        // Should maybe xor_hash compare instead?
        if !xor(body.invite.as_bytes(), state.invite.as_bytes()) {
            return Err(ApiError::InvalidValue(
                IncognitoResponse::InviteInvalid.to_string(),
            ));
        }

        if let Some(domain) = ModelBannedEmail::get(&state.postgres, &body.email).await? {
            return Err(ApiError::InvalidValue(
                IncognitoResponse::DomainBanned(domain.domain).to_string(),
            ));
        }

        // Check if password is exposed in HIBP
        if helpers::pwned_password(&body.password).await? {
            return Err(ApiError::InvalidValue(
                IncognitoResponse::UnsafePassword.to_string(),
            ));
        }

        let response = (
            axum::http::StatusCode::OK,
            oj::OutgoingJson::new(IncognitoResponse::Instructions.to_string()),
        );

        // If email address can be found in redis verify cache, or postgres proper, just return a success response
        // Shouldn't even let a client know if a user is registered or not
        let (redis_user, postgres_user) = tokio::try_join!(
            RedisNewUser::exists(&state.redis, &body.email),
            ModelUser::get(&state.postgres, &body.email)
        )?;

        if redis_user || postgres_user.is_some() {
            return Ok(response);
        }

        let password_hash = ArgonHash::new(C!(body.password)).await?;
        let secret = gen_random_hex(128);

        RedisNewUser::new(&body.email, &body.full_name, &password_hash, &useragent_ip)
            .insert(&state.redis, &secret)
            .await?;

        // Email user verification code/link email
        Email::new(
            &body.full_name,
            &body.email,
            EmailTemplate::Verify(secret),
            &state.email_env,
        )
        .send();

        Ok(response)
    }
}

/// Use reqwest to test against real server
/// cargo watch -q -c -w src/ -x 'test api_router_incognito -- --test-threads=1 --nocapture'
#[cfg(test)]
#[expect(clippy::pedantic, clippy::unwrap_used)]
mod tests {

    use crate::database::{ModelLogin, ModelPasswordReset, RedisNewUser, RedisSession};
    use crate::helpers::gen_random_hex;
    use crate::parse_env::AppEnv;
    use crate::servers::api::routers::incognito::IncognitoRoutes;
    use crate::servers::api_tests::{
        Response, TEST_EMAIL, TEST_PASSWORD, TEST_PASSWORD_HASH, TestSetup, base_url, get_keys,
        start_both_servers,
    };
    use crate::servers::deserializer::IncomingDeserializer;
    use crate::{C, S, sleep, tmp_file};

    use fred::interfaces::{HashesInterface, KeysInterface, SetsInterface};

    use reqwest::StatusCode;
    use sqlx::PgPool;
    use std::collections::HashMap;

    /// Send a request to insert a password_reset
    async fn request_reset(app_env: &AppEnv, postgres: &PgPool) -> String {
        let client = reqwest::Client::new();
        let url = format!("{}/incognito/reset", base_url(app_env));
        let body = HashMap::from([("email", TEST_EMAIL)]);
        client.post(&url).json(&body).send().await.unwrap();
        ModelPasswordReset::get_by_email(postgres, TEST_EMAIL)
            .await
            .unwrap()
            .unwrap()
            .reset_string
    }

    #[tokio::test]
    async fn api_router_incognito_get_online() {
        let test_setup = start_both_servers().await;
        let url = format!("{}/incognito/online", base_url(&test_setup.app_env));
        sleep!();
        let resp = reqwest::get(url).await.unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let result = resp.json::<Response>().await.unwrap().response;
        assert_eq!(result["api_version"], env!("CARGO_PKG_VERSION"));
        assert_eq!(result["uptime"], 1);
    }

    #[tokio::test]
    async fn api_router_incognito_get_online_when_authenticated() {
        let mut test_setup = start_both_servers().await;
        let url = format!("{}/incognito/online", base_url(&test_setup.app_env));
        let client = reqwest::Client::new();
        sleep!();

        let authed_cookie = test_setup.authed_user_cookie().await;

        let resp = client
            .get(url)
            .header("cookie", authed_cookie)
            .send()
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let result = resp.json::<Response>().await.unwrap().response;
        assert_eq!(result["api_version"], env!("CARGO_PKG_VERSION"));
        assert_eq!(result["uptime"], 1);
    }

    #[tokio::test]
    async fn api_router_incognito_register_invalid_invite() {
        let test_setup = start_both_servers().await;
        let client = reqwest::Client::new();
        let url = format!("{}/incognito/register", base_url(&test_setup.app_env));

        let body =
            TestSetup::gen_register_body("name", TEST_PASSWORD, "some_long_invite", TEST_EMAIL);
        let result = client.post(&url).json(&body).send().await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            result.json::<Response>().await.unwrap().response,
            "invite invalid"
        );
    }

    #[tokio::test]
    async fn api_router_incognito_register_banned_email() {
        let test_setup = start_both_servers().await;
        let client = reqwest::Client::new();
        let url = format!("{}/incognito/register", base_url(&test_setup.app_env));
        let body = TestSetup::gen_register_body(
            "name",
            TEST_PASSWORD,
            &test_setup.app_env.invite,
            "email@0-mail.com",
        );
        let result = client.post(&url).json(&body).send().await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            result.json::<Response>().await.unwrap().response,
            "0-mail.com is a banned domain"
        );
    }

    #[tokio::test]
    async fn api_router_incognito_register_short_password() {
        let test_setup = start_both_servers().await;
        let client = reqwest::Client::new();
        let url = format!("{}/incognito/register", base_url(&test_setup.app_env));

        let body = TestSetup::gen_register_body(
            "name",
            "password123",
            &test_setup.app_env.invite,
            TEST_EMAIL,
        );
        let result = client.post(&url).json(&body).send().await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            result.json::<Response>().await.unwrap().response,
            "password"
        );
    }

    #[tokio::test]
    async fn api_router_incognito_register_hibp_password() {
        let test_setup = start_both_servers().await;
        let client = reqwest::Client::new();
        let url = format!("{}/incognito/register", base_url(&test_setup.app_env));

        let body = TestSetup::gen_register_body(
            "name",
            "ILOVEYOU1234",
            &test_setup.app_env.invite,
            TEST_EMAIL,
        );
        let result = client.post(&url).json(&body).send().await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            result.json::<Response>().await.unwrap().response,
            "unsafe password"
        );
    }

    #[tokio::test]
    async fn api_router_incognito_register_already_registered() {
        let mut test_setup = start_both_servers().await;
        let client = reqwest::Client::new();
        let url = format!("{}/incognito/register", base_url(&test_setup.app_env));

        test_setup.insert_test_user().await;

        let body = TestSetup::gen_register_body(
            "name",
            TEST_PASSWORD,
            &test_setup.app_env.invite,
            TEST_EMAIL,
        );
        let result = client.post(&url).json(&body).send().await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.status(), StatusCode::OK);
        assert_eq!(
            result.json::<Response>().await.unwrap().response,
            "Instructions have been sent to the email address provided"
        );

        // Check email HAS NOT been sent
        let result = RedisNewUser::exists(&test_setup.redis, "email@mrjackwills.com").await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
        assert!(!std::fs::exists(tmp_file!("email_headers.txt")).unwrap_or_default());
        assert!(!std::fs::exists(tmp_file!("email_body.txt")).unwrap_or_default());
    }

    #[tokio::test]
    /// If authenticated, unable to access register endpoint
    async fn api_router_incognito_register_already_authenticated() {
        let mut test_setup = start_both_servers().await;
        let client = reqwest::Client::new();
        let url = format!("{}/incognito/register", base_url(&test_setup.app_env));
        let authed_cookie = test_setup.authed_user_cookie().await;

        let body = TestSetup::gen_register_body(
            "name",
            TEST_PASSWORD,
            &test_setup.app_env.invite,
            TEST_EMAIL,
        );
        let result = client
            .post(&url)
            .json(&body)
            .header("cookie", authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        assert_eq!(
            "Invalid Authentication",
            result.json::<Response>().await.unwrap().response
        );
    }

    #[tokio::test]
    async fn api_router_incognito_register_newuser_in_redis() {
        let test_setup = start_both_servers().await;
        let client = reqwest::Client::new();
        let url = format!("{}/incognito/register", base_url(&test_setup.app_env));

        let body = TestSetup::gen_register_body(
            "name",
            TEST_PASSWORD,
            &test_setup.app_env.invite,
            TEST_EMAIL,
        );
        let result = client.post(&url).json(&body).send().await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.status(), StatusCode::OK);
        assert_eq!(
            result.json::<Response>().await.unwrap().response,
            "Instructions have been sent to the email address provided"
        );
        let result = RedisNewUser::exists(&test_setup.redis, TEST_EMAIL).await;
        assert!(result.is_ok());
        assert!(result.unwrap());

        // check email sent - well written to disk
        assert!(std::fs::exists(tmp_file!("email_headers.txt")).unwrap_or_default());
        assert!(std::fs::exists(tmp_file!("email_body.txt")).unwrap_or_default());
        let link = format!(
            "href=\"https://www.{}/user/verify/",
            test_setup.app_env.domain
        );
        assert!(
            std::fs::read_to_string(tmp_file!("email_body.txt"))
                .unwrap()
                .contains(&link)
        );
    }

    #[tokio::test]
    async fn api_router_incognito_register_register_twice() {
        let test_setup = start_both_servers().await;
        let client = reqwest::Client::new();
        let url = format!("{}/incognito/register", base_url(&test_setup.app_env));

        let body = TestSetup::gen_register_body(
            "name",
            TEST_PASSWORD,
            &test_setup.app_env.invite,
            TEST_EMAIL,
        );
        let result = client.post(&url).json(&body).send().await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.status(), StatusCode::OK);
        assert_eq!(
            result.json::<Response>().await.unwrap().response,
            "Instructions have been sent to the email address provided"
        );

        let result = RedisNewUser::exists(&test_setup.redis, TEST_EMAIL).await;
        assert!(result.is_ok());
        assert!(result.unwrap());

        // check email sent - well written to disk
        assert!(std::fs::exists(tmp_file!("email_headers.txt")).unwrap_or_default());
        assert!(std::fs::exists(tmp_file!("email_body.txt")).unwrap_or_default());
        let link = format!(
            "href=\"https://www.{}/user/verify/",
            test_setup.app_env.domain
        );
        assert!(
            std::fs::read_to_string(tmp_file!("email_body.txt"))
                .unwrap()
                .contains(&link)
        );

        let first_secret = get_keys(&test_setup.redis, "verify::secret::*").await;

        TestSetup::delete_emails();

        let body = TestSetup::gen_register_body(
            "name",
            TEST_PASSWORD,
            &test_setup.app_env.invite,
            TEST_EMAIL,
        );
        let result = client.post(&url).json(&body).send().await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.status(), StatusCode::OK);
        assert_eq!(
            result.json::<Response>().await.unwrap().response,
            "Instructions have been sent to the email address provided"
        );

        let result = RedisNewUser::exists(&test_setup.redis, TEST_EMAIL).await;
        assert!(result.is_ok());
        assert!(result.unwrap());
        assert!(!std::fs::exists(tmp_file!("email_headers.txt")).unwrap_or_default());
        assert!(!std::fs::exists(tmp_file!("email_body.txt")).unwrap_or_default());

        let second_secret = get_keys(&test_setup.redis, "verify::secret::*").await;
        assert_eq!(first_secret, second_secret);
    }

    #[tokio::test]
    async fn api_router_incognito_register_then_verify_ok() {
        let test_setup = start_both_servers().await;
        let client = reqwest::Client::new();
        let url = format!("{}/incognito/register", base_url(&test_setup.app_env));

        let body = TestSetup::gen_register_body(
            "name",
            TEST_PASSWORD,
            &test_setup.app_env.invite,
            TEST_EMAIL,
        );
        client.post(&url).json(&body).send().await.unwrap();
        let secret = get_keys(&test_setup.redis, "verify::secret::*").await;
        let secret = secret[0].replace("verify::secret::", "");

        let url = format!(
            "{}/incognito/verify/{}",
            base_url(&test_setup.app_env),
            secret
        );

        let result = reqwest::get(url).await;

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.status(), StatusCode::OK);

        assert_eq!(
            result.json::<Response>().await.unwrap().response,
            "Account verified, please sign in to continue"
        );

        let result = RedisNewUser::get(&test_setup.redis, TEST_EMAIL).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());

        let result = RedisNewUser::exists(&test_setup.redis, TEST_EMAIL).await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[tokio::test]
    async fn api_router_incognito_reset_post_unknown_user() {
        let test_setup = start_both_servers().await;
        let client = reqwest::Client::new();
        let url = format!("{}/incognito/reset", base_url(&test_setup.app_env));

        let body = HashMap::from([("email", TEST_EMAIL)]);

        let result = client.post(&url).json(&body).send().await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.status(), StatusCode::OK);
        assert_eq!(
            result.json::<Response>().await.unwrap().response,
            "Instructions have been sent to the email address provided"
        );

        // check email NOT sent - well written to disk
        assert!(!std::fs::exists(tmp_file!("email_headers.txt")).unwrap_or_default());
        assert!(!std::fs::exists(tmp_file!("email_body.txt")).unwrap_or_default());
    }

    #[tokio::test]
    async fn api_router_incognito_reset_post_known_user() {
        let mut test_setup = start_both_servers().await;
        test_setup.insert_test_user().await;

        let client = reqwest::Client::new();
        let url = format!("{}/incognito/reset", base_url(&test_setup.app_env));

        let body = HashMap::from([("email", TEST_EMAIL)]);

        let result = client.post(&url).json(&body).send().await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.status(), StatusCode::OK);
        assert_eq!(
            result.json::<Response>().await.unwrap().response,
            "Instructions have been sent to the email address provided"
        );

        // Check postgres_secret is in email
        let password_reset =
            ModelPasswordReset::get_by_email(&test_setup.postgres, TEST_EMAIL).await;

        assert!(password_reset.is_ok());
        let password_reset = password_reset.unwrap();

        assert!(password_reset.is_some());
        let password_reset = password_reset.unwrap();

        // check email has been sent - well written to disk, and contain secret & correct subject
        let result = std::fs::read_to_string(tmp_file!("email_headers.txt"));
        assert!(result.is_ok());
        assert!(
            result
                .unwrap()
                .contains("Subject: Password Reset Requested")
        );

        let result = std::fs::read_to_string(tmp_file!("email_body.txt"));
        assert!(result.is_ok());
        assert!(result.unwrap().contains(&password_reset.reset_string));
    }

    #[tokio::test]
    async fn api_router_incognito_reset_post_known_user_second_attempt() {
        // setup
        let mut test_setup = start_both_servers().await;
        test_setup.insert_test_user().await;
        let client = reqwest::Client::new();
        let url = format!("{}/incognito/reset", base_url(&test_setup.app_env));
        let body = HashMap::from([("email", TEST_EMAIL)]);

        // test
        let result = client.post(&url).json(&body).send().await;

        // validate
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.status(), StatusCode::OK);
        assert_eq!(
            result.json::<Response>().await.unwrap().response,
            "Instructions have been sent to the email address provided"
        );
        let first_password_reset =
            ModelPasswordReset::get_by_email(&test_setup.postgres, TEST_EMAIL)
                .await
                .unwrap();
        TestSetup::delete_emails();

        // Second second request, no emails should be sent, and password_reset should match new password_reset
        let result = client.post(&url).json(&body).send().await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.status(), StatusCode::OK);
        assert_eq!(
            result.json::<Response>().await.unwrap().response,
            "Instructions have been sent to the email address provided"
        );

        let second_password_reset =
            ModelPasswordReset::get_by_email(&test_setup.postgres, TEST_EMAIL)
                .await
                .unwrap();

        assert_eq!(first_password_reset, second_password_reset);

        assert!(!std::fs::exists(tmp_file!("email_headers.txt")).unwrap_or_default());
        assert!(!std::fs::exists(tmp_file!("email_body.txt")).unwrap_or_default());
    }

    #[tokio::test]
    /// If authenticated, unable to access reset_post endpoint
    async fn api_router_incognito_reset_post_already_authenticated() {
        let mut test_setup = start_both_servers().await;
        let client = reqwest::Client::new();
        let url = format!("{}/incognito/reset", base_url(&test_setup.app_env));
        let authed_cookie = test_setup.authed_user_cookie().await;

        let body = TestSetup::gen_register_body(
            "name",
            TEST_PASSWORD,
            &test_setup.app_env.invite,
            TEST_EMAIL,
        );
        let result = client
            .post(&url)
            .json(&body)
            .header("cookie", authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        assert_eq!(
            "Invalid Authentication",
            result.json::<Response>().await.unwrap().response
        );
    }

    #[tokio::test]
    /// If authenticated, unable to access reset_get endpoint
    async fn api_router_incognito_reset_get_already_authenticated() {
        let mut test_setup = start_both_servers().await;
        let client = reqwest::Client::new();
        let url = format!("{}/incognito/reset", base_url(&test_setup.app_env));
        let authed_cookie = test_setup.authed_user_cookie().await;

        let result = client
            .get(&url)
            .header("cookie", authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        assert_eq!(
            "Invalid Authentication",
            result.json::<Response>().await.unwrap().response
        );
    }

    #[tokio::test]
    async fn api_router_incognito_reset_get_invalid_hex() {
        let mut test_setup = start_both_servers().await;
        test_setup.insert_test_user().await;
        let client = reqwest::Client::new();
        let url = format!("{}/incognito/reset", base_url(&test_setup.app_env));
        let body = HashMap::from([("email", TEST_EMAIL)]);

        client.post(&url).json(&body).send().await.unwrap();
        let url = format!(
            "{}/incognito/reset/{}",
            base_url(&test_setup.app_env),
            gen_random_hex(127)
        );

        // Test
        let result = reqwest::get(url).await;

        // Validate
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            result.json::<Response>().await.unwrap().response,
            "Incorrect verification data"
        );
    }

    #[tokio::test]
    async fn api_router_incognito_reset_get_unknown_reset_secret() {
        let mut test_setup = start_both_servers().await;
        test_setup.insert_test_user().await;
        let client = reqwest::Client::new();
        let url = format!("{}/incognito/reset", base_url(&test_setup.app_env));
        let body = HashMap::from([("email", TEST_EMAIL)]);

        client.post(&url).json(&body).send().await.unwrap();
        let url = format!(
            "{}/incognito/reset/{}",
            base_url(&test_setup.app_env),
            gen_random_hex(128)
        );

        // Test
        let result = reqwest::get(url).await;

        // Validate
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            result.json::<Response>().await.unwrap().response,
            "Incorrect verification data"
        );
    }

    #[tokio::test]
    async fn api_router_incognito_reset_get_valid() {
        let mut test_setup = start_both_servers().await;
        test_setup.insert_test_user().await;
        let client = reqwest::Client::new();
        let url = format!("{}/incognito/reset", base_url(&test_setup.app_env));
        let body = HashMap::from([("email", TEST_EMAIL)]);

        client.post(&url).json(&body).send().await.unwrap();

        let secret = ModelPasswordReset::get_by_email(&test_setup.postgres, TEST_EMAIL)
            .await
            .unwrap()
            .unwrap()
            .reset_string;
        let url = format!(
            "{}/incognito/reset/{}",
            base_url(&test_setup.app_env),
            secret
        );

        // Test
        let result = reqwest::get(url).await;

        // Validate
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.status(), StatusCode::OK);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result["two_fa_active"], false);
        assert_eq!(result["two_fa_backup"], false);
    }

    #[tokio::test]
    async fn api_router_incognito_reset_get_valid_with_two_fa() {
        let mut test_setup = start_both_servers().await;
        test_setup.insert_test_user().await;
        test_setup.insert_two_fa().await;
        let client = reqwest::Client::new();
        let url = format!("{}/incognito/reset", base_url(&test_setup.app_env));
        let body = HashMap::from([("email", TEST_EMAIL)]);

        client.post(&url).json(&body).send().await.unwrap();

        let secret = ModelPasswordReset::get_by_email(&test_setup.postgres, TEST_EMAIL)
            .await
            .unwrap()
            .unwrap()
            .reset_string;
        let url = format!(
            "{}/incognito/reset/{}",
            base_url(&test_setup.app_env),
            secret
        );

        // Test
        let result = reqwest::get(url).await;

        // Validate
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.status(), StatusCode::OK);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result["two_fa_active"], true);
        assert_eq!(result["two_fa_backup"], false);
    }

    #[tokio::test]
    /// Secret param incorrect
    async fn api_router_incognito_reset_patch_invalid_secret() {
        let mut test_setup = start_both_servers().await;
        test_setup.insert_test_user().await;
        request_reset(&test_setup.app_env, &test_setup.postgres).await;

        let client = reqwest::Client::new();

        // Wrong secret
        let bad_secret = gen_random_hex(128);
        let url = format!(
            "{}/incognito/reset/{}",
            base_url(&test_setup.app_env),
            bad_secret
        );

        let body = HashMap::from([("password", TEST_PASSWORD)]);
        let result = client.patch(&url).json(&body).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            result.json::<Response>().await.unwrap().response,
            "Incorrect verification data"
        );

        // short secret
        let bad_secret = gen_random_hex(100);
        let url = format!(
            "{}/incognito/reset/{}",
            base_url(&test_setup.app_env),
            bad_secret
        );

        let result = client.patch(&url).json(&body).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            result.json::<Response>().await.unwrap().response,
            "Incorrect verification data"
        );

        // long secret
        let bad_secret = gen_random_hex(200);
        let url = format!(
            "{}/incognito/reset/{}",
            base_url(&test_setup.app_env),
            bad_secret
        );
        let result = client.patch(&url).json(&body).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            result.json::<Response>().await.unwrap().response,
            "Incorrect verification data"
        );
    }

    #[tokio::test]
    /// If authenticated, unable to access reset_patch endpoint
    async fn api_router_incognito_reset_patch_already_authenticated() {
        let mut test_setup = start_both_servers().await;
        let client = reqwest::Client::new();
        let url = format!("{}/incognito/reset", base_url(&test_setup.app_env));
        let authed_cookie = test_setup.authed_user_cookie().await;

        let result = client
            .patch(&url)
            .body("body")
            .header("cookie", authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        assert_eq!(
            "Invalid Authentication",
            result.json::<Response>().await.unwrap().response
        );
    }

    #[tokio::test]
    /// Invalid body
    async fn api_router_incognito_reset_patch_invalid_body() {
        let mut test_setup = start_both_servers().await;
        test_setup.insert_test_user().await;
        let reset_secret = request_reset(&test_setup.app_env, &test_setup.postgres).await;
        let client = reqwest::Client::new();

        let url = format!(
            "{}/incognito/reset/{}",
            base_url(&test_setup.app_env),
            reset_secret
        );

        // No password in body
        let body: HashMap<String, String> = HashMap::new();
        let result = client.patch(&url).json(&body).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            result.json::<Response>().await.unwrap().response,
            "missing password"
        );

        // random entry in body
        let body = HashMap::from([("password", TEST_PASSWORD), ("not_token", "012234")]);
        let result = client.patch(&url).json(&body).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            result.json::<Response>().await.unwrap().response,
            "invalid input"
        );

        // invalid token format
        let body = HashMap::from([("password", TEST_PASSWORD), ("token", "8102569")]);
        let result = client.patch(&url).json(&body).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::BAD_REQUEST);
        assert_eq!(result.json::<Response>().await.unwrap().response, "token");
    }

    #[tokio::test]
    /// invalid token
    async fn api_router_incognito_reset_patch_invalid_token() {
        let mut test_setup = start_both_servers().await;
        test_setup.insert_test_user().await;
        test_setup.insert_two_fa().await;
        let reset_secret = request_reset(&test_setup.app_env, &test_setup.postgres).await;
        let client = reqwest::Client::new();
        let valid_token = test_setup.get_invalid_token();

        // invalid token format
        let url = format!(
            "{}/incognito/reset/{}",
            base_url(&test_setup.app_env),
            reset_secret
        );

        let body = HashMap::from([("password", TEST_PASSWORD), ("token", &valid_token)]);
        let result = client.patch(&url).json(&body).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            result.json::<Response>().await.unwrap().response,
            "Invalid email address and/or password and/or token"
        );
    }

    #[tokio::test]
    /// Invalid password provided
    async fn api_router_incognito_reset_patch_invalid_password() {
        let mut test_setup = start_both_servers().await;
        test_setup.insert_test_user().await;
        let reset_secret = request_reset(&test_setup.app_env, &test_setup.postgres).await;
        let client = reqwest::Client::new();

        // password in hibp
        let url = format!(
            "{}/incognito/reset/{}",
            base_url(&test_setup.app_env),
            reset_secret
        );
        let body = HashMap::from([("password", "ILOVEYOU1234")]);
        let result = client.patch(&url).json(&body).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            result.json::<Response>().await.unwrap().response,
            "unsafe password"
        );

        // user's email address in password
        let url = format!(
            "{}/incognito/reset/{}",
            base_url(&test_setup.app_env),
            reset_secret
        );

        let body = HashMap::from([(
            "password",
            format!("abcd{}123456", TEST_EMAIL.to_uppercase()),
        )]);

        let result = client.patch(&url).json(&body).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            result.json::<Response>().await.unwrap().response,
            "unsafe password"
        );
    }

    #[tokio::test]
    /// Patch ok
    async fn api_router_incognito_reset_patch_ok() {
        let mut test_setup = start_both_servers().await;
        test_setup.insert_test_user().await;
        let reset_secret = request_reset(&test_setup.app_env, &test_setup.postgres).await;
        let client = reqwest::Client::new();

        let url = format!(
            "{}/incognito/reset/{}",
            base_url(&test_setup.app_env),
            reset_secret
        );

        let body = HashMap::from([("password", gen_random_hex(24))]);

        let result = client.patch(&url).json(&body).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::OK);
        assert_eq!(
            result.json::<Response>().await.unwrap().response,
            "Password reset complete - please sign in"
        );

        let post_hash = test_setup.get_password_hash().await;
        assert_ne!(TEST_PASSWORD_HASH, post_hash);
    }

    #[tokio::test]
    async fn api_router_incognito_reset_patch_ok_with_token() {
        let mut test_setup = start_both_servers().await;
        test_setup.insert_test_user().await;
        test_setup.insert_two_fa().await;
        let reset_secret = request_reset(&test_setup.app_env, &test_setup.postgres).await;
        let client = reqwest::Client::new();
        let valid_token = test_setup.get_valid_token();

        let url = format!(
            "{}/incognito/reset/{}",
            base_url(&test_setup.app_env),
            reset_secret
        );
        let body = HashMap::from([("password", gen_random_hex(24)), ("token", valid_token)]);

        let result = client.patch(&url).json(&body).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::OK);
        assert_eq!(
            result.json::<Response>().await.unwrap().response,
            "Password reset complete - please sign in"
        );

        let post_hash = test_setup.get_password_hash().await;
        assert_ne!(TEST_PASSWORD_HASH, post_hash);
    }

    #[tokio::test]
    /// Password reset consumed, unable to be used again
    async fn api_router_incognito_reset_patch_secret_consumed() {
        let mut test_setup = start_both_servers().await;
        test_setup.insert_test_user().await;
        let reset_secret = request_reset(&test_setup.app_env, &test_setup.postgres).await;
        let client = reqwest::Client::new();

        let url = format!(
            "{}/incognito/reset/{}",
            base_url(&test_setup.app_env),
            reset_secret
        );
        let body = HashMap::from([("password", gen_random_hex(24))]);
        client.patch(&url).json(&body).send().await.unwrap();

        let result = client.patch(&url).json(&body).send().await.unwrap();

        assert_eq!(result.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            result.json::<Response>().await.unwrap().response,
            "Incorrect verification data"
        );
    }

    #[tokio::test]
    /// Unknown user, 403
    async fn api_router_incognito_signin_post_unknown() {
        let test_setup = start_both_servers().await;
        let client = reqwest::Client::new();
        let url = format!("{}/incognito/signin", base_url(&test_setup.app_env));

        let body = TestSetup::gen_signin_body(None, None, None, None);
        let result = client.post(&url).json(&body).send().await.unwrap();

        assert_eq!(result.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            result.json::<Response>().await.unwrap().response,
            "Invalid email address and/or password and/or token"
        );
    }

    #[tokio::test]
    /// invalid login, attempt counter increased
    async fn api_router_incognito_signin_post_login_attempt_increase() {
        let mut test_setup = start_both_servers().await;
        test_setup.insert_test_user().await;
        let client = reqwest::Client::new();
        let url = format!("{}/incognito/signin", base_url(&test_setup.app_env));
        let body =
            TestSetup::gen_signin_body(None, Some(S!("thisistheincorrectpassword")), None, None);

        let result = client.post(&url).json(&body).send().await.unwrap();
        let user = test_setup.get_model_user().await.unwrap();
        let login_count = ModelLogin::get(&test_setup.postgres, user.registered_user_id).await;

        assert_eq!(result.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            result.json::<Response>().await.unwrap().response,
            "Invalid email address and/or password and/or token"
        );
        assert_eq!(login_count.unwrap().unwrap().login_attempt_number, 1);
    }

    #[tokio::test]
    /// invalid login - bad token, login attempt counter increased by one
    async fn api_router_incognito_signin_post_login_bad_token_attempt_increase() {
        let mut test_setup = start_both_servers().await;
        test_setup.insert_test_user().await;
        test_setup.insert_two_fa().await;
        let client = reqwest::Client::new();
        let url = format!("{}/incognito/signin", base_url(&test_setup.app_env));
        let valid_token = test_setup.get_invalid_token();

        let body = TestSetup::gen_signin_body(None, None, Some(valid_token), None);

        let result = client.post(&url).json(&body).send().await.unwrap();
        let user = test_setup.get_model_user().await.unwrap();
        let login_count = ModelLogin::get(&test_setup.postgres, user.registered_user_id).await;
        assert_eq!(result.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            result.json::<Response>().await.unwrap().response,
            "Invalid email address and/or password and/or token"
        );
        assert_eq!(login_count.unwrap().unwrap().login_attempt_number, 1);
    }

    #[tokio::test]
    /// 20 invalid attempts, email sent, all further valid login still unable to complete
    async fn api_router_incognito_signin_post_20_email_sent() {
        let mut test_setup = start_both_servers().await;
        test_setup.insert_test_user().await;
        let client = reqwest::Client::new();

        let url = format!("{}/incognito/signin", base_url(&test_setup.app_env));

        let body =
            TestSetup::gen_signin_body(None, Some(S!("thisistheincorrectpassword")), None, None);

        for _ in 0..=19 {
            client.post(&url).json(&body).send().await.unwrap();
        }

        let result = std::fs::read_to_string(tmp_file!("email_headers.txt"));
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Subject: Security Alert"));
        let result = std::fs::read_to_string(tmp_file!("email_body.txt"));
        assert!(result.is_ok());
        assert!(
            result
                .unwrap()
                .contains("Due to multiple failed login attempts your account has been locked.")
        );

        let body = TestSetup::gen_signin_body(None, None, None, None);

        // Valid login attempt unable to complete
        let result = client.post(&url).json(&body).send().await.unwrap();
        let user = test_setup.get_model_user().await.unwrap();
        let login_count = ModelLogin::get(&test_setup.postgres, user.registered_user_id).await;

        assert_eq!(result.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            result.json::<Response>().await.unwrap().response,
            "Invalid email address and/or password and/or token"
        );
        assert_eq!(login_count.unwrap().unwrap().login_attempt_number, 21);
    }

    #[tokio::test]
    /// After one invalid, and then one valid, signin attempt, login_count = 0
    async fn api_router_incognito_signin_post_login_attempt_reset() {
        let mut test_setup = start_both_servers().await;
        test_setup.insert_test_user().await;
        test_setup.insert_two_fa().await;
        let valid_token = test_setup.get_valid_token();
        let invalid_token = test_setup.get_invalid_token();
        let client = reqwest::Client::new();

        let url = format!("{}/incognito/signin", base_url(&test_setup.app_env));

        let body = TestSetup::gen_signin_body(None, None, Some(invalid_token), None);

        // Valid login attempt unable to complete
        let result = client.post(&url).json(&body).send().await.unwrap();
        let user = test_setup.get_model_user().await.unwrap();
        let login_count = ModelLogin::get(&test_setup.postgres, user.registered_user_id).await;

        assert_eq!(result.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            result.json::<Response>().await.unwrap().response,
            "Invalid email address and/or password and/or token"
        );
        assert_eq!(login_count.unwrap().unwrap().login_attempt_number, 1);

        let body = TestSetup::gen_signin_body(None, None, Some(valid_token), None);

        // Valid login attempt unable to complete
        let result = client.post(&url).json(&body).send().await.unwrap();
        let user = test_setup.get_model_user().await.unwrap();
        let login_count = ModelLogin::get(&test_setup.postgres, user.registered_user_id).await;

        assert_eq!(result.status(), StatusCode::OK);
        assert_eq!(login_count.unwrap().unwrap().login_attempt_number, 0);
    }

    #[tokio::test]
    /// When two factor enabled, but no token provided, should return a 202 message
    async fn api_router_incognito_signin_post_login_no_token() {
        let mut test_setup = start_both_servers().await;
        test_setup.insert_test_user().await;
        test_setup.insert_two_fa().await;
        let client = reqwest::Client::new();
        let user = test_setup.get_model_user().await.unwrap();

        let url = format!("{}/incognito/signin", base_url(&test_setup.app_env));

        let body = TestSetup::gen_signin_body(None, None, None, None);

        // Valid login attempt unable to complete
        let result = client.post(&url).json(&body).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::ACCEPTED);
        let result = result.json::<Response>().await.unwrap().response;

        assert_eq!(result["two_fa_backup"], false);

        // Login count should increase on a 202 response
        let login_count = ModelLogin::get(&test_setup.postgres, user.registered_user_id).await;
        assert_eq!(login_count.unwrap().unwrap().login_attempt_number, 1);
    }

    #[tokio::test]
    /// When two factor enabled, no token provided, but invalid password supplied, should return a 403 message
    async fn api_router_incognito_signin_post_login_no_token_invalid_password() {
        let mut test_setup = start_both_servers().await;
        test_setup.insert_test_user().await;
        test_setup.insert_two_fa().await;
        let client = reqwest::Client::new();
        let user = test_setup.get_model_user().await.unwrap();

        let url = format!("{}/incognito/signin", base_url(&test_setup.app_env));

        let body = TestSetup::gen_signin_body(None, Some(S!("some_invalid_password")), None, None);

        // Valid login attempt unable to complete
        let result = client.post(&url).json(&body).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            result.json::<Response>().await.unwrap().response,
            "Invalid email address and/or password and/or token"
        );

        // Login count should increase
        let login_count = ModelLogin::get(&test_setup.postgres, user.registered_user_id).await;
        assert_eq!(login_count.unwrap().unwrap().login_attempt_number, 1);
    }

    #[tokio::test]
    /// After one invalid attempt, submit a valid attempt, login_count should now equal = 0
    async fn api_router_incognito_signin_post_with_token_login_attempt_reset() {
        let mut test_setup = start_both_servers().await;
        test_setup.insert_test_user().await;
        test_setup.insert_two_fa().await;
        let client = reqwest::Client::new();
        let valid_token = test_setup.get_valid_token();

        let url = format!("{}/incognito/signin", base_url(&test_setup.app_env));

        let body = TestSetup::gen_signin_body(
            None,
            Some(S!("thisistheincorrectpassword")),
            Some(C!(valid_token)),
            None,
        );

        let result = client.post(&url).json(&body).send().await.unwrap();
        let user = test_setup.get_model_user().await.unwrap();
        let login_count = ModelLogin::get(&test_setup.postgres, user.registered_user_id).await;

        assert_eq!(result.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            result.json::<Response>().await.unwrap().response,
            "Invalid email address and/or password and/or token"
        );
        assert_eq!(login_count.unwrap().unwrap().login_attempt_number, 1);

        let body = TestSetup::gen_signin_body(None, None, Some(valid_token), None);

        // Valid login attempt unable to complete
        let result = client.post(&url).json(&body).send().await.unwrap();
        let user = test_setup.get_model_user().await.unwrap();
        let login_count = ModelLogin::get(&test_setup.postgres, user.registered_user_id).await;

        assert_eq!(result.status(), StatusCode::OK);
        assert_eq!(login_count.unwrap().unwrap().login_attempt_number, 0);
    }

    #[tokio::test]
    /// Invalid login with a backup token
    async fn api_router_incognito_signin_post_backup_token_invalid() {
        let mut test_setup = start_both_servers().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.insert_two_fa().await;

        let client = reqwest::Client::new();
        let url = format!("{}/user/twofa", base_url(&test_setup.app_env),);

        let result = client
            .post(&url)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::OK);

        let url = format!("{}/incognito/signin", base_url(&test_setup.app_env));

        let user = test_setup.get_model_user().await.unwrap();
        assert_eq!(user.two_fa_backup_count, 10);

        // This can fail! Unlikely but not zero
        let token = "519181150EEEAC92";

        let body = TestSetup::gen_signin_body(None, None, Some(token.to_owned()), None);
        let result = client.post(&url).json(&body).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::UNAUTHORIZED);
        let user = test_setup.get_model_user().await.unwrap();
        assert_eq!(user.two_fa_backup_count, 10);
    }

    #[tokio::test]
    /// Valid login with a backup token
    async fn api_router_incognito_signin_post_backup_token_valid() {
        let mut test_setup = start_both_servers().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.insert_two_fa().await;

        let client = reqwest::Client::new();
        let url = format!("{}/user/twofa", base_url(&test_setup.app_env),);

        let result = client
            .post(&url)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::OK);

        let result = result.json::<Response>().await.unwrap().response;
        let codes = result["backups"].as_array().unwrap();

        let url = format!("{}/incognito/signin", base_url(&test_setup.app_env));

        let user = test_setup.get_model_user().await.unwrap();
        assert_eq!(user.two_fa_backup_count, 10);

        let token = codes[4].as_str().unwrap();

        let body = TestSetup::gen_signin_body(None, None, Some(token.to_owned()), None);
        let result = client.post(&url).json(&body).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::OK);
        let user = test_setup.get_model_user().await.unwrap();
        assert_eq!(user.two_fa_backup_count, 9);

        // Using the same backup code again fails
        let result = client.post(&url).json(&body).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::UNAUTHORIZED);
        let user = test_setup.get_model_user().await.unwrap();
        assert_eq!(user.two_fa_backup_count, 9);
    }

    #[tokio::test]
    /// Valid login, session created, cookie returned
    async fn api_router_incognito_signin_post_valid_session() {
        let mut test_setup = start_both_servers().await;
        test_setup.insert_test_user().await;
        let client = reqwest::Client::new();
        let user = test_setup.get_model_user().await.unwrap();

        let url = format!("{}/incognito/signin", base_url(&test_setup.app_env));

        let body = TestSetup::gen_signin_body(None, None, None, None);

        let result = client.post(&url).json(&body).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::OK);

        // Assert cookie is received & correct
        let cookie = result.headers().get("set-cookie");
        assert!(cookie.is_some());

        let cookie = cookie.unwrap();
        assert!(
            cookie
                .to_str()
                .unwrap()
                .contains("HttpOnly; SameSite=Strict; Path=/; Domain=127.0.0.1; Max-Age=21600")
        );

        // Assert session in db
        let session_vec = get_keys(&test_setup.redis, "session::*").await;
        assert_eq!(session_vec.len(), 1);
        let session_name = session_vec.first().unwrap();
        let session: RedisSession = test_setup.redis.hget(session_name, "data").await.unwrap();
        let session_ttl: usize = test_setup.redis.ttl(session_name).await.unwrap();

        assert!(session_ttl > 21598);
        assert!(session_ttl < 21601);
        // and also less than!

        let key = format!(
            "session_set::user::{}",
            test_setup.model_user.as_ref().unwrap().registered_user_id
        );
        let redis_set: Vec<String> = test_setup.redis.smembers(key).await.unwrap();
        assert!(redis_set.len() == 1);

        assert_eq!(session.registered_user_id, user.registered_user_id);
        assert_eq!(session.email, user.email);

        // Assert session in db
        let session_vec = get_keys(&test_setup.redis, "session::*").await;
        assert_eq!(session_vec.len(), 1);
        let session_name = session_vec.first().unwrap();
        let session = test_setup
            .redis
            .hget::<String, &str, &str>(session_name, "data")
            .await
            .unwrap();
        let session_ttl: usize = test_setup.redis.ttl(session_name).await.unwrap();

        let session = serde_json::from_str::<RedisSession>(&session).unwrap();

        assert!(session_ttl > 21598);

        let key = format!(
            "session_set::user::{}",
            test_setup.model_user.as_ref().unwrap().registered_user_id
        );
        let redis_set: Vec<String> = test_setup.redis.smembers(key).await.unwrap();
        assert!(redis_set.len() == 1);

        assert_eq!(session.registered_user_id, user.registered_user_id);
        assert_eq!(session.email, user.email);
    }

    #[tokio::test]
    /// Able to sign in if already signed in, but old session gets destroyed
    /// New session created, previous one destroyed
    async fn api_router_incognito_signin_post_authed_already_authed_valid() {
        let mut test_setup = start_both_servers().await;
        let client = reqwest::Client::new();
        let url = format!("{}/incognito/signin", base_url(&test_setup.app_env));
        let body = TestSetup::gen_signin_body(None, None, None, None);
        let authed_cookie = test_setup.authed_user_cookie().await;

        let key = format!(
            "session_set::user::{}",
            test_setup.model_user.unwrap().registered_user_id
        );
        let pre_set: Vec<String> = test_setup.redis.smembers(&key).await.unwrap();
        assert!(pre_set.len() == 1);

        let result = client
            .post(&url)
            .json(&body)
            .header("cookie", authed_cookie)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::OK);
        let post_set: Vec<String> = test_setup.redis.smembers(key).await.unwrap();

        assert_ne!(pre_set[0], post_set[0]);
        assert!(post_set.len() == 1);
    }

    #[tokio::test]
    /// Authenticated user able to get incognito meals route
    /// Get the food all food (descriptions + person + date) object, check that it gets inserted into redis cache
    async fn api_router_incognito_authed_get_food_ok() {
        let mut test_setup = start_both_servers().await;
        let authed_cookie = test_setup.authed_user_cookie().await;

        let client = reqwest::Client::new();

        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            IncognitoRoutes::Meals.addr()
        );

        let result = client
            .get(url)
            .header("cookie", authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::OK);

        let result = result.json::<Response>().await.unwrap().response;

        let descriptions = result.get("d");
        assert!(descriptions.is_some());
        let descriptions = descriptions.unwrap().as_object().unwrap();

        for (id, item) in descriptions {
            assert!(id.parse::<u64>().is_ok());
            assert!(item.as_str().is_some());
        }

        let categories = result.get("c");
        assert!(categories.is_some());
        let categories = categories.unwrap().as_object().unwrap();

        for (id, item) in categories {
            assert!(id.parse::<u64>().is_ok());
            assert!(item.is_string());
            assert!(
                item.as_str()
                    .unwrap()
                    .replace(' ', "")
                    .chars()
                    .all(char::is_uppercase)
            );
        }

        let meal_dates = result.get("m");
        assert!(meal_dates.is_some());
        let meal_dates = meal_dates.unwrap().as_array().unwrap();

        assert!(meal_dates.len() > 200);
        let mut photo_count = 0;

        for i in meal_dates {
            // assert each has a d, and j object, and a c object, and each j & d object should have a c, and m
            let entry = i.as_object().unwrap();
            let meal_date = entry.get("a");
            assert!(meal_date.is_some());
            let meal_date = meal_date.unwrap();
            assert!(meal_date.is_string());
            assert!(meal_date.as_str().unwrap().chars().count() == 6);
            assert!(meal_date.as_str().unwrap().chars().all(char::is_numeric));

            assert!(entry.get("d").is_none());

            let person = entry.get("j");
            assert!(person.is_some());
            let person = person.unwrap();
            assert!(person.is_object());
            let person = person.as_object().unwrap();
            assert!(person.get("m").is_some());
            assert!(person.get("m").unwrap().is_i64());
            let m_id = person.get("m").unwrap().as_i64().unwrap();
            assert!(descriptions.contains_key(&m_id.to_string()));

            assert!(person.get("c").is_some());
            assert!(person.get("c").unwrap().is_i64());
            let c_id = person.get("c").unwrap().as_i64().unwrap();
            assert!(categories.contains_key(&c_id.to_string()));

            for i in ["v", "t", "r"] {
                if let Some(v) = person.get(i) {
                    assert!(v.as_i64().unwrap() == 1);
                }
            }

            if let Some(p) = person.get("p") {
                assert!(p.is_object());
                let p = p.as_object().unwrap();
                assert!(p.get("o").is_none());
                let converted = p.get("c");
                assert!(converted.is_some());
                let converted = converted.unwrap().as_str().unwrap();
                assert!(converted.ends_with("11.jpg"));
                photo_count += 1;
            }
        }
        assert!(photo_count > 100);

        // Check redis cache
        let redis_cache: Option<String> = test_setup
            .redis
            .hget("cache::jack_meals", "data")
            .await
            .unwrap();
        assert!(redis_cache.is_some());
    }

    #[tokio::test]
    /// Get the food all food (descriptions + person + date) object, check that it gets inserted into redis cache
    async fn api_router_incognito_get_food_ok() {
        let test_setup = start_both_servers().await;

        let client = reqwest::Client::new();

        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            IncognitoRoutes::Meals.addr()
        );

        let result = client.get(url).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::OK);

        let result = result.json::<Response>().await.unwrap().response;

        let descriptions = result.get("d");
        assert!(descriptions.is_some());
        let descriptions = descriptions.unwrap().as_object().unwrap();

        for (id, item) in descriptions {
            assert!(id.parse::<u64>().is_ok());
            assert!(item.as_str().is_some());
        }

        let categories = result.get("c");
        assert!(categories.is_some());
        let categories = categories.unwrap().as_object().unwrap();

        for (id, item) in categories {
            assert!(id.parse::<u64>().is_ok());
            assert!(item.is_string());
            assert!(
                item.as_str()
                    .unwrap()
                    .replace(' ', "")
                    .chars()
                    .all(char::is_uppercase)
            );
        }

        let meal_dates = result.get("m");
        assert!(meal_dates.is_some());
        let meal_dates = meal_dates.unwrap().as_array().unwrap();

        assert!(meal_dates.len() > 200);
        let mut photo_count = 0;

        for i in meal_dates {
            // assert each has a d, and j object, and a c object, and each j & d object should have a c, and m
            let entry = i.as_object().unwrap();
            let meal_date = entry.get("a");
            assert!(meal_date.is_some());
            let meal_date = meal_date.unwrap();
            assert!(meal_date.is_string());
            assert!(meal_date.as_str().unwrap().chars().count() == 6);
            assert!(meal_date.as_str().unwrap().chars().all(char::is_numeric));

            assert!(entry.get("d").is_none());

            let person = entry.get("j");
            assert!(person.is_some());
            let person = person.unwrap();
            assert!(person.is_object());
            let person = person.as_object().unwrap();
            assert!(person.get("m").is_some());
            assert!(person.get("m").unwrap().is_i64());
            let m_id = person.get("m").unwrap().as_i64().unwrap();
            assert!(descriptions.contains_key(&m_id.to_string()));

            assert!(person.get("c").is_some());
            assert!(person.get("c").unwrap().is_i64());
            let c_id = person.get("c").unwrap().as_i64().unwrap();
            assert!(categories.contains_key(&c_id.to_string()));

            for i in ["v", "t", "r"] {
                if let Some(v) = person.get(i) {
                    assert!(v.as_i64().unwrap() == 1);
                }
            }

            if let Some(p) = person.get("p") {
                assert!(p.is_object());
                let p = p.as_object().unwrap();
                assert!(p.get("o").is_none());
                let converted = p.get("c");
                assert!(converted.is_some());
                let converted = converted.unwrap().as_str().unwrap();
                assert!(converted.ends_with("11.jpg"));
                photo_count += 1;
            }
        }
        assert!(photo_count > 100);

        // Check redis cache
        let redis_cache: Option<String> = test_setup
            .redis
            .hget("cache::jack_meals", "data")
            .await
            .unwrap();
        assert!(redis_cache.is_some());
    }

    #[tokio::test]
    /// An authed user is unable to use this hash route, needs to use /food/hash insteaad
    async fn api_router_incognito_hash_auth_ok() {
        let mut test_setup = start_both_servers().await;
        let authed_cookie = test_setup.authed_user_cookie().await;

        let client = reqwest::Client::new();
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            IncognitoRoutes::MealHash.addr()
        );

        let result = client
            .get(url)
            .header("cookie", authed_cookie)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::OK);
        let result = result.json::<Response>().await.unwrap().response;

        assert!(result.is_string());
        let result = result.as_str().unwrap();
        assert!(IncomingDeserializer::is_hex(result, 64));

        // Check redis cache
        let redis_cache: Option<String> = test_setup
            .redis
            .get("cache::jack_meals_hash")
            .await
            .unwrap();
        assert!(redis_cache.is_some());
        assert_eq!(redis_cache.unwrap(), result);
    }

    #[tokio::test]
    /// Get the current hash of all meals, check that it gets inserted into redis cache
    async fn api_router_incognito_hash_unauth_ok() {
        let test_setup = start_both_servers().await;

        let client = reqwest::Client::new();
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            IncognitoRoutes::MealHash.addr()
        );

        let result = client.get(url).send().await.unwrap();

        assert_eq!(result.status(), StatusCode::OK);
        let result = result.json::<Response>().await.unwrap().response;

        assert!(result.is_string());
        let result = result.as_str().unwrap();
        assert!(IncomingDeserializer::is_hex(result, 64));

        // Check redis cache
        let redis_cache: Option<String> = test_setup
            .redis
            .get("cache::jack_meals_hash")
            .await
            .unwrap();
        assert!(redis_cache.is_some());
        assert_eq!(redis_cache.unwrap(), result);
    }
}
