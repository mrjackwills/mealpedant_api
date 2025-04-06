use fred::clients::Pool;
use sqlx::PgPool;
use std::{net::ToSocketAddrs, ops::Deref, time::SystemTime};
use ulid::Ulid;

use axum::{
    extract::{ConnectInfo, FromRef, FromRequestParts, State},
    http::{HeaderMap, Request},
    middleware::Next,
    response::Response,
};
use axum_extra::extract::{PrivateCookieJar, cookie::Key};
use std::{
    net::{IpAddr, SocketAddr},
    sync::Arc,
};
use tokio::signal;
use tracing::info;

pub mod api;
pub mod authentication;
pub mod deserializer;
pub mod static_serve;

use crate::{
    C, S,
    api_error::ApiError,
    database::{RateLimit, backup::BackupEnv},
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

use self::outgoing_json::oj::AsJsonRes;

type Outgoing<T> = (axum::http::StatusCode, AsJsonRes<T>);

// Could have a trait for this, as long as it has a get redis, get postgres, get cookie
#[derive(Clone)]
pub struct ApiState(Arc<InnerApiState>);

/// deref so you can still access the inner fields easily
impl Deref for ApiState {
    type Target = InnerApiState;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ApiState {
    pub fn new(app_env: &AppEnv, postgres: PgPool, redis: Pool) -> Self {
        Self(Arc::new(InnerApiState::new(app_env, postgres, redis)))
    }
}

pub struct InnerApiState {
    pub backup_env: BackupEnv,
    pub email_env: EmailerEnv,
    pub photo_env: PhotoLocationEnv,
    pub location_public: String,
    pub postgres: PgPool,
    pub invite: String,
    pub cookie_name: String,
    pub redis: Pool,
    pub domain: String,
    pub run_mode: RunMode,
    pub start_time: SystemTime,
    cookie_key: Key,
}

impl InnerApiState {
    pub fn new(app_env: &AppEnv, postgres: PgPool, redis: Pool) -> Self {
        Self {
            backup_env: BackupEnv::new(app_env),
            email_env: EmailerEnv::new(app_env),
            photo_env: PhotoLocationEnv::new(app_env),
            postgres,
            location_public: C!(app_env.location_public),
            redis,
            invite: C!(app_env.invite),
            cookie_name: C!(app_env.cookie_name),
            domain: C!(app_env.domain),
            run_mode: app_env.run_mode,
            start_time: app_env.start_time,
            cookie_key: Key::from(&app_env.cookie_secret),
        }
    }
}

impl FromRef<ApiState> for Key {
    fn from_ref(state: &ApiState) -> Self {
        C!(state.0.cookie_key)
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

/// Attempt to extract out an ulid from the cookie jar
pub fn get_cookie_ulid(state: &ApiState, jar: &PrivateCookieJar) -> Option<Ulid> {
    jar.get(&state.cookie_name)
        .and_then(|i| Ulid::from_string(i.value()).ok())
}

/// get a bind-able SocketAddr from the AppEnv
fn get_addr(host: &str, port: u16) -> Result<SocketAddr, ApiError> {
    match (C!(host), port).to_socket_addrs() {
        Ok(i) => {
            let vec_i = i.take(1).collect::<Vec<SocketAddr>>();
            vec_i
                .first()
                .map_or(Err(ApiError::Internal(S!("No addr"))), |addr| Ok(*addr))
        }
        Err(e) => Err(ApiError::Internal(e.to_string())),
    }
}

/// Limit the users request based on ip address, using redis as mem store
async fn rate_limiting(
    State(state): State<ApiState>,
    jar: PrivateCookieJar,
    req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, ApiError> {
    let (mut parts, body) = req.into_parts();
    let addr = ConnectInfo::<SocketAddr>::from_request_parts(&mut parts, &state).await?;
    let ip = get_ip(&parts.headers, &addr);
    let ulid = get_cookie_ulid(&state, &jar);
    RateLimit::check(&state.redis, ip, ulid).await?;
    Ok(next.run(Request::from_parts(parts, body)).await)
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
#[expect(clippy::unwrap_used, clippy::nursery)]
pub mod api_tests {
    use fred::clients::Pool;
    use fred::interfaces::{ClientLike, KeysInterface};
    use fred::types::scan::Scanner;
    use futures::TryStreamExt;
    use jiff::civil::Date;
    use regex::Regex;
    use reqwest::StatusCode;
    use sqlx::PgPool;
    use sqlx::types::ipnetwork::IpNetwork;
    use std::collections::HashMap;
    use std::net::IpAddr;
    use std::net::Ipv4Addr;
    use std::sync::LazyLock;

    use crate::C;
    use crate::S;
    use crate::database::{
        DbRedis, ModelMeal, ModelTwoFA, ModelUser, ModelUserAgentIp, Person, RedisNewUser,
        RedisTwoFASetup, ReqUserAgentIp, db_postgres,
    };
    use crate::helpers::{gen_random_hex, now_utc};
    use crate::parse_env;
    use crate::parse_env::AppEnv;
    use crate::sleep;

    use rand::{Rng, distributions::Alphanumeric};

    use serde::{Deserialize, Serialize};
    use serde_json::Value;

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

    static RATELIMIT_REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new("rate limited for ([5][0-9]|60) seconds").unwrap());

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Response {
        pub response: Value,
    }

    pub struct TestSetup {
        pub app_env: AppEnv,
        pub redis: Pool,
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
            // self.delete_photos();
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
                user_agent: S!("test_user_agent"),
                ip: IpAddr::V4(Ipv4Addr::new(123, 123, 123, 123)),
            }
        }

        /// Generate a meal for tomorrow
        pub fn gen_meal(&mut self, with_photo: bool) -> TestBodyMeal {
            let category = gen_random_hex(10);
            let description = gen_random_hex(24);
            let date = format!("{}", now_utc().tomorrow().unwrap().date());
            let photo_original = if with_photo {
                Some(format!(
                    "{ulid}10.jpg",
                    ulid = ulid::Ulid::new().to_string().to_lowercase(),
                ))
            } else {
                None
            };

            let photo_converted = if with_photo {
                Some(format!(
                    "{ulid}11.jpg",
                    ulid = ulid::Ulid::new().to_string().to_lowercase(),
                ))
            } else {
                None
            };
            let body = TestBodyMeal {
                date,
                category,
                description,
                person: S!("Jack"),
                restaurant: false,
                takeaway: true,
                vegetarian: false,
                photo_original,
                photo_converted,
            };
            self.test_meal = Some(C!(body));
            body
        }

        async fn delete_meal(&mut self) {
            let meal = self.gen_meal(true);
            let person = Person::try_from(meal.person.as_str()).unwrap();
            let date = meal.date.parse::<Date>().unwrap();
            ModelMeal::delete(&self.postgres, &person, date).await.ok();
        }

        pub async fn query_meal(&self) -> Option<ModelMeal> {
            if let Some(meal) = self.test_meal.as_ref() {
                let person = Person::try_from(meal.person.as_str()).unwrap();
                let date = meal.date.parse::<Date>().unwrap();
                ModelMeal::get_by_date_person(&self.postgres, &person, date)
                    .await
                    .unwrap()
            } else {
                None
            }
        }

        /// Delete emails that were written to disk
        pub async fn delete_login_attempts(&self) {
            sqlx::query!("DELETE FROM login_attempt")
                .execute(&self.postgres)
                .await
                .unwrap();
        }

        /// Delete emails that were written to disk
        pub async fn delete_two_fa_secret(&self) {
            if let Some(user) = self.model_user.as_ref() {
                sqlx::query!(
                    "DELETE FROM two_fa_secret WHERE registered_user_id = $1",
                    user.registered_user_id
                )
                .execute(&self.postgres)
                .await
                .unwrap();
            }
        }

        /// Remove test user from postgres
        pub async fn delete_test_user(&self) {
            if let Some(user) = self.model_user.as_ref() {
                sqlx::query!(
                    "DELETE FROM admin_user WHERE registered_user_id = $1",
                    user.registered_user_id
                )
                .execute(&self.postgres)
                .await
                .unwrap();
            }

            if let Some(user) = self.anon_user.as_ref() {
                sqlx::query!(
                    "DELETE FROM admin_user WHERE registered_user_id = $1",
                    user.registered_user_id
                )
                .execute(&self.postgres)
                .await
                .unwrap();
            }

            sqlx::query!(
                "DELETE FROM registered_user WHERE email IN ($1, $2)",
                TEST_EMAIL,
                ANON_EMAIL
            )
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

        /// Delete the useragent and ip from database
        pub async fn delete_useragent_ip(&self) {
            let req = Self::gen_req();
            sqlx::query!(
                "DELETE FROM ip_address WHERE ip = $1",
                IpNetwork::from(req.ip)
            )
            .execute(&self.postgres)
            .await
            .unwrap();

            sqlx::query!(
                "DELETE FROM user_agent WHERE user_agent_string = $1",
                req.user_agent
            )
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
                let req = ModelUserAgentIp::get(&self.postgres, &C!(self.redis), &Self::gen_req())
                    .await
                    .unwrap();
                sqlx::query!(
                    "INSERT INTO admin_user(registered_user_id, ip_id, admin) VALUES ($1, $2, $3)",
                    user.registered_user_id,
                    req.ip_id,
                    true
                )
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

        /// Sign in with the test user, then return the cookie so that other requests can be authenticated
        pub async fn signin_cookie(&mut self) -> String {
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
            sqlx::query_as!(
                P,
                "SELECT password_hash FROM registered_user WHERE email = $1",
                TEST_EMAIL
            )
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
                (S!("full_name"), full_name.to_owned()),
                (S!("password"), password.to_owned()),
                (S!("invite"), invite.to_owned()),
                (S!("email"), email.to_owned()),
            ])
        }
    }

    /// redis KEYS command, but safely using a scanner
    pub async fn get_keys(redis: &Pool, pattern: &str) -> Vec<String> {
        let mut scanner = redis.next().scan(pattern, Some(100), None);
        let mut output = vec![];
        while let Some(mut page) = scanner.try_next().await.unwrap() {
            if let Some(page) = page.take_results() {
                for i in page {
                    output.push(i.as_str().unwrap_or_default().to_owned());
                }
            }
            page.next();
        }
        output
    }

    /// Get basic api params, also flushes all redis keys, deletes all test data, DOESN'T start the api server
    pub async fn setup() -> TestSetup {
        let app_env = parse_env::AppEnv::get_env();
        let postgres = db_postgres::db_pool(&app_env).await.unwrap();
        let redis = DbRedis::get_pool(&app_env).await.unwrap();
        let mut test_setup = TestSetup {
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

    // /// start the static server on it's own thread
    // pub async fn start_static_server() -> TestSetup {
    //     let setup = setup().await;
    //     let app_env = C!(setup.app_env);
    //     let h_r = C!(setup.redis);
    //     let db1 = C!(setup.postgres);

    //     let handle = tokio::spawn(async {
    //         crate::servers::static_serve::StaticRouter::serve(app_env, db1, h_r)
    //             .await
    //             .unwrap();
    //     });

    //     // just sleep to make sure the server is running - 1ms is enough
    //     sleep!(1);

    //     TestSetup {
    //         _handle: Some(handle),
    //         app_env: setup.app_env,
    //         redis: setup.redis,
    //         postgres: setup.postgres,
    //         model_user: None,
    //         test_meal: None,
    //         anon_user: None,
    //     }
    // }

    /// start the api server, and the static server, each on their own thread
    pub async fn start_both_servers() -> TestSetup {
        let setup = setup().await;
        let app_env_api = C!(setup.app_env);
        let redis_api = C!(setup.redis);
        let postgres_api = C!(setup.postgres);

        let app_env_static = C!(setup.app_env);
        let redis_static = C!(setup.redis);
        let postgres_static = C!(setup.postgres);

        tokio::spawn(async {
            crate::servers::api::serve(app_env_api, postgres_api, redis_api)
                .await
                .unwrap();
        });

        tokio::spawn(async {
            crate::servers::static_serve::StaticRouter::serve(
                app_env_static,
                postgres_static,
                redis_static,
            )
            .await
            .unwrap();
        });

        // just sleep to make sure the server is running - 1ms is enough
        sleep!(1);

        TestSetup {
            app_env: setup.app_env,
            redis: setup.redis,
            postgres: setup.postgres,
            model_user: None,
            test_meal: None,
            anon_user: None,
        }
    }

    pub fn base_url(app_env: &AppEnv) -> String {
        format!(
            "http://127.0.0.1:{}{}",
            app_env.api_port,
            crate::servers::api::get_api_version()
        )
    }

    #[test]
    fn http_mod_get_api_version() {
        assert_eq!(crate::servers::api::get_api_version(), S!("/v1"));
    }

    #[tokio::test]
    async fn http_mod_get_unknown() {
        let test_setup = start_both_servers().await;

        let version = crate::servers::api::get_api_version();

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
        let test_setup = start_both_servers().await;

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
        let test_setup = start_both_servers().await;

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
        assert!(RATELIMIT_REGEX.is_match(result.as_str().unwrap()));
    }

    #[tokio::test]
    /// rate limit when using user email address as a key
    async fn http_mod_rate_limit_small_authenticated() {
        let mut test_setup = start_both_servers().await;

        let authed_cookie = test_setup.authed_user_cookie().await;
        let client = reqwest::Client::new();

        let url = format!("{}/incognito/online", base_url(&test_setup.app_env));
        for _ in 1..=499 {
            client
                .get(&url)
                .header("cookie", &authed_cookie)
                .send()
                .await
                .unwrap();
        }
        let rate_keys = get_keys(&test_setup.redis, "ratelimit::email*").await;
        let points: u64 = test_setup.redis.get(&rate_keys[0]).await.unwrap();
        assert_eq!(points, 499);

        // 500th request is fine
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

        // 501st request is rate limited
        let resp = client
            .get(&url)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
        let result = resp.json::<Response>().await.unwrap().response;

        assert!(RATELIMIT_REGEX.is_match(result.as_str().unwrap()));
    }

    #[tokio::test]
    async fn http_mod_rate_limit_big_unauthenticated() {
        let test_setup = start_both_servers().await;

        let url = format!("{}/incognito/online", base_url(&test_setup.app_env));
        for _ in 1..=179 {
            reqwest::get(&url).await.unwrap();
        }

        // 180th request is rate limited for one minute
        let resp = reqwest::get(&url).await.unwrap();
        assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
        let result = resp.json::<Response>().await.unwrap().response;
        assert!(RATELIMIT_REGEX.is_match(result.as_str().unwrap()));

        // 180+ request is rate limited for 300 seconds
        let resp = reqwest::get(&url).await.unwrap();
        assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
        let result = resp.json::<Response>().await.unwrap().response;
        assert_eq!(result, "rate limited for 300 seconds");
    }

    #[tokio::test]
    async fn http_mod_rate_limit_big_authenticated() {
        let mut test_setup = start_both_servers().await;

        let authed_cookie = test_setup.authed_user_cookie().await;
        let client = reqwest::Client::new();

        let url = format!("{}/incognito/online", base_url(&test_setup.app_env));

        for _ in 1..=999 {
            client
                .get(&url)
                .header("cookie", &authed_cookie)
                .send()
                .await
                .unwrap();
        }

        let rate_keys: Vec<String> = get_keys(&test_setup.redis, "ratelimit::email*").await;
        let points: u64 = test_setup.redis.get(&rate_keys[0]).await.unwrap();
        assert_eq!(points, 999);

        // 1000th request is rate limited for 1 minute,
        let resp = client
            .get(&url)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
        let result = resp.json::<Response>().await.unwrap().response;
        assert!(RATELIMIT_REGEX.is_match(result.as_str().unwrap()));

        // 1000+ request is rate limited for 300 seconds
        let resp = client
            .get(&url)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
        let result = resp.json::<Response>().await.unwrap().response;

        assert!(
            Regex::new("rate limited for (29[0-9]|300) seconds")
                .unwrap()
                .is_match(result.as_str().unwrap())
        );
    }
}
