use axum_extra::extract::{cookie::{Cookie, SameSite}, PrivateCookieJar};
use cookie::{time::Duration};
use sqlx::PgPool;
use std::fmt;
use uuid::Uuid;

use crate::{
    api::{
        authentication::{authenticate_signin, authenticate_token, not_authenticated},
        deserializer::IncomingDeserializer,
        ij, oj, ApiRouter, ApplicationState, Outgoing,
    },
    api_error::ApiError,
    argon::ArgonHash,
    database::{
        ModelBannedEmail, ModelLogin, ModelPasswordReset, ModelUser, ModelUserAgentIp,
        RedisNewUser, RedisSession,
    },
    emailer::{Email, EmailTemplate},
    helpers::{self, calc_uptime, gen_random_hex, xor},
};
use axum::{
    extract::{Path, State},
    middleware,
    response::IntoResponse,
    routing::{get, post},
    Router,
};

enum IncognitoRoutes {
    Online,
    Register,
    Reset,
    ResetParam,
    Signin,
    VerifyParam,
}

impl IncognitoRoutes {
    fn addr(&self) -> String {
        let route_name = match self {
            Self::Online => "online",
            Self::Register => "register",
            Self::Reset => "reset",
            Self::ResetParam => "reset/:secret",
            Self::Signin => "signin",
            Self::VerifyParam => "verify/:secret",
        };
        format!("/{route_name}")
    }
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
            Self::InviteInvalid => "invite invalid".to_owned(),
            Self::UnsafePassword => "unsafe password".to_owned(),
            Self::Verified => "Account verified, please sign in to continue".to_owned(),
            Self::VerifyInvalid => "Incorrect verification data".to_owned(),
            Self::Instructions => {
                "Instructions have been sent to the email address provided".to_owned()
            }
            Self::ResetPatch => "Password reset complete - please sign in".to_owned(),
        };
        write!(f, "{disp}")
    }
}

pub struct IncognitoRouter;

impl ApiRouter for IncognitoRouter {
    fn get_prefix() -> &'static str {
        "/incognito"
    }

    fn create_router(state: &ApplicationState) -> Router<ApplicationState> {
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
            .layer(middleware::from_fn_with_state(
                state.clone(),
                not_authenticated,
            ))
            .route(&IncognitoRoutes::Signin.addr(), post(Self::signin_post))
            .route(&IncognitoRoutes::Online.addr(), get(Self::get_online))
    }
}

impl IncognitoRouter {
    /// Return a simple online status response
    #[allow(clippy::unused_async)]
    async fn get_online(State(state): State<ApplicationState>) -> impl IntoResponse {
        (
            axum::http::StatusCode::OK,
            oj::OutgoingJson::new(oj::Online {
                uptime: calc_uptime(state.start_time),
                api_version: env!("CARGO_PKG_VERSION").into(),
            }),
        )
    }

    /// Insert a password reset entry, email user the secret link
    /// Always return same response, even if user/email isn't known in database
    async fn reset_post(
        useragent_ip: ModelUserAgentIp,
        State(state): State<ApplicationState>,
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
        State(state): State<ApplicationState>,
        ij::IncomingJson(body): ij::IncomingJson<ij::PasswordToken>,
    ) -> Result<Outgoing<String>, ApiError> {
        if let Some(reset_user) =
            ModelPasswordReset::get_by_secret(&state.postgres, &secret).await?
        {
            if let Some(two_fa_secret) = reset_user.two_fa_secret {
                if !authenticate_token(
                    body.token,
                    &state.postgres,
                    &two_fa_secret,
                    reset_user.registered_user_id,
                    reset_user.two_fa_backup_count,
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

            let password_hash = ArgonHash::new(body.password.clone()).await?;

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
        } else {
            Err(ApiError::InvalidValue(
                IncognitoResponse::VerifyInvalid.to_string(),
            ))
        }
    }

    /// check if a given reset string is still valid, and also the two-fa status of the user
    async fn reset_param_get(
        Path(secret): Path<String>,
        State(state): State<ApplicationState>,
    ) -> Result<Outgoing<oj::PasswordReset>, ApiError> {
        if !IncomingDeserializer::is_hex(&secret, 128) {
            return Err(ApiError::InvalidValue(
                IncognitoResponse::VerifyInvalid.to_string(),
            ));
        }
        if let Some(valid_reset) =
            ModelPasswordReset::get_by_secret(&state.postgres, &secret).await?
        {
            let response = oj::PasswordReset {
                two_fa_active: valid_reset.two_fa_secret.is_some(),
                two_fa_backup: valid_reset.two_fa_backup_count > 0,
            };
            Ok((axum::http::StatusCode::OK, oj::OutgoingJson::new(response)))
        } else {
            Err(ApiError::InvalidValue(
                IncognitoResponse::VerifyInvalid.to_string(),
            ))
        }
    }

    /// User gets emailed a link when they sign up, they hit this route and it verifies the email address
    /// and insert the new user into postgres
    async fn verify_param_get(
        Path(secret): Path<String>,
        State(state): State<ApplicationState>,
    ) -> Result<Outgoing<String>, ApiError> {
        if !IncomingDeserializer::is_hex(&secret, 128) {
            return Err(ApiError::InvalidValue(
                IncognitoResponse::VerifyInvalid.to_string(),
            ));
        }

        if let Some(new_user) = RedisNewUser::get(&state.redis, &secret).await? {
            ModelUser::insert(&state.postgres, &new_user).await?;
            RedisNewUser::delete(&new_user, &state.redis, &secret).await?;
            Ok((
                axum::http::StatusCode::OK,
                oj::OutgoingJson::new(IncognitoResponse::Verified.to_string()),
            ))
        } else {
            Err(ApiError::InvalidValue(
                IncognitoResponse::VerifyInvalid.to_string(),
            ))
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
        State(state): State<ApplicationState>,
        useragent_ip: ModelUserAgentIp,
        jar: PrivateCookieJar,
        ij::IncomingJson(body): ij::IncomingJson<ij::Signin>,
    ) -> Result<impl IntoResponse, ApiError> {
        // If front end and back end out of sync, and front end user has an api cookie, but not front-end authed, delete server cookie api session
        if let Some(data) = jar.get(&state.cookie_name) {
            RedisSession::delete(&state.redis, &Uuid::parse_str(data.value())?).await?;
        }

        if let Some(user) = ModelUser::get(&state.postgres, &body.email).await? {
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
            // Check password before 2fa token request?

            // If twofa token required, but not sent, 202 response
            if user.two_fa_secret.is_some() && body.token.is_none() {
                // Should this increase the login count?, yes
                ModelLogin::insert(
                    &state.postgres,
                    user.registered_user_id,
                    useragent_ip,
                    false,
                    None,
                )
                .await?;
                // Think I should throw an error here
                // So that the function return type can be strict
                // need to inclued two_backup as a bool
                return Ok((
                    axum::http::StatusCode::ACCEPTED,
                    oj::OutgoingJson::new(oj::SigninAccepted {
                        two_fa_backup: user.two_fa_backup_count > 0,
                    }),
                )
                    .into_response());
            }

            if !authenticate_signin(&user, &body.password, body.token, &state.postgres).await? {
                return Err(Self::invalid_signin(
                    &state.postgres,
                    user.registered_user_id,
                    useragent_ip,
                )
                .await?);
            }

            let uuid = Uuid::new_v4();
            ModelLogin::insert(
                &state.postgres,
                user.registered_user_id,
                useragent_ip,
                true,
                Some(uuid),
            )
            .await?;

            let ttl = if body.remember {
                Duration::days(7 * 4 * 6)
            } else {
                Duration::hours(6)
            };

            let cookie = Cookie::build(state.cookie_name, uuid.to_string())
                .domain(state.domain)
                .path("/")
                .secure(state.run_mode.is_production())
                .same_site(SameSite::Strict)
                .http_only(true)
                .max_age(ttl)
                .finish();

            RedisSession::new(user.registered_user_id, &user.email)
                .insert(&state.redis, ttl, uuid)
                .await?;
            Ok(jar.add(cookie).into_response())
        } else {
            // No known user
            // Add an artifical delay? Of between 500ms and 1500ms?
            Err(ApiError::Authorization)
        }
    }

    async fn register_post(
        State(state): State<ApplicationState>,
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

        // If email address can be found in redis verify cache, or postgres propper, just return a success response
        // Shouldn't even let a client know if a user is registerd to not
        let (redis_user, postgres_user) = tokio::try_join!(
            RedisNewUser::exists(&state.redis, &body.email),
            ModelUser::get(&state.postgres, &body.email)
        )?;

        if redis_user || postgres_user.is_some() {
            return Ok(response);
        }

        let password_hash = ArgonHash::new(body.password.clone()).await?;
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

/// Use reqwest to test agains real server
/// cargo watch -q -c -w src/ -x 'test api_router_incognito -- --test-threads=1 --nocapture'
#[cfg(test)]
#[allow(clippy::pedantic, clippy::nursery, clippy::unwrap_used)]
mod tests {

    use crate::api::api_tests::{
        base_url, sleep, start_server, Response, TestSetup, TEST_EMAIL, TEST_PASSWORD,
        TEST_PASSWORD_HASH,
    };
    use crate::database::{ModelLogin, ModelPasswordReset, RedisNewUser, RedisSession};
    use crate::helpers::gen_random_hex;
    use crate::parse_env::AppEnv;

    use redis::AsyncCommands;
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
        let test_setup = start_server().await;
        let url = format!("{}/incognito/online", base_url(&test_setup.app_env));
        sleep(1000).await;
        let resp = reqwest::get(url).await.unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let result = resp.json::<Response>().await.unwrap().response;
        assert_eq!(result["api_version"], env!("CARGO_PKG_VERSION"));
        assert_eq!(result["uptime"], 1);
    }

    #[tokio::test]
    async fn api_router_incognito_get_online_when_authenticated() {
        let mut test_setup = start_server().await;
        let url = format!("{}/incognito/online", base_url(&test_setup.app_env));
        let client = reqwest::Client::new();
        sleep(1000).await;

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
        let test_setup = start_server().await;
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
        let test_setup = start_server().await;
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
        let test_setup = start_server().await;
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
        let test_setup = start_server().await;
        let client = reqwest::Client::new();
        let url = format!("{}/incognito/register", base_url(&test_setup.app_env));

        let body = TestSetup::gen_register_body(
            "name",
            "superman1234",
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
        let mut test_setup = start_server().await;
        let client = reqwest::Client::new();
        let url = format!("{}/incognito/register", base_url(&test_setup.app_env));

        test_setup.insert_test_user().await;
        // delete_emails();

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
        let result = std::fs::metadata("/dev/shm/email_headers.txt");
        assert!(result.is_err());
        let result = std::fs::metadata("/dev/shm/email_body.txt");
        assert!(result.is_err());
    }

    #[tokio::test]
    /// If authenticated, unable to access register endpoint
    async fn api_router_incognito_register_already_authenticated() {
        let mut test_setup = start_server().await;
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
        let test_setup = start_server().await;
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
        let result = std::fs::metadata("/dev/shm/email_headers.txt");
        assert!(result.is_ok());
        let result = std::fs::metadata("/dev/shm/email_body.txt");
        assert!(result.is_ok());
        let link = format!(
            "href=\"https://www.{}/user/verify/",
            test_setup.app_env.domain
        );
        assert!(std::fs::read_to_string("/dev/shm/email_body.txt")
            .unwrap()
            .contains(&link));
    }

    #[tokio::test]
    async fn api_router_incognito_register_register_twice() {
        let test_setup = start_server().await;
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
        let result = std::fs::metadata("/dev/shm/email_headers.txt");
        assert!(result.is_ok());
        let result = std::fs::metadata("/dev/shm/email_body.txt");
        assert!(result.is_ok());
        let link = format!(
            "href=\"https://www.{}/user/verify/",
            test_setup.app_env.domain
        );
        assert!(std::fs::read_to_string("/dev/shm/email_body.txt")
            .unwrap()
            .contains(&link));

        let first_secret: Vec<String> = test_setup
            .redis
            .lock()
            .await
            .keys("verify::secret::*")
            .await
            .unwrap();

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
        let result = std::fs::metadata("/dev/shm/email_headers.txt");
        assert!(result.is_err());
        let result = std::fs::metadata("/dev/shm/email_body.txt");
        assert!(result.is_err());

        let second_secret: Vec<String> = test_setup
            .redis
            .lock()
            .await
            .keys("verify::secret::*")
            .await
            .unwrap();
        assert_eq!(first_secret, second_secret);
    }

    #[tokio::test]
    async fn api_router_incognito_register_then_verify_ok() {
        let test_setup = start_server().await;
        let client = reqwest::Client::new();
        let url = format!("{}/incognito/register", base_url(&test_setup.app_env));

        let body = TestSetup::gen_register_body(
            "name",
            TEST_PASSWORD,
            &test_setup.app_env.invite,
            TEST_EMAIL,
        );
        client.post(&url).json(&body).send().await.unwrap();
        let secret: Vec<String> = test_setup
            .redis
            .lock()
            .await
            .keys("verify::secret::*")
            .await
            .unwrap();
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
        let test_setup = start_server().await;
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
        let result = std::fs::metadata("/dev/shm/email_headers.txt");
        assert!(result.is_err());
        let result = std::fs::metadata("/dev/shm/email_body.txt");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn api_router_incognito_reset_post_known_user() {
        let mut test_setup = start_server().await;
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
        let result = std::fs::read_to_string("/dev/shm/email_headers.txt");
        assert!(result.is_ok());
        assert!(result
            .unwrap()
            .contains("Subject: Password Reset Requested"));

        let result = std::fs::read_to_string("/dev/shm/email_body.txt");
        assert!(result.is_ok());
        assert!(result.unwrap().contains(&password_reset.reset_string));
    }

    #[tokio::test]
    async fn api_router_incognito_reset_post_known_user_second_attempt() {
        // setup
        let mut test_setup = start_server().await;
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

        let result = std::fs::metadata("/dev/shm/email_headers.txt");
        assert!(result.is_err());
        let result = std::fs::metadata("/dev/shm/email_body.txt");
        assert!(result.is_err());
    }

    #[tokio::test]
    /// If authenticated, unable to access reset_post endpoint
    async fn api_router_incognito_reset_post_already_authenticated() {
        let mut test_setup = start_server().await;
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
        let mut test_setup = start_server().await;
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
        let mut test_setup = start_server().await;
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
        let mut test_setup = start_server().await;
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
        let mut test_setup = start_server().await;
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
        let mut test_setup = start_server().await;
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
    /// Secrect param incorrect
    async fn api_router_incognito_reset_patch_invalid_secret() {
        let mut test_setup = start_server().await;
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
        let mut test_setup = start_server().await;
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
        let mut test_setup = start_server().await;
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
    // invalid token
    async fn api_router_incognito_reset_patch_invalid_token() {
        let mut test_setup = start_server().await;
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
        let mut test_setup = start_server().await;
        test_setup.insert_test_user().await;
        let reset_secret = request_reset(&test_setup.app_env, &test_setup.postgres).await;
        let client = reqwest::Client::new();

        // password in hibp
        let url = format!(
            "{}/incognito/reset/{}",
            base_url(&test_setup.app_env),
            reset_secret
        );
        let body = HashMap::from([("password", "iloveyou1234")]);
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
        let mut test_setup = start_server().await;
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
        let mut test_setup = start_server().await;
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
        let mut test_setup = start_server().await;
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
        let test_setup = start_server().await;
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
        let mut test_setup = start_server().await;
        test_setup.insert_test_user().await;
        let client = reqwest::Client::new();
        let url = format!("{}/incognito/signin", base_url(&test_setup.app_env));
        let body = TestSetup::gen_signin_body(
            None,
            Some("thisistheincorrectpassword".to_owned()),
            None,
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
    }

    #[tokio::test]
    /// invalid login - bad token, login attempt counter increased by one
    async fn api_router_incognito_signin_post_login_bad_token_attempt_increase() {
        let mut test_setup = start_server().await;
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
        let mut test_setup = start_server().await;
        test_setup.insert_test_user().await;
        let client = reqwest::Client::new();

        let url = format!("{}/incognito/signin", base_url(&test_setup.app_env));

        let body = TestSetup::gen_signin_body(
            None,
            Some("thisistheincorrectpassword".to_owned()),
            None,
            None,
        );

        for _ in 0..=19 {
            client.post(&url).json(&body).send().await.unwrap();
        }
        // sleep(100).await;

        let result = std::fs::read_to_string("/dev/shm/email_headers.txt");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Subject: Security Alert"));
        let result = std::fs::read_to_string("/dev/shm/email_body.txt");
        assert!(result.is_ok());
        assert!(result
            .unwrap()
            .contains("Due to multiple failed login attempts your account has been locked."));

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
        let mut test_setup = start_server().await;
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
        let mut test_setup = start_server().await;
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
    /// After one invalid attempt, submit a valid attempt, login_count should now equal = 0
    async fn api_router_incognito_signin_post_with_token_login_attempt_reset() {
        let mut test_setup = start_server().await;
        test_setup.insert_test_user().await;
        test_setup.insert_two_fa().await;
        let client = reqwest::Client::new();
        let valid_token = test_setup.get_valid_token();

        let url = format!("{}/incognito/signin", base_url(&test_setup.app_env));

        let body = TestSetup::gen_signin_body(
            None,
            Some("thisistheincorrectpassword".to_owned()),
            Some(valid_token.clone()),
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
    /// Valid login, session created, cookie returned
    async fn api_router_incognito_signin_post_valid_session() {
        let mut test_setup = start_server().await;
        test_setup.insert_test_user().await;
        let client = reqwest::Client::new();
        let user = test_setup.get_model_user().await.unwrap();

        let url = format!("{}/incognito/signin", base_url(&test_setup.app_env));

        let body = TestSetup::gen_signin_body(None, None, None, None);

        let result = client.post(&url).json(&body).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::OK);

        // Assert cookie is recieved & correct
        let cookie = result.headers().get("set-cookie");
        assert!(cookie.is_some());

        let cookie = cookie.unwrap();
        assert!(cookie
            .to_str()
            .unwrap()
            .contains("HttpOnly; SameSite=Strict; Path=/; Domain=127.0.0.1; Max-Age=21600"));

        // Assert session in db
        let session_vec: Vec<String> = test_setup
            .redis
            .lock()
            .await
            .keys("session::*")
            .await
            .unwrap();
        assert_eq!(session_vec.len(), 1);
        let session_name = session_vec.get(0).unwrap();
        let session: RedisSession = test_setup
            .redis
            .lock()
            .await
            .hget(session_name, "data")
            .await
            .unwrap();
        let session_ttl: usize = test_setup
            .redis
            .lock()
            .await
            .ttl(session_name)
            .await
            .unwrap();

        assert!(session_ttl > 21598);

        let key = format!(
            "session_set::user::{}",
            test_setup.model_user.unwrap().registered_user_id
        );
        let redis_set: Vec<String> = test_setup.redis.lock().await.smembers(key).await.unwrap();
        assert!(redis_set.len() == 1);

        assert_eq!(session.registered_user_id, user.registered_user_id);
        assert_eq!(session.email, user.email);
    }

    #[tokio::test]
    /// Able to sign in if already signed in, but old session gets destroyed
    /// New session created, previous one destroyed
    async fn api_router_incognito_signin_post_authed_already_authed_valid() {
        let mut test_setup = start_server().await;
        let client = reqwest::Client::new();
        let url = format!("{}/incognito/signin", base_url(&test_setup.app_env));
        let body = TestSetup::gen_signin_body(None, None, None, None);
        let authed_cookie = test_setup.authed_user_cookie().await;

        let key = format!(
            "session_set::user::{}",
            test_setup.model_user.unwrap().registered_user_id
        );
        let pre_set: Vec<String> = test_setup.redis.lock().await.smembers(&key).await.unwrap();
        assert!(pre_set.len() == 1);

        let result = client
            .post(&url)
            .json(&body)
            .header("cookie", authed_cookie)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::OK);
        let post_set: Vec<String> = test_setup.redis.lock().await.smembers(key).await.unwrap();

        assert_ne!(pre_set[0], post_set[0]);
        assert!(post_set.len() == 1);
    }
}
