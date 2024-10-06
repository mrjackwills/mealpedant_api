use fred::clients::RedisPool;
use sqlx::PgPool;
use std::{net::ToSocketAddrs, ops::Deref, time::SystemTime};
use tower_http::cors::CorsLayer;
use uuid::Uuid;

use axum::{
    extract::{ConnectInfo, FromRef, FromRequestParts, OriginalUri, State},
    http::{HeaderMap, HeaderValue, Request},
    middleware::{self, Next},
    response::Response,
    Extension, Router,
};
use axum_extra::extract::{cookie::Key, PrivateCookieJar};
use std::{
    net::{IpAddr, SocketAddr},
    sync::Arc,
};
use tokio::signal;
use tower::ServiceBuilder;
use tracing::info;

mod authentication;
mod deserializer;
mod routers;

use crate::{
    api_error::ApiError,
    database::{backup::BackupEnv, RateLimit},
    emailer::EmailerEnv,
    parse_env::{AppEnv, RunMode},
    photo_convertor::PhotoLocationEnv,
};

mod incoming_json;
mod outgoing_json;

pub use incoming_json::ij;
pub use outgoing_json::oj;

const X_REAL_IP: &str = "x-real-ip";
const X_FORWARDED_FOR: &str = "x-forwarded-for";
const USER_AGENT: &str = "user-agent";

use self::{oj::OutgoingJson, outgoing_json::oj::AsJsonRes};

type Outgoing<T> = (axum::http::StatusCode, AsJsonRes<T>);

#[derive(Clone)]
pub struct ApplicationState(Arc<InnerState>);

// deref so you can still access the inner fields easily
impl Deref for ApplicationState {
    type Target = InnerState;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ApplicationState {
    pub fn new(app_env: &AppEnv, postgres: PgPool, redis: RedisPool) -> Self {
        Self(Arc::new(InnerState::new(app_env, postgres, redis)))
    }
}

pub struct InnerState {
    pub backup_env: BackupEnv,
    pub email_env: EmailerEnv,
    pub photo_env: PhotoLocationEnv,
    pub postgres: PgPool,
    pub invite: String,
    pub cookie_name: String,
    pub redis: RedisPool,
    pub domain: String,
    pub run_mode: RunMode,
    pub start_time: SystemTime,
    cookie_key: Key,
}

impl InnerState {
    pub fn new(app_env: &AppEnv, postgres: PgPool, redis: RedisPool) -> Self {
        Self {
            backup_env: BackupEnv::new(app_env),
            email_env: EmailerEnv::new(app_env),
            photo_env: PhotoLocationEnv::new(app_env),
            postgres,
            redis,
            invite: app_env.invite.clone(),
            cookie_name: app_env.cookie_name.clone(),
            domain: app_env.domain.clone(),
            run_mode: app_env.run_mode,
            start_time: app_env.start_time,
            cookie_key: Key::from(&app_env.cookie_secret),
        }
    }
}

impl FromRef<ApplicationState> for Key {
    fn from_ref(state: &ApplicationState) -> Self {
        state.0.cookie_key.clone()
    }
}

/// extract `x-forwarded-for` header
fn x_forwarded_for(headers: &HeaderMap) -> Option<IpAddr> {
    headers
        .get(X_FORWARDED_FOR)
        .and_then(|x| x.to_str().ok())
        .and_then(|s| s.split(',').find_map(|s| s.trim().parse::<IpAddr>().ok()))
}

/// extract the `x-real-ip` header
fn x_real_ip(headers: &HeaderMap) -> Option<IpAddr> {
    headers
        .get(X_REAL_IP)
        .and_then(|x| x.to_str().ok())
        .and_then(|s| s.parse::<IpAddr>().ok())
}

/// Get a users ip address, application should always be behind an nginx reverse proxy
/// so header x-forwarded-for should always be valid, then try x-real-ip
/// if neither headers work, use the optional socket address from axum
/// but if for some nothing works, return ipv4 255.255.255.255
pub fn get_ip(headers: &HeaderMap, addr: &ConnectInfo<SocketAddr>) -> IpAddr {
    x_forwarded_for(headers)
        .or_else(|| x_real_ip(headers))
        .map_or(addr.0.ip(), |ip_addr| ip_addr)
}

/// Extract the user-agent string
pub fn get_user_agent_header(headers: &HeaderMap) -> String {
    headers
        .get(USER_AGENT)
        .and_then(|x| x.to_str().ok())
        .unwrap_or("UNKNOWN")
        .to_owned()
}

/// Limit the users request based on ip address, using redis as mem store
async fn rate_limiting(
    State(state): State<ApplicationState>,
    jar: PrivateCookieJar,
    req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, ApiError> {
    let (mut parts, body) = req.into_parts();
    let addr = ConnectInfo::<SocketAddr>::from_request_parts(&mut parts, &state).await?;
    let ip = get_ip(&parts.headers, &addr);

    let uuid = jar
        .get(&state.cookie_name)
        .and_then(|data| Uuid::parse_str(data.value()).ok());
    RateLimit::check(&state.redis, ip, uuid).await?;
    Ok(next.run(Request::from_parts(parts, body)).await)
}

/// Create a /v[x] prefix for all api routes, where x is the current major version
fn get_api_version() -> String {
    format!(
        "/v{}",
        env!("CARGO_PKG_VERSION")
            .split('.')
            .take(1)
            .collect::<String>()
    )
}

/// return a unknown endpoint response
pub async fn fallback(
    OriginalUri(original_uri): OriginalUri,
) -> (axum::http::StatusCode, AsJsonRes<String>) {
    (
        axum::http::StatusCode::NOT_FOUND,
        OutgoingJson::new(format!("unknown endpoint: {original_uri}")),
    )
}

pub trait ApiRouter {
    fn create_router(state: &ApplicationState) -> Router<ApplicationState>;
    // fn get_prefix() -> &'static str;
}

/// get a bind-able SocketAddr from the AppEnv
fn get_addr(app_env: &AppEnv) -> Result<SocketAddr, ApiError> {
    match (app_env.api_host.clone(), app_env.api_port).to_socket_addrs() {
        Ok(i) => {
            let vec_i = i.take(1).collect::<Vec<SocketAddr>>();
            vec_i
                .first()
                .map_or(Err(ApiError::Internal("No addr".to_string())), |addr| {
                    Ok(*addr)
                })
        }
        Err(e) => Err(ApiError::Internal(e.to_string())),
    }
}

/// Serve the application
pub async fn serve(app_env: AppEnv, postgres: PgPool, redis: RedisPool) -> Result<(), ApiError> {
    let prefix = get_api_version();

    let cors_url = match app_env.run_mode {
        RunMode::Development => String::from("http://127.0.0.1:8002"),
        RunMode::Production => format!("https://www.{}", app_env.domain),
    };

    let cors = CorsLayer::new()
        .allow_methods([
            axum::http::Method::DELETE,
            axum::http::Method::GET,
            axum::http::Method::OPTIONS,
            axum::http::Method::PATCH,
            axum::http::Method::POST,
            axum::http::Method::PUT,
        ])
        .allow_credentials(true)
        .allow_headers(vec![
            axum::http::header::ACCEPT,
            axum::http::header::ACCEPT_LANGUAGE,
            axum::http::header::ACCESS_CONTROL_ALLOW_ORIGIN,
            axum::http::header::AUTHORIZATION,
            axum::http::header::CACHE_CONTROL,
            axum::http::header::CONTENT_LANGUAGE,
            axum::http::header::CONTENT_TYPE,
        ])
        .allow_origin(cors_url.parse::<HeaderValue>().map_err(|i|ApiError::Internal(i.to_string()))?);

    let application_state = ApplicationState::new(&app_env, postgres, redis);

    let key = application_state.cookie_key.clone();

    let api_routes = Router::new()
        .merge(routers::Admin::create_router(&application_state))
        .merge(routers::Food::create_router(&application_state))
        .merge(routers::Incognito::create_router(&application_state))
        .merge(routers::Meal::create_router(&application_state))
        .merge(routers::Photo::create_router(&application_state))
        .merge(routers::User::create_router(&application_state));

    let app = Router::new()
        .nest(&prefix, api_routes)
        .fallback(fallback)
        .with_state(application_state.clone())
        .layer(
            ServiceBuilder::new()
                .layer(cors)
                .layer(Extension(key))
                .layer(middleware::from_fn_with_state(
                    application_state,
                    rate_limiting,
                )),
        );
    let addr = get_addr(&app_env)?;
    info!("starting server @ {addr}{prefix}");

    match axum::serve(
        tokio::net::TcpListener::bind(&addr).await?,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    {
        Ok(()) => Ok(()),
        Err(_) => Err(ApiError::Internal("api_server".to_owned())),
    }
}

#[expect(clippy::expect_used)]
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }

    info!("signal received, starting graceful shutdown",);
}

/// http tests - ran via actual requests to a (local) server
/// cargo watch -q -c -w src/ -x 'test http_mod -- --test-threads=1 --nocapture'
#[cfg(test)]
#[expect(clippy::unwrap_used, clippy::nursery, clippy::large_futures)]
pub mod api_tests {
    use fred::clients::RedisPool;
    use fred::interfaces::ClientLike;
    use fred::interfaces::KeysInterface;
    use fred::types::Scanner;
    use futures::TryStreamExt;
    use reqwest::StatusCode;
    use sqlx::PgPool;
    use std::collections::HashMap;
    use std::net::IpAddr;
    use std::net::Ipv4Addr;
    use time::format_description;
    use time::Date;

    use crate::api::get_api_version;
    use crate::api::serve;
    use crate::database::{
        db_postgres, DbRedis, ModelMeal, ModelTwoFA, ModelUser, ModelUserAgentIp, Person,
        RedisNewUser, RedisTwoFASetup, ReqUserAgentIp,
    };
    use crate::helpers::gen_random_hex;
    use crate::parse_env;
    use crate::parse_env::AppEnv;
    use crate::sleep;

    use rand::{distributions::Alphanumeric, Rng};

    use serde::{Deserialize, Serialize};
    use serde_json::Value;
    use tokio::task::JoinHandle;

    use super::authentication::totp_from_secret;

    #[macro_export]
    macro_rules! tmp_file {
        ($ms:expr) => {
            format!("/ramdrive/mealpedant/{}", $ms)
        };
    }

    pub const TEST_EMAIL: &str = "test_user@email.com";
    pub const TEST_PASSWORD: &str = "N}}2&zwhgUmfVup[g))EmCchQxcu%R~x";
    pub const TEST_PASSWORD_HASH: &str = "$argon2id$v=19$m=4096,t=1,p=1$D/DKFfvJbZOBICD6y/798w$ifr1qDS9aQLyRPT+57ZOKmfUnrju+fbkEpiK6w2ADuo";
    pub const TEST_FULL_NAME: &str = "Test user full name";

    pub const ANON_EMAIL: &str = "anon_user@email.com";
    pub const ANON_PASSWORD: &str = "this_is_the_anon_test_user_password";
    pub const ANON_PASSWORD_HASH: &str = "$argon2id$v=19$m=4096,t=1,p=1$ODYzbGwydnl4YzAwMDAwMA$x0HG3MOFFlMEDQoVNNacku3lj7yx2Mniacytc+ULPxU8GPj+";
    pub const ANON_FULL_NAME: &str = "Anon user full name";

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Response {
        pub response: Value,
    }

    pub struct TestSetup {
        pub _handle: Option<JoinHandle<()>>,
        pub app_env: AppEnv,
        pub redis: RedisPool,
        pub postgres: PgPool,
        pub model_user: Option<ModelUser>,
        pub anon_user: Option<ModelUser>,
        pub test_meal: Option<TestBodyMeal>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct TestBodyMeal {
        pub date: String,
        pub category: String,
        pub description: String,
        pub person: String,
        pub restaurant: bool,
        pub takeaway: bool,
        pub vegetarian: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub photo_original: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub photo_converted: Option<String>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct TestBodyMealPatch {
        pub original_date: String,
        pub meal: TestBodyMeal,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct TestBodySignin {
        pub email: String,
        pub password: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub token: Option<String>,
        pub remember: bool,
    }

    // gen body? store in self? if self.body then delete?

    /// Cleanup the test environment, and close postgres connection
    impl TestSetup {
        async fn clean_up(&mut self) {
            self.delete_two_fa_secret().await;
            self.delete_meal().await;
            // delete admin
            self.delete_test_user().await;
            self.delete_useragent_ip().await;
            self.delete_login_attempts().await;
            Self::delete_emails();
            self.delete_photos();
            self.delete_backups();
            self.flush_redis().await;
        }

        /// Delete all redis keys
        pub async fn flush_redis(&self) {
            self.redis.flushall::<()>(true).await.unwrap();
        }
        /// generate user ip address, user agent, normally done in middleware automatically by server
        pub fn gen_req() -> ReqUserAgentIp {
            ReqUserAgentIp {
                user_agent: String::from("test_user_agent"),
                ip: IpAddr::V4(Ipv4Addr::new(123, 123, 123, 123)),
            }
        }

        pub fn gen_meal(&mut self, with_photo: bool) -> TestBodyMeal {
            let now = time::OffsetDateTime::now_utc();
            let category = gen_random_hex(10);
            let description = gen_random_hex(24);
            let date = format!("{}", now.date());
            let photo_original = if with_photo {
                Some(format!("{date}_J_O_abcdef0123456789.jpg"))
            } else {
                None
            };

            let photo_converted = if with_photo {
                Some(format!("{date}_J_C_abcdef0123456789.jpg"))
            } else {
                None
            };
            let body = TestBodyMeal {
                date,
                category,
                description,
                person: "Jack".to_owned(),
                restaurant: false,
                takeaway: true,
                vegetarian: false,
                photo_original,
                photo_converted,
            };
            self.test_meal = Some(body.clone());
            body
        }

        async fn delete_meal(&mut self) {
            let meal = self.gen_meal(true);
            let person = Person::try_from(meal.person.as_str()).unwrap();
            let format = format_description::parse("[year]-[month]-[day]").unwrap();
            let date = Date::parse(&meal.date, &format).unwrap();
            ModelMeal::delete(&self.postgres, &self.redis, &person, date)
                .await
                .ok();
        }

        pub async fn query_meal(&self) -> Option<ModelMeal> {
            if let Some(meal) = self.test_meal.as_ref() {
                let person = Person::try_from(meal.person.as_str()).unwrap();
                let format = format_description::parse("[year]-[month]-[day]").unwrap();
                let date = Date::parse(&meal.date, &format).unwrap();
                ModelMeal::get(&self.postgres, &person, date).await.unwrap()
            } else {
                None
            }
        }

        /// Delete emails that were written to disk
        pub async fn delete_login_attempts(&self) {
            let query = r"DELETE FROM login_attempt";
            sqlx::query(query).execute(&self.postgres).await.unwrap();
        }

        /// Delete emails that were written to disk
        pub async fn delete_two_fa_secret(&self) {
            if let Some(user) = self.model_user.as_ref() {
                let query = r"DELETE FROM two_fa_secret WHERE registered_user_id = $1";
                sqlx::query(query)
                    .bind(user.registered_user_id)
                    .execute(&self.postgres)
                    .await
                    .unwrap();
            }
        }

        /// Remove test user from postgres
        pub async fn delete_test_user(&self) {
            if let Some(user) = self.model_user.as_ref() {
                let query = r"DELETE FROM admin_user WHERE registered_user_id = $1";
                sqlx::query(query)
                    .bind(user.registered_user_id)
                    .execute(&self.postgres)
                    .await
                    .unwrap();
            }

            if let Some(user) = self.anon_user.as_ref() {
                let query = r"DELETE FROM admin_user WHERE registered_user_id = $1";
                sqlx::query(query)
                    .bind(user.registered_user_id)
                    .execute(&self.postgres)
                    .await
                    .unwrap();
            }

            let query = r"DELETE FROM registered_user WHERE email IN ($1, $2)";
            sqlx::query(query)
                .bind(TEST_EMAIL)
                .bind(ANON_EMAIL)
                .execute(&self.postgres)
                .await
                .unwrap();
        }

        /// Delete emails that were written to disk
        pub fn delete_emails() {
            std::fs::remove_file(tmp_file!("email_headers.txt")).ok();
            std::fs::remove_file(tmp_file!("email_body.txt")).ok();
        }

        pub fn delete_backups(&self) {
            for file in std::fs::read_dir(&self.app_env.location_backup).unwrap() {
                std::fs::remove_file(file.unwrap().path()).unwrap();
            }
        }

        /// Delete all photos - should be on a ram disk for tests
        pub fn delete_photos(&self) {
            let dirs = [
                self.app_env.location_photo_converted.clone(),
                self.app_env.location_photo_original.clone(),
            ];
            for directory in dirs {
                for file in std::fs::read_dir(directory).unwrap() {
                    std::fs::remove_file(file.unwrap().path()).unwrap();
                }
            }
        }

        /// Delete the useragent and ip from database
        pub async fn delete_useragent_ip(&self) {
            let req = Self::gen_req();
            let query = r"DELETE FROM ip_address WHERE ip = $1::inet";
            sqlx::query(query)
                .bind(req.ip.to_string())
                .execute(&self.postgres)
                .await
                .unwrap();

            let query = r"DELETE FROM user_agent WHERE user_agent_string = $1";
            sqlx::query(query)
                .bind(req.user_agent)
                .execute(&self.postgres)
                .await
                .unwrap();
        }

        pub async fn get_model_user(&self) -> Option<ModelUser> {
            ModelUser::get(&self.postgres, TEST_EMAIL).await.unwrap()
        }

        pub async fn get_anon_user(&self) -> Option<ModelUser> {
            ModelUser::get(&self.postgres, ANON_EMAIL).await.unwrap()
        }

        /// Somewhat diry way to insert a new user - uses server & json requests etc
        pub async fn insert_test_user(&mut self) {
            let req = ModelUserAgentIp::get(&self.postgres, &self.redis, &Self::gen_req())
                .await
                .unwrap();

            let new_user = RedisNewUser {
                email: TEST_EMAIL.to_owned(),
                full_name: TEST_FULL_NAME.to_owned(),
                password_hash: TEST_PASSWORD_HASH.to_string(),
                ip_id: req.ip_id,
                user_agent_id: req.user_agent_id,
            };

            ModelUser::insert(&self.postgres, &new_user).await.unwrap();
            self.model_user = self.get_model_user().await;
        }

        /// Insert new anon user, also has twofa
        pub async fn insert_anon_user(&mut self) {
            let req = ModelUserAgentIp::get(&self.postgres, &self.redis, &Self::gen_req())
                .await
                .unwrap();

            let new_user = RedisNewUser {
                email: ANON_EMAIL.to_owned(),
                full_name: ANON_FULL_NAME.to_owned(),
                password_hash: ANON_PASSWORD_HASH.to_string(),
                ip_id: req.ip_id,
                user_agent_id: req.user_agent_id,
            };

            ModelUser::insert(&self.postgres, &new_user).await.unwrap();

            let anon_user = self.get_anon_user().await;

            let secret = gen_random_hex(32);
            let two_fa_setup = RedisTwoFASetup::new(&secret);
            let req = ModelUserAgentIp::get(&self.postgres, &self.redis, &Self::gen_req())
                .await
                .unwrap();
            ModelTwoFA::insert(
                &self.postgres,
                two_fa_setup,
                req,
                anon_user.as_ref().unwrap(),
            )
            .await
            .unwrap();
            self.anon_user = self.get_anon_user().await;
        }

        // Assumes a test user is already in database, then insert a twofa_secret into postgres
        // pub async fn insert_anon_two_fa(&mut self) {}

        pub async fn two_fa_always_required(&mut self, setting: bool) {
            ModelTwoFA::update_always_required(
                &self.postgres,
                setting,
                self.model_user.as_ref().unwrap(),
            )
            .await
            .unwrap();
            self.model_user = self.get_model_user().await;
        }

        // Assumes a test user is already in database, then insert a twofa_secret into postgres
        pub async fn insert_two_fa(&mut self) {
            let secret = gen_random_hex(32);
            let two_fa_setup = RedisTwoFASetup::new(&secret);
            let req = ModelUserAgentIp::get(&self.postgres, &self.redis, &Self::gen_req())
                .await
                .unwrap();
            ModelTwoFA::insert(
                &self.postgres,
                two_fa_setup,
                req,
                self.model_user.as_ref().unwrap(),
            )
            .await
            .unwrap();
            self.model_user = self.get_model_user().await;
        }

        /// turn the test user into an admin
        pub async fn make_user_admin(&self) {
            if let Some(user) = self.model_user.as_ref() {
                let req =
                    ModelUserAgentIp::get(&self.postgres, &self.redis.clone(), &Self::gen_req())
                        .await
                        .unwrap();
                let query =
                    "INSERT INTO admin_user(registered_user_id, ip_id, admin) VALUES ($1, $2, $3)";
                sqlx::query(query)
                    .bind(user.registered_user_id)
                    .bind(req.ip_id)
                    .bind(true)
                    .execute(&self.postgres)
                    .await
                    .unwrap();
            }
        }

        /// Insert a user, and sign in, then return the cookie so that other requests can be authenticated
        pub async fn authed_user_cookie(&mut self) -> String {
            self.insert_test_user().await;
            let client = reqwest::Client::new();
            let url = format!("{}/incognito/signin", base_url(&self.app_env));
            let body = Self::gen_signin_body(None, None, None, None);
            let signin = client.post(&url).json(&body).send().await.unwrap();
            signin
                .headers()
                .get("set-cookie")
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned()
        }

        /// Insert a user, and sign in, then return the cookie so that other requests can be authenticated
        pub async fn anon_user_cookie(&mut self) -> String {
            // Need to get token
            let token = totp_from_secret(
                self.anon_user
                    .as_ref()
                    .unwrap()
                    .two_fa_secret
                    .as_ref()
                    .unwrap(),
            )
            .unwrap()
            .generate_current()
            .unwrap();

            let client = reqwest::Client::new();
            let url = format!("{}/incognito/signin", base_url(&self.app_env));
            let body = Self::gen_signin_body(
                Some(ANON_EMAIL.to_owned()),
                Some(ANON_PASSWORD.to_owned()),
                Some(token),
                None,
            );
            let signin = client.post(&url).json(&body).send().await.unwrap();
            signin
                .headers()
                .get("set-cookie")
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned()
        }

        pub async fn get_password_hash(&self) -> String {
            #[derive(sqlx::FromRow)]
            struct P {
                password_hash: String,
            }
            let query = r"SELECT password_hash FROM registered_user WHERE email = $1";
            sqlx::query_as::<_, P>(query)
                .bind(TEST_EMAIL)
                .fetch_one(&self.postgres)
                .await
                .unwrap()
                .password_hash
        }

        // Generate signin body
        pub fn gen_signin_body(
            email: Option<String>,
            password: Option<String>,
            op_token: Option<String>,
            remember: Option<bool>,
        ) -> TestBodySignin {
            if let Some(token) = op_token {
                TestBodySignin {
                    email: email.unwrap_or_else(|| TEST_EMAIL.to_owned()),
                    password: password.unwrap_or_else(|| TEST_PASSWORD.to_owned()),
                    token: Some(token),
                    remember: remember.unwrap_or(false),
                }
            } else {
                TestBodySignin {
                    email: email.unwrap_or_else(|| TEST_EMAIL.to_owned()),
                    password: password.unwrap_or_else(|| TEST_PASSWORD.to_owned()),
                    token: None,
                    remember: remember.unwrap_or(false),
                }
            }
        }

        pub fn get_invalid_token(&self) -> String {
            totp_from_secret(
                self.model_user
                    .as_ref()
                    .unwrap()
                    .two_fa_secret
                    .as_ref()
                    .unwrap(),
            )
            .unwrap()
            .generate(123_456_789)
        }

        pub fn get_valid_token(&self) -> String {
            totp_from_secret(
                self.model_user
                    .as_ref()
                    .unwrap()
                    .two_fa_secret
                    .as_ref()
                    .unwrap(),
            )
            .unwrap()
            .generate_current()
            .unwrap()
        }

        // Generate register body
        pub fn gen_register_body(
            full_name: &str,
            password: &str,
            invite: &str,
            email: &str,
        ) -> HashMap<String, String> {
            HashMap::from([
                (String::from("full_name"), full_name.to_owned()),
                (String::from("password"), password.to_owned()),
                (String::from("invite"), invite.to_owned()),
                (String::from("email"), email.to_owned()),
            ])
        }
    }

    /// redis KEYS command, but safely using a scanner
    pub async fn get_keys(redis: &RedisPool, pattern: &str) -> Vec<String> {
        let mut scanner = redis.next().scan(pattern, Some(100), None);
        let mut output = vec![];
        while let Some(mut page) = scanner.try_next().await.unwrap() {
            if let Some(page) = page.take_results() {
                for i in page {
                    output.push(i.as_str().unwrap_or_default().to_owned());
                }
            }
            let _ = page.next();
        }
        output
    }

    /// Get basic api params, also flushes all redis keys, deletes all test data, DOESN'T start the api server
    pub async fn setup() -> TestSetup {
        let app_env = parse_env::AppEnv::get_env();
        let postgres = db_postgres::db_pool(&app_env).await.unwrap();
        let redis = DbRedis::get_pool(&app_env).await.unwrap();
        let mut test_setup = TestSetup {
            _handle: None,
            app_env,
            postgres,
            redis,
            model_user: None,
            test_meal: None,
            anon_user: None,
        };
        test_setup.clean_up().await;
        test_setup
    }

    /// start the api server on it's own thread
    pub async fn start_server() -> TestSetup {
        let setup = setup().await;
        let app_env = setup.app_env.clone();
        let h_r = setup.redis.clone();
        let db1 = setup.postgres.clone();

        let handle = tokio::spawn(async {
            serve(app_env, db1, h_r).await.unwrap();
        });

        // just sleep to make sure the server is running - 1ms is enough
        sleep!(1);

        TestSetup {
            _handle: Some(handle),
            app_env: setup.app_env,
            redis: setup.redis,
            postgres: setup.postgres,
            model_user: None,
            test_meal: None,
            anon_user: None,
        }
    }

    pub fn base_url(app_env: &AppEnv) -> String {
        format!("http://127.0.0.1:{}{}", app_env.api_port, get_api_version())
    }

    #[test]
    fn http_mod_get_api_version() {
        assert_eq!(get_api_version(), "/v1".to_owned());
    }

    #[tokio::test]
    async fn http_mod_get_unknown() {
        let test_setup = start_server().await;

        let version = get_api_version();

        let random_route: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(16)
            .map(char::from)
            .collect();
        let url = format!("{}/{random_route}", base_url(&test_setup.app_env));
        let resp = reqwest::get(url).await.unwrap();

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);

        let result = resp.json::<Response>().await.unwrap().response;

        assert_eq!(
            result,
            format!("unknown endpoint: {version}/{random_route}")
        );
    }

    #[tokio::test]
    /// Not rate limited, but points == request made, and ttl correct
    async fn http_mod_rate_limit() {
        let test_setup = start_server().await;

        let url = format!("{}/incognito/online", base_url(&test_setup.app_env));
        // 45
        for _ in 1..=45 {
            reqwest::get(&url).await.unwrap();
        }

        let count: usize = test_setup
            .redis
            .get("ratelimit::ip::127.0.0.1")
            .await
            .unwrap();
        let ttl: usize = test_setup
            .redis
            .ttl("ratelimit::ip::127.0.0.1")
            .await
            .unwrap();
        assert_eq!(count, 45);
        assert!((59..61).contains(&ttl));
    }

    #[tokio::test]
    /// rate limit when using ip as a key
    async fn http_mod_rate_limit_small_unauthenticated() {
        let test_setup = start_server().await;

        let url = format!("{}/incognito/online", base_url(&test_setup.app_env));
        for _ in 1..=89 {
            reqwest::get(&url).await.unwrap();
        }

        // 90th request is fine
        let resp = reqwest::get(&url).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let result = resp.json::<Response>().await.unwrap().response;
        assert_eq!(result["api_version"], env!("CARGO_PKG_VERSION"));
        assert!(result.get("uptime").is_some());

        // 91st request is rate limited
        let resp = reqwest::get(url).await.unwrap();
        assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
        let result = resp.json::<Response>().await.unwrap().response;
        let messages = ["rate limited for 60 seconds", "rate limited for 59 seconds"];
        assert!(messages.contains(&result.as_str().unwrap()));
    }

    #[tokio::test]
    /// rate limit when using user email address as a key
    async fn http_mod_rate_limit_small_authenticated() {
        let mut test_setup = start_server().await;

        let authed_cookie = test_setup.authed_user_cookie().await;
        let client = reqwest::Client::new();

        let url = format!("{}/incognito/online", base_url(&test_setup.app_env));
        for _ in 1..=89 {
            client
                .get(&url)
                .header("cookie", &authed_cookie)
                .send()
                .await
                .unwrap();
        }
        let rate_keys = get_keys(&test_setup.redis, "ratelimit::email*").await;
        let points: u64 = test_setup.redis.get(&rate_keys[0]).await.unwrap();
        assert_eq!(points, 89);

        // 90th request is fine
        let resp = client
            .get(&url)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let result = resp.json::<Response>().await.unwrap().response;
        assert_eq!(result["api_version"], env!("CARGO_PKG_VERSION"));
        assert!(result.get("uptime").is_some());

        // 91st request is rate limited
        let resp = client
            .get(&url)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
        let result = resp.json::<Response>().await.unwrap().response;

        let messages = ["rate limited for 60 seconds", "rate limited for 59 seconds"];
        assert!(messages.contains(&result.as_str().unwrap()));
    }

    #[tokio::test]
    async fn http_mod_rate_limit_big_unauthenticated() {
        let test_setup = start_server().await;

        let url = format!("{}/incognito/online", base_url(&test_setup.app_env));
        for _ in 1..=179 {
            reqwest::get(&url).await.unwrap();
        }

        // 180th request is rate limited for one minute
        let resp = reqwest::get(&url).await.unwrap();
        assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
        let result = resp.json::<Response>().await.unwrap().response;
        let messages = ["rate limited for 60 seconds", "rate limited for 59 seconds"];
        assert!(messages.contains(&result.as_str().unwrap()));

        // 180+ request is rate limited for 300 seconds
        let resp = reqwest::get(&url).await.unwrap();
        assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
        let result = resp.json::<Response>().await.unwrap().response;
        assert_eq!(result, "rate limited for 300 seconds");
    }

    #[tokio::test]
    async fn http_mod_rate_limit_big_authenticated() {
        let mut test_setup = start_server().await;

        let authed_cookie = test_setup.authed_user_cookie().await;
        let client = reqwest::Client::new();

        let url = format!("{}/incognito/online", base_url(&test_setup.app_env));

        for _ in 1..=179 {
            client
                .get(&url)
                .header("cookie", &authed_cookie)
                .send()
                .await
                .unwrap();
        }

        let rate_keys: Vec<String> = get_keys(&test_setup.redis, "ratelimit::email*").await;
        let points: u64 = test_setup.redis.get(&rate_keys[0]).await.unwrap();
        assert_eq!(points, 179);

        // 180th request is rate limited for 1 minute,
        let resp = client
            .get(&url)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
        let result = resp.json::<Response>().await.unwrap().response;
        let messages = ["rate limited for 60 seconds", "rate limited for 59 seconds"];
        assert!(messages.contains(&result.as_str().unwrap()));

        // 180+ request is rate limited for 300 seconds
        let resp = client
            .get(&url)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
        let result = resp.json::<Response>().await.unwrap().response;
        let messages = [
            "rate limited for 300 seconds",
            "rate limited for 299 seconds",
        ];
        assert!(messages.contains(&result.as_str().unwrap()));
    }
}
