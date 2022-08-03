use ::cookie::Key;
// use ::cookie::Key;
use redis::aio::Connection;
use reqwest::Method;

use sqlx::PgPool;
use std::{net::ToSocketAddrs, time::SystemTime};
use tower_http::{cors::CorsLayer, limit::RequestBodyLimitLayer};
use uuid::Uuid;

use axum::{
    extract::{ConnectInfo, RequestParts},
    handler::Handler,
    http::{Extensions, HeaderMap, HeaderValue, Request},
    middleware::{self, Next},
    response::Response,
    Extension, Router,
};
use axum_extra::extract::PrivateCookieJar;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};
use tokio::{sync::Mutex, signal};
use tower::ServiceBuilder;
use tracing::info;

mod authentication;
mod deserializer;
mod routers;

use crate::{
    api_error::ApiError,
    database::{backup::BackupEnv, RateLimit},
    emailer::EmailerEnv,
    parse_env::AppEnv,
    photo_convertor::PhotoEnv,
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
pub struct ApplicationState {
    pub backup_env: BackupEnv,
    pub email_env: EmailerEnv,
    pub photo_env: PhotoEnv,
    pub postgres: PgPool,
    pub redis: Arc<Mutex<Connection>>,
    pub invite: String,
    pub cookie_name: String,
    pub domain: String,
    pub production: bool,
    pub start_time: SystemTime,
}

impl ApplicationState {
    pub fn new(postgres: PgPool, redis: Arc<Mutex<Connection>>, app_env: &AppEnv) -> Self {
        Self {
            backup_env: BackupEnv::new(app_env),
            email_env: EmailerEnv::new(app_env),
            photo_env: PhotoEnv::new(app_env),
            postgres,
            redis,
            invite: app_env.invite.clone(),
            cookie_name: app_env.cookie_name.clone(),
            domain: app_env.domain.clone(),
            production: app_env.production,
            start_time: app_env.start_time,
        }
    }
}

pub fn get_state(extensions: &Extensions) -> Result<ApplicationState, ApiError> {
    match extensions.get::<ApplicationState>() {
        Some(data) => Ok(data.clone()),
        None => Err(ApiError::Internal(String::from("application_state"))),
    }
}

/// extract `x-forwared-for` header
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
pub fn get_ip(headers: &HeaderMap, addr: Option<&ConnectInfo<SocketAddr>>) -> IpAddr {
    if let Some(ip_addr) = x_forwarded_for(headers).or_else(|| x_real_ip(headers)) {
        ip_addr
    } else if let Some(ip) = addr {
        ip.0.ip()
    } else {
        IpAddr::V4(Ipv4Addr::new(255, 255, 255, 255))
    }
}

/// Extract the user-agent string
pub fn get_user_agent_header(headers: &HeaderMap) -> String {
    headers
        .get(USER_AGENT)
        .and_then(|x| x.to_str().ok())
        .unwrap_or("UNKNOWN")
        .to_owned()
}

// "Extension of type `cookie::secure::key::Key` was not found. Perhaps you forgot to add it? See `axum::Extension

// Limit the users request based on ip address, using redis as mem store
async fn rate_limiting<B: std::marker::Send>(
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, ApiError> {
    let state = get_state(req.extensions())?;
    let addr: Option<&ConnectInfo<SocketAddr>> = req.extensions().get();
    let ip = get_ip(req.headers(), addr);
    let mut parts = RequestParts::new(req);
    let mut uuid = None;

    if let Ok(jar) = parts.extract::<PrivateCookieJar<Key>>().await {
        if let Some(data) = jar.get(&state.cookie_name) {
            uuid = Some(Uuid::parse_str(data.value())?);
        }
    }

    RateLimit::check(&state.redis, ip, uuid).await?;
    let req = parts.try_into_request()?;
    Ok(next.run(req).await)
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

#[allow(clippy::unused_async)]
/// return a unknown endpoint response
async fn fallback(uri: axum::http::Uri) -> (axum::http::StatusCode, AsJsonRes<String>) {
    (
        axum::http::StatusCode::NOT_FOUND,
        OutgoingJson::new(format!("unknown endpoint: {}", uri)),
    )
}

pub trait ApiRouter<T> {
    fn create_router() -> Router<T>;
}

/// Serve the application
pub async fn serve(
    app_env: AppEnv,
    postgres: PgPool,
    redis: Arc<Mutex<Connection>>,
) -> Result<(), ApiError> {
    let prefix = get_api_version();

    let cors_url = if app_env.production {
        format!("https://www.{}", app_env.domain)
    } else {
        String::from("http://127.0.0.1:8002")
    };

    // let cors_url = if app_env.production {
    // 	[

    // 		format!("https://www.{}", app_env.domain).parse().unwrap(),
    // 		format!("https://static.{}", app_env.domain).parse().unwrap()
    // 	]
    // } else {
    //     [
    // 		String::from("http://127.0.0.1:8002").parse().unwrap(),
    // 		String::from("http://127.0.0.1:8002").parse().unwrap()
    // 	]
    // };

    let cookie_key = cookie::Key::from(&app_env.cookie_secret);

	#[allow(clippy::unwrap_used)]
    let cors = CorsLayer::new()
        .allow_methods([
            Method::DELETE,
            Method::GET,
            Method::OPTIONS,
            Method::PATCH,
            Method::POST,
            Method::PUT,
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
        .allow_origin(cors_url.parse::<HeaderValue>().unwrap());

    let application_state = ApplicationState::new(postgres, redis, &app_env);

    let incognito_router = routers::Incognito::create_router();
    let user_router = routers::User::create_router();
    let photo_router = routers::Photo::create_router();
    let food_router = routers::Food::create_router();
    let meal_router = routers::Meal::create_router();
    let admin_router = routers::Admin::create_router();

    let app = Router::new()
        .nest(
            &prefix,
            incognito_router.layer(RequestBodyLimitLayer::new(4096)),
        )
        .nest(&prefix, user_router.layer(RequestBodyLimitLayer::new(4096)))
        .nest(&prefix, food_router.layer(RequestBodyLimitLayer::new(4096)))
        .nest(
            &prefix,
            admin_router.layer(RequestBodyLimitLayer::new(4096)),
        )
        // This size is too small?
        .nest(&prefix, meal_router.layer(RequestBodyLimitLayer::new(4096)))
        .nest(&prefix, photo_router)
        .fallback(fallback.into_service())
        .layer(
            ServiceBuilder::new()
                .layer(cors)
                .layer(Extension(application_state))
                .layer(Extension(cookie_key))
                .layer(middleware::from_fn(rate_limiting)),
        );

    let addr = match (app_env.api_host, app_env.api_port).to_socket_addrs() {
        Ok(i) => {
            let vec_i = i.take(1).collect::<Vec<SocketAddr>>();
            if let Some(addr) = vec_i.get(0) {
                Ok(*addr)
            } else {
                Err(ApiError::Internal("No addr".to_string()))
            }
        }
        Err(e) => Err(ApiError::Internal(e.to_string())),
    }?;

    let starting = format!("starting server @ {}", addr);
    info!(%starting);

    // axum::Server::bind(&addr)
    //     .serve(app.into_make_service_with_connect_info::<SocketAddr>())
    //     .with_graceful_shutdown(signal_shutdown())
    //     .await
    //     .unwrap();

    // Ok(())

	match axum::Server::bind(&addr)
	.serve(app.into_make_service_with_connect_info::<SocketAddr>())
	.with_graceful_shutdown(shutdown_signal())
	.await
{
	Ok(_) => Ok(()),
	Err(_) => Err(ApiError::Internal("api_server".to_owned())),
}

}


#[allow(clippy::expect_used)]
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
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("signal received, starting graceful shutdown");
}


/// http tests - ran via actual requests to a (local) server
/// cargo watch -q -c -w src/ -x 'test http_mod -- --test-threads=1 --nocapture'
#[cfg(test)]
#[allow(clippy::unwrap_used)]
pub mod api_tests {
    use google_authenticator::GoogleAuthenticator;
    use sqlx::PgPool;
    use std::collections::HashMap;
    use std::net::IpAddr;
    use std::net::Ipv4Addr;
    use std::sync::Arc;
    use time::format_description;
    use time::Date;

    use crate::api::get_api_version;
    use crate::api::serve;
    use crate::database::{DbRedis, ModelMeal, ModelTwoFA, ModelUser, ModelUserAgentIp, Person, RedisNewUser, RedisTwoFASetup, ReqUserAgentIp, db_postgres};
    use crate::helpers::gen_random_hex;
    use crate::parse_env;
    use crate::parse_env::AppEnv;

    use rand::{distributions::Alphanumeric, Rng};
    use redis::{aio::Connection, AsyncCommands};
    use reqwest::StatusCode;
    use serde::{Deserialize, Serialize};
    use serde_json::Value;
    use tokio::sync::Mutex;
    use tokio::task::JoinHandle;

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
        pub handle: Option<JoinHandle<()>>,
        pub app_env: AppEnv,
        pub redis: Arc<Mutex<Connection>>,
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
            let all_keys: Vec<String> = self.redis.lock().await.keys("*").await.unwrap();
            for key in all_keys {
                self.redis.lock().await.del::<'_, String, ()>(key).await.unwrap();
            }
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
                Some(format!("{}_J_O_abcdef0123456789.jpg", date))
            } else {
                None
            };

            let photo_converted = if with_photo {
                Some(format!("{}_J_C_abcdef0123456789.jpg", date))
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
            let person = Person::new(&meal.person).unwrap();
            let format = format_description::parse("[year]-[month]-[day]").unwrap();
            let date = Date::parse(&meal.date, &format).unwrap();
            ModelMeal::delete(&self.postgres, &self.redis, &person, date)
                .await
                .unwrap_or(None);
        }

        pub async fn query_meal(&self) -> Option<ModelMeal> {
            if let Some(meal) = self.test_meal.as_ref() {
                let person = Person::new(&meal.person).unwrap();
                let format = format_description::parse("[year]-[month]-[day]").unwrap();
                let date = Date::parse(&meal.date, &format).unwrap();
                ModelMeal::get(&self.postgres, &person, date)
                    .await
                    .unwrap()
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

            let query = r"DELETE FROM registered_user WHERE email = $1 OR EMAIL = $2";
            sqlx::query(query)
                .bind(TEST_EMAIL)
                .bind(ANON_EMAIL)
                .execute(&self.postgres)
                .await
                .unwrap();
        }

        /// Delete emails that were written to disk
        pub fn delete_emails() {
            std::fs::remove_file("/dev/shm/email_headers.txt").unwrap_or(());
            std::fs::remove_file("/dev/shm/email_body.txt").unwrap_or(());
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
            let req = TestSetup::gen_req();
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
            let req = ModelUserAgentIp::get(&self.postgres, &self.redis, &TestSetup::gen_req())
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
            let req = ModelUserAgentIp::get(&self.postgres, &self.redis, &TestSetup::gen_req())
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

            let auth = GoogleAuthenticator::new();
            let secret = auth.create_secret(32);
            let two_fa_setup = RedisTwoFASetup::new(&secret);
            let req = ModelUserAgentIp::get(&self.postgres, &self.redis, &TestSetup::gen_req())
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

        // Assumes a test user is already in database, then insert a twofa_secret into postgres
        pub async fn insert_two_fa(&mut self) {
            let auth = GoogleAuthenticator::new();
            let secret = auth.create_secret(32);
            let two_fa_setup = RedisTwoFASetup::new(&secret);
            let req = ModelUserAgentIp::get(&self.postgres, &self.redis, &TestSetup::gen_req())
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
                let req = ModelUserAgentIp::get(&self.postgres, &self.redis, &TestSetup::gen_req())
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

            let auth = GoogleAuthenticator::new();
            let token = auth
                .get_code(
                    self.anon_user
                        .as_ref()
                        .unwrap()
                        .two_fa_secret
                        .as_ref()
                        .unwrap(),
                    0,
                )
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
            let auth = GoogleAuthenticator::new();
            auth.get_code(
                self.model_user
                    .as_ref()
                    .unwrap()
                    .two_fa_secret
                    .as_ref()
                    .unwrap(),
                123_456_789,
            )
            .unwrap()
        }

        pub fn get_valid_token(&self) -> String {
            let auth = GoogleAuthenticator::new();
            auth.get_code(
                self.model_user
                    .as_ref()
                    .unwrap()
                    .two_fa_secret
                    .as_ref()
                    .unwrap(),
                0,
            )
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

    /// Get basic api params, also flushes all redis keys, deletes all test data, DOESN'T start the api server
    pub async fn setup() -> TestSetup {
        let app_env = parse_env::AppEnv::get_env();
        let postgres = db_postgres::db_pool(&app_env).await.unwrap();
        let redis = Arc::new(Mutex::new(DbRedis::get_connection(&app_env).await.unwrap()));
        let mut test_setup = TestSetup {
            handle: None,
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

    pub async fn sleep(ms: u64) {
        tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
    }

    /// start the api server on it's own thread
    pub async fn start_server() -> TestSetup {
        let setup = setup().await;
        let app_env = setup.app_env.clone();
        let h_r = Arc::clone(&setup.redis);
        let db1 = setup.postgres.clone();

        let handle = tokio::spawn(async {
            serve(app_env, db1, h_r).await.unwrap();
        });

        // just sleep to make sure the server is running - 1ms is enough
        sleep(1).await;

        TestSetup {
            handle: Some(handle),
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
        let url = format!("{}/{}", base_url(&test_setup.app_env), random_route);
        let resp = reqwest::get(url).await.unwrap();

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);

        let result = resp.json::<Response>().await.unwrap().response;

        assert_eq!(
            result,
            format!("unknown endpoint: {}/{}", version, random_route)
        );
    }

    #[tokio::test]
    /// rate limit when using ip as a key
    async fn http_mod_rate_limit_small_unauthenticated() {
        let test_setup = start_server().await;

        let url = format!("{}/incognito/online", base_url(&test_setup.app_env));
        for _ in 0..=88 {
            reqwest::get(&url).await.unwrap();
        }

        // 89 request is fine
        let resp = reqwest::get(&url).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let result = resp.json::<Response>().await.unwrap().response;
        assert_eq!(result["api_version"], env!("CARGO_PKG_VERSION"));
        assert!(result.get("uptime").is_some());

        // 90+ request is rate limited
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
        for _ in 0..=88 {
            client
                .get(&url)
                .header("cookie", &authed_cookie)
                .send()
                .await
                .unwrap();
        }

        let rate_keys: Vec<String> = test_setup
            .redis
            .lock()
            .await
            .keys("ratelimit::email*")
            .await
            .unwrap();
        let points: u64 = test_setup
            .redis
            .lock()
            .await
            .get(&rate_keys[0])
            .await
            .unwrap();
        assert_eq!(points, 89);

        // 89 request is fine
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

        // 90+ request is rate limited
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
        for _ in 0..=178 {
            reqwest::get(&url).await.unwrap();
        }

        // 179th request is rate limited
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

        for _ in 0..=178 {
            client
                .get(&url)
                .header("cookie", &authed_cookie)
                .send()
                .await
                .unwrap();
        }

        let rate_keys: Vec<String> = test_setup
            .redis
            .lock()
            .await
            .keys("ratelimit::email*")
            .await
            .unwrap();
        let points: u64 = test_setup
            .redis
            .lock()
            .await
            .get(&rate_keys[0])
            .await
            .unwrap();
        assert_eq!(points, 179);

        // 179th request is rate limited
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
