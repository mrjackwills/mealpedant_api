use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    middleware,
    response::{AppendHeaders, IntoResponse},
    routing::{delete, get, put},
    Router,
};
use axum_extra::extract::PrivateCookieJar;
use std::time::SystemTime;
use tokio_util::io::ReaderStream;
use uuid::Uuid;

use crate::{
    api::{
        authentication::{authenticate_password_token, is_admin},
        deserializer, ij, oj, ApiRouter, ApplicationState, Outgoing,
    },
    api_error::ApiError,
    database::{
        admin_queries::{self, AllUsers, Session},
        backup::{create_backup, BackupType},
        ModelPasswordReset, ModelUser, ModelUserAgentIp, RateLimit, RedisSession,
    },
    define_routes,
    emailer::{CustomEmail, Email, EmailTemplate},
    helpers::{calc_uptime, gen_random_hex},
};

struct SysInfo {
    virt: usize,
    rss: usize,
    uptime: u64,
    uptime_app: u64,
}

impl SysInfo {
    async fn new(start_time: SystemTime) -> Self {
        // When running in docker, pid should always be 1
        let pid = std::process::id();

        let memory = tokio::fs::read_to_string(format!("/proc/{pid}/statm"))
            .await
            .unwrap_or_default()
            .split(' ')
            .take(2)
            .map(|i| i.parse::<usize>().unwrap_or_default() * 4096)
            .collect::<Vec<_>>();

        let uptime = tokio::fs::read_to_string("/proc/uptime")
            .await
            .unwrap_or_default()
            .split('.')
            .take(1)
            .collect::<String>()
            .parse::<u64>()
            .unwrap_or_default();

        Self {
            virt: *memory.first().unwrap_or(&0),
            rss: *memory.get(1).unwrap_or(&0),
            uptime,
            uptime_app: calc_uptime(start_time),
        }
    }
}

define_routes! {
    AdminRoutes,
    "/admin",
    Base => "",
    Backup => "/backup",
    BackupParam => "/backup/:file_name",
    Email => "/email",
    Limit => "/limit",
    Logs => "/logs",
    Memory => "/memory",
    Restart => "/restart",
    User => "/user",
    SessionParam => "/session/:session_name_or_email"
}

pub struct AdminRouter;

// impl AdminRouter {
impl ApiRouter for AdminRouter {
    fn create_router(state: &ApplicationState) -> Router<ApplicationState> {
        Router::new()
            .route(&AdminRoutes::Base.addr(), get(Self::base_get))
            .route(
                &AdminRoutes::BackupParam.addr(),
                get(Self::backup_param_get),
            )
            .route(
                &AdminRoutes::Backup.addr(),
                delete(Self::backup_delete)
                    .get(Self::backup_get)
                    .post(Self::backup_post),
            )
            .route(
                &AdminRoutes::Email.addr(),
                get(Self::email_get).post(Self::email_post),
            )
            .route(
                &AdminRoutes::Limit.addr(),
                delete(Self::limit_delete).get(Self::limit_get),
            )
            .route(&AdminRoutes::Logs.addr(), get(Self::logs_get))
            .route(&AdminRoutes::Memory.addr(), get(Self::memory_get))
            .route(&AdminRoutes::Restart.addr(), put(Self::restart_put))
            .route(
                &AdminRoutes::SessionParam.addr(),
                delete(Self::session_param_delete).get(Self::session_param_get),
            )
            .route(
                &AdminRoutes::User.addr(),
                get(Self::user_get).patch(Self::user_patch),
            )
            .layer(middleware::from_fn_with_state(state.clone(), is_admin))
    }
}

impl AdminRouter {
    // just return a 200 status code if user is indeed an admin user, handled by is_admin middleware
    #[allow(clippy::unused_async)]
    async fn base_get() -> Result<axum::http::StatusCode, ApiError> {
        Ok(axum::http::StatusCode::OK)
    }

    // Delete a single backup file
    async fn backup_delete(
        State(state): State<ApplicationState>,
        ij::IncomingJson(body): ij::IncomingJson<ij::BackupDelete>,
    ) -> Result<axum::http::StatusCode, ApiError> {
        let backup_path = format!("{}/{}", state.backup_env.location_backup, body.file_name);
        tokio::fs::remove_file(backup_path).await?;
        Ok(axum::http::StatusCode::OK)
    }

    /// Return array of all backup filenames
    async fn backup_get(
        State(state): State<ApplicationState>,
    ) -> Result<Outgoing<oj::Backups>, ApiError> {
        let mut output = vec![];

        let mut backups = tokio::fs::read_dir(&state.backup_env.location_backup).await?;
        while let Some(entry) = backups.next_entry().await? {
            output.push(oj::BackupFile {
                file_name: entry.file_name().into_string().unwrap_or_default(),
                file_size: entry.metadata().await?.len(),
            });
        }
        output.sort_by(|a, b| b.file_name.cmp(&a.file_name));

        Ok((
            axum::http::StatusCode::OK,
            oj::OutgoingJson::new(oj::Backups { backups: output }),
        ))
    }

    /// create a backup, with or without photos
    async fn backup_post(
        State(state): State<ApplicationState>,
        ij::IncomingJson(body): ij::IncomingJson<ij::BackupPost>,
    ) -> Result<axum::http::StatusCode, ApiError> {
        let backup_type = if body.with_photos {
            BackupType::Full
        } else {
            BackupType::SqlOnly
        };
        create_backup(&state.backup_env, backup_type).await?;
        Ok(axum::http::StatusCode::OK)
    }

    /// Download a backup file, could also do it via nginx and an internal request to /admin,
    /// as have done with the photo original/converted static hosting
    async fn backup_param_get(
        State(state): State<ApplicationState>,
        Path(file_name): Path<String>,
    ) -> Result<impl IntoResponse, ApiError> {
        if deserializer::IncomingDeserializer::parse_backup_name(&file_name) {
            let Ok(file) =
                tokio::fs::File::open(format!("{}/{file_name}", state.backup_env.location_backup))
                    .await
            else {
                return Err(ApiError::InvalidValue("backup_name".to_owned()));
            };

            let attach = format!("attachment; filename=\"{file_name}\"");
            let len = format!("{}", file.metadata().await?.len());
            let stream = ReaderStream::new(file);
            let body = Body::from_stream(stream);
            let headers = AppendHeaders([
                (
                    header::CONTENT_TYPE,
                    "application/octet-stream; charset=utf-8",
                ),
                (header::CONTENT_LENGTH, &len),
                (header::CONTENT_DISPOSITION, &attach),
            ]);

            Ok((headers, body).into_response())
        } else {
            Err(ApiError::InvalidValue("backup_name".to_owned()))
        }
    }

    /// Get an array of strings of all current, active, users
    async fn email_get(
        State(state): State<ApplicationState>,
    ) -> Result<Outgoing<Vec<String>>, ApiError> {
        Ok((
            StatusCode::OK,
            oj::OutgoingJson::new(admin_queries::ActiveEmail::get(&state.postgres).await?),
        ))
    }

    /// Send a custom email to an array of users
    async fn email_post(
        State(state): State<ApplicationState>,
        ij::IncomingJson(body): ij::IncomingJson<ij::EmailPost>,
    ) -> Result<axum::http::StatusCode, ApiError> {
        let template = EmailTemplate::Custom(CustomEmail::new(
            body.title,
            body.line_one,
            body.line_two,
            body.link,
            body.button_text,
        ));
        for address in body.emails {
            if let Some(user) = ModelUser::get(&state.postgres, &address).await? {
                Email::new(
                    &user.full_name,
                    &address,
                    template.clone(),
                    &state.email_env,
                )
                .send();
            }
        }
        Ok(StatusCode::OK)
    }

    /// Remove a rate limit count
    async fn limit_delete(
        State(state): State<ApplicationState>,
        ij::IncomingJson(body): ij::IncomingJson<ij::LimitDelete>,
    ) -> Result<axum::http::StatusCode, ApiError> {
        RateLimit::delete(body.key, &state.redis).await?;
        Ok(StatusCode::OK)
    }

    /// Get all rate limits, are either ip or user_id based
    async fn limit_get(
        State(state): State<ApplicationState>,
    ) -> Result<Outgoing<Vec<oj::Limit>>, ApiError> {
        Ok((
            StatusCode::OK,
            oj::OutgoingJson::new(RateLimit::get_all(&state.redis).await?),
        ))
    }

    /// Read log file and send as a giant array - probably stupid/ineffcient
    async fn logs_get(
        State(state): State<ApplicationState>,
    ) -> Result<Outgoing<Vec<oj::Logs>>, ApiError> {
        let logs = tokio::fs::read_to_string(format!("{}/api.log", state.backup_env.location_logs))
            .await?;
        let output = logs
            .lines()
            .rev()
            .filter_map(|i| serde_json::from_str::<oj::Logs>(i).ok())
            .collect::<Vec<_>>();

        Ok((StatusCode::OK, oj::OutgoingJson::new(output)))
    }

    /// Get server info, uptime, app uptime, virt mem, and rss memory
    async fn memory_get(
        State(state): State<ApplicationState>,
    ) -> Result<Outgoing<oj::AdminMemory>, ApiError> {
        let sysinfo = SysInfo::new(state.start_time).await;
        Ok((
            StatusCode::OK,
            oj::OutgoingJson::new(oj::AdminMemory {
                uptime: sysinfo.uptime,
                uptime_app: sysinfo.uptime_app,
                virt: sysinfo.virt,
                rss: sysinfo.rss,
            }),
        ))
    }

    /// Force restart by closing the application with status code 0, this assumes it's running in an auto-restart environment
    /// For testing just return status code 200, don't process:exit
    async fn restart_put(
        State(state): State<ApplicationState>,
        user: ModelUser,
        ij::IncomingJson(body): ij::IncomingJson<ij::PasswordToken>,
    ) -> Result<axum::http::StatusCode, ApiError> {
        if !authenticate_password_token(&user, &body.password, body.token, &state.postgres).await? {
            return Err(ApiError::Authorization);
        }
        if cfg!(not(test)) {
            // This is broken?
            std::process::exit(0);
        }
        // Replace this once never type is in std
        // This will never get reach in production code
        Ok(StatusCode::OK)
    }

    // Delete a user session
    async fn session_param_delete(
        State(state): State<ApplicationState>,
        jar: PrivateCookieJar,
        Path(session): Path<String>,
        // Can move into a json, then use is::uuid on it?
        // TODO use is::uuid on this
    ) -> Result<axum::http::StatusCode, ApiError> {
        if let Ok(uuid) = Uuid::parse_str(&session) {
            let session = jar.get(&state.cookie_name).map(|i| i.value().to_owned());

            if let Ok(s_uuid) = Uuid::parse_str(&session.unwrap_or_default()) {
                if s_uuid == uuid {
                    return Err(ApiError::InvalidValue(
                        "can't remove current session".to_owned(),
                    ));
                }
            }
            RedisSession::delete(&state.redis, &uuid).await?;
            Ok(StatusCode::OK)
        } else {
            Err(ApiError::InvalidValue("uuid".to_owned()))
        }
    }

    /// Get all sessions for a given email address
    async fn session_param_get(
        // TODO use is::email on this
        State(state): State<ApplicationState>,
        jar: PrivateCookieJar,
        Path(email): Path<String>,
    ) -> Result<Outgoing<Vec<Session>>, ApiError> {
        let current_session_uuid = jar.get(&state.cookie_name).map(|i| i.value().to_owned());
        Ok((
            StatusCode::OK,
            oj::OutgoingJson::new(
                admin_queries::Session::get(
                    &email,
                    &state.redis,
                    &state.postgres,
                    current_session_uuid,
                )
                .await?,
            ),
        ))
    }

    /// Get big array of users
    async fn user_get(
        State(state): State<ApplicationState>,
    ) -> Result<Outgoing<Vec<AllUsers>>, ApiError> {
        Ok((
            StatusCode::OK,
            oj::OutgoingJson::new(AllUsers::get(&state.postgres).await?),
        ))
    }

    /// Update a single user entry
    async fn user_patch(
        State(state): State<ApplicationState>,
        useragent_ip: ModelUserAgentIp,
        user: ModelUser,
        ij::IncomingJson(body): ij::IncomingJson<ij::AdminUserPatch>,
    ) -> Result<axum::http::StatusCode, ApiError> {
        if let Some(patch_user) = admin_queries::User::get(&state.postgres, &body.email).await? {
            if patch_user.registered_user_id == user.registered_user_id {
                return Err(ApiError::InvalidValue("can't edit self".to_owned()));
            }

            if let Some(active) = body.patch.active {
                // remove all sessions
                RedisSession::delete_all(&state.redis, patch_user.registered_user_id).await?;
                admin_queries::update_active(
                    &state.postgres,
                    active,
                    patch_user.registered_user_id,
                )
                .await?;
            } else if body.patch.attempt.is_some() {
                admin_queries::update_login_attempt(&state.postgres, patch_user.registered_user_id)
                    .await?;
            } else if let Some(password_reset_id) = body.patch.password_reset_id {
                admin_queries::consume_password_reset(&state.postgres, password_reset_id).await?;
            } else if let Some(with_email) = body.patch.reset {
                let secret = gen_random_hex(128);
                ModelPasswordReset::insert(
                    &state.postgres,
                    patch_user.registered_user_id,
                    &secret,
                    useragent_ip,
                )
                .await?;

                if with_email {
                    Email::new(
                        &patch_user.full_name,
                        &patch_user.email,
                        EmailTemplate::PasswordResetRequested(secret),
                        &state.email_env,
                    )
                    .send();
                }
            } else if body.patch.two_fa_secret.is_some() {
                admin_queries::disable_two_fa(&state.postgres, patch_user.registered_user_id)
                    .await?;
            }
            Ok(StatusCode::OK)
        } else {
            Err(ApiError::InvalidValue("Unknown user".to_owned()))
        }
    }
}

// Use reqwest to test against real server
// cargo watch -q -c -w src/ -x 'test api_router_admin -- --test-threads=1 --nocapture'
#[cfg(test)]
#[allow(clippy::pedantic, clippy::nursery, clippy::unwrap_used)]
mod tests {

    use redis::AsyncCommands;
    use reqwest::StatusCode;
    use std::collections::HashMap;

    use super::AdminRoutes;
    use crate::{
        api::{
            api_tests::{
                base_url, start_server, Response, ANON_EMAIL, ANON_FULL_NAME, TEST_EMAIL,
                TEST_FULL_NAME, TEST_PASSWORD,
            },
            ij::{AdminUserPatch, EmailPost, UserPatch},
        },
        database::{
            backup::{create_backup, BackupEnv, BackupType},
            ModelPasswordReset,
        },
        helpers::gen_random_hex,
        parse_env::AppEnv,
        sleep,
    };

    /// generate a backup and return it's file name
    async fn get_backup_filename(app_env: &AppEnv, t: BackupType) -> String {
        let backup_env = BackupEnv::new(app_env);
        create_backup(&backup_env, t).await.unwrap();
        let mut file_name = String::new();
        for i in std::fs::read_dir(&app_env.location_backup).unwrap() {
            file_name = i.unwrap().file_name().to_str().unwrap().to_owned();
        }
        file_name
    }

    #[tokio::test]
    // Unauthenticated user unable to [PATCH, POST] "/" route
    async fn api_router_admin_base_unauthenticated() {
        let test_setup = start_server().await;
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            AdminRoutes::Base.addr()
        );
        let client = reqwest::Client::new();

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
    // Authenticated, but not admin user, user unable to [PATCH, POST] "/" route
    async fn api_router_admin_base_not_admin() {
        let mut test_setup = start_server().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            AdminRoutes::Base.addr()
        );
        let client = reqwest::Client::new();

        let result = client
            .patch(&url)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");

        let result = client
            .post(&url)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");
    }

    #[tokio::test]
    async fn api_router_admin_base_admin_get_valid() {
        let mut test_setup = start_server().await;

        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.make_user_admin().await;

        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            AdminRoutes::Base.addr()
        );
        let client = reqwest::Client::new();

        let result = client
            .get(&url)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::OK);
    }

    // Backup

    #[tokio::test]
    // Unauthenticated user unable to [DELETE, GET, POST] "/backup" route
    async fn api_router_admin_backup_unauthenticated() {
        let test_setup = start_server().await;
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            AdminRoutes::Backup.addr()
        );
        let client = reqwest::Client::new();

        let result = client.delete(&url).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");

        let result = client.get(&url).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");

        let result = client.post(&url).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");
    }

    #[tokio::test]
    // Authenticated, but not admin user, user unable to [DELETE, GET, POST] "/backup" route
    async fn api_router_admin_backup_not_admin() {
        let mut test_setup = start_server().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            AdminRoutes::Backup.addr()
        );
        let client = reqwest::Client::new();

        let result = client
            .delete(&url)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");

        let result = client
            .get(&url)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");

        let result = client
            .post(&url)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");
    }

    #[tokio::test]
    // Admin user able to get list of backups,
    async fn api_router_admin_backup_get_ok() {
        let mut test_setup = start_server().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.make_user_admin().await;
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            AdminRoutes::Backup.addr()
        );
        let client = reqwest::Client::new();

        get_backup_filename(&test_setup.app_env, BackupType::SqlOnly).await;

        let result = client
            .get(&url)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::OK);
        let result = result.json::<Response>().await.unwrap().response;
        // also len 1
        assert!(result.is_object());
        assert_eq!(
            result
                .as_object()
                .unwrap()
                .get("backups")
                .unwrap()
                .as_array()
                .unwrap()
                .len(),
            1
        );

        let result = result
            .as_object()
            .unwrap()
            .get("backups")
            .unwrap()
            .as_array()
            .unwrap()[0]
            .as_object()
            .unwrap();

        assert!(result.get("file_name").is_some());
        assert!(result.get("file_name").unwrap().is_string());
        assert!(result.get("file_size").is_some());
        assert!(result.get("file_size").unwrap().is_i64());
    }

    #[tokio::test]
    // Admin user create backup with photos,
    async fn api_router_admin_backup_post_with_photo_ok() {
        let mut test_setup = start_server().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.make_user_admin().await;

        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            AdminRoutes::Backup.addr()
        );
        let client = reqwest::Client::new();
        let body = HashMap::from([("with_photos", true)]);

        let result = client
            .post(&url)
            .json(&body)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::OK);

        // Assert that only single backup created
        let number_backups = std::fs::read_dir(&test_setup.app_env.location_backup)
            .unwrap()
            .count();
        assert_eq!(number_backups, 1);

        // Assert is between 400mb and 450mb
        // Need to change these figures as the number of photos grows
        for i in std::fs::read_dir(&test_setup.app_env.location_backup).unwrap() {
            assert!(i.as_ref().unwrap().metadata().unwrap().len() > 400_000_000);
            assert!(i.unwrap().metadata().unwrap().len() < 450_000_000);
        }
    }

    #[tokio::test]
    // Admin user create backup without photos,
    async fn api_router_admin_backup_post_without_photo_ok() {
        let mut test_setup = start_server().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.make_user_admin().await;

        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            AdminRoutes::Backup.addr()
        );
        let client = reqwest::Client::new();
        let body = HashMap::from([("with_photos", false)]);

        let result = client
            .post(&url)
            .json(&body)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::OK);

        // Assert that only single backup created
        let number_backups = std::fs::read_dir(&test_setup.app_env.location_backup)
            .unwrap()
            .count();
        assert_eq!(number_backups, 1);

        // Assert is between 1mb and 5mb in size
        for i in std::fs::read_dir(&test_setup.app_env.location_backup).unwrap() {
            assert!(i.as_ref().unwrap().metadata().unwrap().len() > 800_000);
            assert!(i.unwrap().metadata().unwrap().len() < 5_000_000);
        }
    }

    #[tokio::test]
    // Delete a backup invalid request body
    async fn api_router_admin_backup_delete_invalid() {
        let mut test_setup = start_server().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.make_user_admin().await;

        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            AdminRoutes::Backup.addr()
        );

        get_backup_filename(&test_setup.app_env, BackupType::SqlOnly).await;

        // name invalid
        let client = reqwest::Client::new();
        let body = HashMap::from([("file_name", "some_random_name")]);

        let result = client
            .delete(&url)
            .json(&body)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::BAD_REQUEST);

        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "backup_name");

        // Assert that backup still on disk
        let number_backups = std::fs::read_dir(&test_setup.app_env.location_backup)
            .unwrap()
            .count();
        assert_eq!(number_backups, 1);

        // Not on disk
        let body = HashMap::from([(
            "file_name",
            "mealpedant_2020-07-07_03.00.00_LOGS_REDIS_SQL_77C451BD.tar.age",
        )]);

        let result = client
            .delete(&url)
            .json(&body)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::INTERNAL_SERVER_ERROR);

        // Assert that backup still on disk
        let number_backups = std::fs::read_dir(&test_setup.app_env.location_backup)
            .unwrap()
            .count();
        assert_eq!(number_backups, 1);
    }

    #[tokio::test]
    // Delete a backup,
    async fn api_router_admin_backup_delete_ok() {
        let mut test_setup = start_server().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.make_user_admin().await;

        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            AdminRoutes::Backup.addr()
        );
        let file_name = get_backup_filename(&test_setup.app_env, BackupType::SqlOnly).await;

        let client = reqwest::Client::new();
        let body = HashMap::from([("file_name", file_name)]);

        let result = client
            .delete(&url)
            .json(&body)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::OK);

        // Assert that only single backup created
        let number_backups = std::fs::read_dir(&test_setup.app_env.location_backup)
            .unwrap()
            .count();
        assert_eq!(number_backups, 0);
    }

    #[tokio::test]
    // Unauthenticated user unable to [GET] "/backup/:file_name" route
    async fn api_router_admin_backup_param_unauthenticated() {
        let test_setup = start_server().await;
        let file_name = get_backup_filename(&test_setup.app_env, BackupType::SqlOnly).await;

        let url = format!(
            "{}{}/{}",
            base_url(&test_setup.app_env),
            AdminRoutes::Backup.addr(),
            file_name
        );
        let client = reqwest::Client::new();

        let result = client.get(&url).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");
    }

    #[tokio::test]
    // Authenticated, but not admin user, user unable to [GET] "/backup/:file_name" route
    async fn api_router_admin_backup_param_not_admin() {
        let mut test_setup = start_server().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        let file_name = get_backup_filename(&test_setup.app_env, BackupType::SqlOnly).await;

        let url = format!(
            "{}{}/{}",
            base_url(&test_setup.app_env),
            AdminRoutes::Backup.addr(),
            file_name
        );
        let client = reqwest::Client::new();

        let result = client
            .get(&url)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");
    }

    #[tokio::test]
    // Authenticated, but not admin user, user unable to [GET] "/backup/:file_name" route
    async fn api_router_admin_backup_param_invalid_name() {
        let mut test_setup = start_server().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.make_user_admin().await;
        let file_name = get_backup_filename(&test_setup.app_env, BackupType::SqlOnly).await;

        let url = format!(
            "{}{}/{}",
            base_url(&test_setup.app_env),
            AdminRoutes::Backup.addr(),
            file_name.chars().skip(1).collect::<String>()
        );

        let client = reqwest::Client::new();

        let result = client
            .get(&url)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::BAD_REQUEST);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "backup_name");
    }

    #[tokio::test]
    // Authenticated, but not admin user, user unable to [GET] "/backup/:file_name" route
    async fn api_router_admin_backup_param_ok() {
        let mut test_setup = start_server().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.make_user_admin().await;
        let file_name = get_backup_filename(&test_setup.app_env, BackupType::SqlOnly).await;

        let url = format!(
            "{}{}/{}",
            base_url(&test_setup.app_env),
            AdminRoutes::Backup.addr(),
            file_name
        );
        let client = reqwest::Client::new();

        let result = client
            .get(&url)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::OK);

        let download_as_bytes = result.bytes().await;

        assert!(download_as_bytes.is_ok());
    }

    // Memory
    #[tokio::test]
    // Unauthenticated user unable to [GET] "/memory" route
    async fn api_router_admin_memory_unauthenticated() {
        let test_setup = start_server().await;

        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            AdminRoutes::Memory.addr(),
        );
        let client = reqwest::Client::new();

        let result = client.get(&url).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");
    }

    #[tokio::test]
    // Authenticated, but not admin user, user unable to [GET] "/memory" route
    async fn api_router_admin_memory_not_admin() {
        let mut test_setup = start_server().await;
        let authed_cookie = test_setup.authed_user_cookie().await;

        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            AdminRoutes::Memory.addr(),
        );
        let client = reqwest::Client::new();

        let result = client
            .get(&url)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");
    }

    #[tokio::test]
    // Authenticated admin user able to get memory & uptime statsq
    async fn api_router_admin_memory_ok() {
        let mut test_setup = start_server().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.make_user_admin().await;

        sleep!();
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            AdminRoutes::Memory.addr(),
        );
        let client = reqwest::Client::new();

        let result = client
            .get(&url)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::OK);
        let result = result.json::<Response>().await.unwrap().response;

        assert!(result["rss"].is_number());
        assert!(result["virt"].is_number());
        assert!(result["uptime_app"].is_number());
        assert!(result["uptime"].is_number());

        // Assume the app has been alive for 1..10 seconds, in reality should be 1 or 2
        assert!((1..=10).contains(&result["uptime_app"].as_u64().unwrap()));
        // Assume the comptuer has been on for longer than 15 seconds
        assert!(result["uptime"].as_u64().unwrap() > 15);

        assert!(result["virt"].as_u64().unwrap() > result["rss"].as_u64().unwrap());
    }

    // Restart
    #[tokio::test]
    // Unauthenticated user unable to [PUT] "/restart" route
    async fn api_router_admin_restart_unauthenticated() {
        let test_setup = start_server().await;

        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            AdminRoutes::Restart.addr(),
        );
        let client = reqwest::Client::new();
        // Need to create body
        let body = HashMap::from([("password", TEST_PASSWORD)]);

        let result = client.put(&url).json(&body).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");
    }

    #[tokio::test]
    // Authenticated, but not admin user, user unable to [PUT] "/restart" route
    async fn api_router_admin_restart_not_admin() {
        let mut test_setup = start_server().await;
        let authed_cookie = test_setup.authed_user_cookie().await;

        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            AdminRoutes::Restart.addr(),
        );
        let client = reqwest::Client::new();
        let body = HashMap::from([("password", TEST_PASSWORD)]);

        let result = client
            .get(&url)
            .json(&body)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");
    }

    #[tokio::test]
    // Authenticated admin user able to restart the application
    async fn api_router_admin_restart_ok() {
        let mut test_setup = start_server().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.make_user_admin().await;

        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            AdminRoutes::Restart.addr(),
        );
        let client = reqwest::Client::new();
        let body = HashMap::from([("password", TEST_PASSWORD)]);

        let result = client
            .put(&url)
            .json(&body)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::OK);
    }

    // User

    #[tokio::test]
    // Unauthenticated user unable to [GET, PATCH] "/user" route
    async fn api_router_admin_user_unauthenticated() {
        let test_setup = start_server().await;
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            AdminRoutes::User.addr(),
        );
        let client = reqwest::Client::new();

        let result = client.get(&url).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");

        let result = client.patch(&url).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");
    }

    #[tokio::test]
    // Authenticated, but not admin user, user unable to [GET, PATCH] "/user" route
    async fn api_router_admin_user_not_admin() {
        let mut test_setup = start_server().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            AdminRoutes::User.addr(),
        );
        let client = reqwest::Client::new();

        let result = client
            .get(&url)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");

        let result = client
            .patch(&url)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");
    }

    #[tokio::test]
    // Authenticated admin user able to get list of current users
    async fn api_router_admin_user_get_ok() {
        let mut test_setup = start_server().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.make_user_admin().await;
        test_setup.insert_two_fa().await;
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            AdminRoutes::User.addr(),
        );
        let client = reqwest::Client::new();
        let body = HashMap::from([("password", TEST_PASSWORD)]);

        let result = client
            .get(&url)
            .json(&body)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::OK);
        let result = result.json::<Response>().await.unwrap().response;
        let result = result.as_array().unwrap();
        let len = result.len() - 1;

        let active = result[len].as_object().unwrap()["active"]
            .as_bool()
            .unwrap();
        assert!(active);

        let admin = result[len].as_object().unwrap()["admin"].as_bool().unwrap();
        assert!(admin);

        let email = result[len].as_object().unwrap()["email"].as_str().unwrap();
        assert_eq!(email, TEST_EMAIL);

        let full_name = result[len].as_object().unwrap()["full_name"]
            .as_str()
            .unwrap();
        assert_eq!(full_name, TEST_FULL_NAME);

        let login_attempt_number = result[len].as_object().unwrap()["login_attempt_number"]
            .as_i64()
            .unwrap();
        assert_eq!(login_attempt_number, 0);

        let login_ip = result[len].as_object().unwrap()["login_ip"]
            .as_str()
            .unwrap();
        assert_eq!(login_ip, "127.0.0.1");

        let login_success = result[len].as_object().unwrap()["login_success"]
            .as_bool()
            .unwrap();
        assert!(login_success);

        let password_reset_consumed =
            result[len].as_object().unwrap()["password_reset_consumed"].is_null();
        assert!(password_reset_consumed);

        let password_reset_creation_ip =
            result[len].as_object().unwrap()["password_reset_creation_ip"].is_null();
        assert!(password_reset_creation_ip);

        let password_reset_date = result[len].as_object().unwrap()["password_reset_date"].is_null();
        assert!(password_reset_date);

        let password_reset_id = result[len].as_object().unwrap()["password_reset_id"].is_null();
        assert!(password_reset_id);

        let reset_string = result[len].as_object().unwrap()["reset_string"].is_null();
        assert!(reset_string);

        let two_fa_active = result[len].as_object().unwrap()["two_fa_active"]
            .as_bool()
            .unwrap();
        assert!(two_fa_active);

        let user_creation_ip = result[len].as_object().unwrap()["user_creation_ip"]
            .as_str()
            .unwrap();
        assert_eq!(user_creation_ip, "123.123.123.123");

        let user_agent_string = result[len].as_object().unwrap()["user_agent_string"]
            .as_str()
            .unwrap();
        assert_eq!(user_agent_string, "UNKNOWN");
    }

    #[tokio::test]
    // Authenticated admin can't patch self
    async fn api_router_admin_user_patch_self() {
        let mut test_setup = start_server().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.make_user_admin().await;
        test_setup.insert_anon_user().await;

        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            AdminRoutes::User.addr(),
        );
        let client = reqwest::Client::new();
        let body = AdminUserPatch {
            patch: UserPatch {
                active: Some(false),
                attempt: None,
                password_reset_id: None,
                reset: None,
                two_fa_secret: None,
            },
            email: TEST_EMAIL.to_owned(),
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
        assert_eq!(result, "can't edit self");
    }

    #[tokio::test]
    // Authenticated admin can't patch an unknown user
    async fn api_router_admin_user_patch_unknown() {
        let mut test_setup = start_server().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.make_user_admin().await;
        test_setup.insert_anon_user().await;

        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            AdminRoutes::User.addr(),
        );
        let client = reqwest::Client::new();
        let body = AdminUserPatch {
            patch: UserPatch {
                active: Some(false),
                attempt: None,
                password_reset_id: None,
                reset: None,
                two_fa_secret: None,
            },
            email: "abc@example.com".to_owned(),
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
        assert_eq!(result, "Unknown user");
    }

    #[tokio::test]
    // Authenticated admin update user, set as inactive
    async fn api_router_admin_user_patch_inactive() {
        let mut test_setup = start_server().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.make_user_admin().await;
        test_setup.insert_anon_user().await;
        let client = reqwest::Client::new();

        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            AdminRoutes::User.addr(),
        );

        // Set as inactive
        let body = AdminUserPatch {
            patch: UserPatch {
                active: Some(false),
                attempt: None,
                password_reset_id: None,
                reset: None,
                two_fa_secret: None,
            },
            email: ANON_EMAIL.to_owned(),
        };

        let result = client
            .patch(&url)
            .json(&body)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::OK);
        let anon_user = test_setup.get_anon_user().await;
        assert!(anon_user.is_none());
    }

    #[tokio::test]
    // Authenticated admin update user, set as active
    async fn api_router_admin_user_patch_active() {
        let mut test_setup = start_server().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.make_user_admin().await;
        test_setup.insert_anon_user().await;
        let client = reqwest::Client::new();

        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            AdminRoutes::User.addr(),
        );

        let body = AdminUserPatch {
            patch: UserPatch {
                active: Some(false),
                attempt: None,
                password_reset_id: None,
                reset: None,
                two_fa_secret: None,
            },
            email: ANON_EMAIL.to_owned(),
        };

        client
            .patch(&url)
            .json(&body)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();

        let body = AdminUserPatch {
            patch: UserPatch {
                active: Some(true),
                attempt: None,
                password_reset_id: None,
                reset: None,
                two_fa_secret: None,
            },
            email: ANON_EMAIL.to_owned(),
        };

        let result = client
            .patch(&url)
            .json(&body)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::OK);
        let anon_user = test_setup.get_anon_user().await;
        assert!(anon_user.is_some());
    }

    #[tokio::test]
    // Authenticated admin update user, force password reset with email sent
    async fn api_router_admin_user_patch_password_with_email() {
        let mut test_setup = start_server().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.make_user_admin().await;
        test_setup.insert_anon_user().await;
        let client = reqwest::Client::new();

        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            AdminRoutes::User.addr(),
        );

        // insert password reset, and send email
        let body = AdminUserPatch {
            patch: UserPatch {
                active: None,
                attempt: None,
                password_reset_id: None,
                reset: Some(true),
                two_fa_secret: None,
            },
            email: ANON_EMAIL.to_owned(),
        };

        let result = client
            .patch(&url)
            .json(&body)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::OK);

        let password_reset = ModelPasswordReset::get_by_email(&test_setup.postgres, ANON_EMAIL)
            .await
            .unwrap();
        assert!(password_reset.is_some());

        let result = std::fs::metadata("/dev/shm/email_headers.txt");
        assert!(result.is_ok());
        let result = std::fs::metadata("/dev/shm/email_body.txt");
        assert!(result.is_ok());
        assert!(std::fs::read_to_string("/dev/shm/email_body.txt")
            .unwrap()
            .contains("This password reset link will only be valid for one hour"));

        assert!(std::fs::read_to_string("/dev/shm/email_headers.txt")
            .unwrap()
            .contains("Password Reset Requested"));
    }

    #[tokio::test]
    // Authenticated admin update user, force password reset without email sent
    async fn api_router_admin_user_patch_password_no_email() {
        let mut test_setup = start_server().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.make_user_admin().await;
        test_setup.insert_anon_user().await;
        let client = reqwest::Client::new();

        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            AdminRoutes::User.addr(),
        );

        // insert password reset, and no email
        let body = AdminUserPatch {
            patch: UserPatch {
                active: None,
                attempt: None,
                password_reset_id: None,
                reset: Some(false),
                two_fa_secret: None,
            },
            email: ANON_EMAIL.to_owned(),
        };

        let result = client
            .patch(&url)
            .json(&body)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::OK);
        let result = std::fs::metadata("/dev/shm/email_headers.txt");
        assert!(result.is_err());
        let result = std::fs::metadata("/dev/shm/email_body.txt");
        assert!(result.is_err());

        let password_reset = ModelPasswordReset::get_by_email(&test_setup.postgres, ANON_EMAIL)
            .await
            .unwrap();
        assert!(password_reset.is_some());
    }

    #[tokio::test]
    // Authenticated admin update user, consume an inplace password reset
    async fn api_router_admin_user_patch_consume_password() {
        let mut test_setup = start_server().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.make_user_admin().await;
        test_setup.insert_anon_user().await;
        let client = reqwest::Client::new();

        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            AdminRoutes::User.addr(),
        );

        // insert password reset, and no email
        let body = AdminUserPatch {
            patch: UserPatch {
                active: None,
                attempt: None,
                password_reset_id: None,
                reset: Some(false),
                two_fa_secret: None,
            },
            email: ANON_EMAIL.to_owned(),
        };

        client
            .patch(&url)
            .json(&body)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();

        let password_reset = ModelPasswordReset::get_by_email(&test_setup.postgres, ANON_EMAIL)
            .await
            .unwrap()
            .unwrap();

        // Consume a password reset request
        let body = AdminUserPatch {
            patch: UserPatch {
                active: None,
                attempt: None,
                password_reset_id: Some(password_reset.password_reset_id),
                reset: None,
                two_fa_secret: None,
            },
            email: ANON_EMAIL.to_owned(),
        };

        let result = client
            .patch(&url)
            .json(&body)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::OK);

        let password_reset = ModelPasswordReset::get_by_email(&test_setup.postgres, ANON_EMAIL)
            .await
            .unwrap();
        assert!(password_reset.is_none());
    }

    #[tokio::test]
    // Authenticated admin update user, consume an inplace password reset
    async fn api_router_admin_user_patch_two_fa() {
        let mut test_setup = start_server().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.make_user_admin().await;
        test_setup.insert_anon_user().await;
        let client = reqwest::Client::new();
        assert!(test_setup
            .anon_user
            .as_ref()
            .unwrap()
            .two_fa_secret
            .is_some());

        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            AdminRoutes::User.addr(),
        );

        let body = AdminUserPatch {
            patch: UserPatch {
                active: None,
                attempt: None,
                password_reset_id: None,
                reset: None,
                two_fa_secret: Some(true),
            },
            email: ANON_EMAIL.to_owned(),
        };

        let result = client
            .patch(&url)
            .json(&body)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::OK);

        let anon_user = test_setup.get_anon_user().await.unwrap();
        assert!(anon_user.two_fa_secret.is_none());
        assert_eq!(anon_user.two_fa_backup_count, 0);
    }

    // SESSION

    #[tokio::test]
    // Unauthenticated user unable to [DELETE, GET] "/session/:param" route
    async fn api_router_admin_session_unauthenticated() {
        let test_setup = start_server().await;
        let url = format!(
            "{}/admin/session/20982f6987cf4b77bc7b35097157b12d",
            base_url(&test_setup.app_env),
        );
        let client = reqwest::Client::new();

        let result = client.delete(&url).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");

        let result = client.get(&url).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");
    }

    #[tokio::test]
    // Unauthenticated user unable to [DELETE, GET] "/session/:param" route
    async fn api_router_admin_session_param_not_admin() {
        let test_setup = start_server().await;
        let url = format!(
            "{}/admin/session/20982f6987cf4b77bc7b35097157b12d",
            base_url(&test_setup.app_env),
        );
        let client = reqwest::Client::new();

        let result = client.delete(&url).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");

        let result = client.get(&url).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");
    }

    #[tokio::test]
    // Authenticated admin user, invalid request when uuid isn't correct format
    async fn api_router_admin_session_param_invalid_uuid() {
        let mut test_setup = start_server().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.make_user_admin().await;
        let url = format!(
            "{}/admin/session/20982f6987cf4b77bc7b35097157",
            base_url(&test_setup.app_env),
        );
        let client = reqwest::Client::new();

        let result = client
            .delete(&url)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::BAD_REQUEST);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "uuid");
    }

    #[tokio::test]
    // Authenticated admin user, invalid request when session is current session
    async fn api_router_admin_session_param_self_session() {
        let mut test_setup = start_server().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.make_user_admin().await;
        let session_set_key = format!(
            "session_set::user::{}",
            test_setup.model_user.unwrap().registered_user_id
        );
        let session_set: Vec<String> = test_setup
            .redis
            .lock()
            .await
            .smembers(session_set_key)
            .await
            .unwrap();
        let (_, uuid) = session_set.get(0).unwrap().split_at(9);
        let url = format!("{}/admin/session/{}", base_url(&test_setup.app_env), uuid);
        let client = reqwest::Client::new();

        let result = client
            .delete(&url)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::BAD_REQUEST);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "can't remove current session");
    }

    #[tokio::test]
    // Authenticated admin user, delete anon_user session, anon user unable to get /user route, session & session set for anon user both non/empty
    async fn api_router_admin_session_param_delete_anon_session() {
        let mut test_setup = start_server().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.make_user_admin().await;
        test_setup.insert_anon_user().await;
        let anon_cookie = test_setup.anon_user_cookie().await;

        let session_set_key = format!(
            "session_set::user::{}",
            test_setup.anon_user.unwrap().registered_user_id
        );
        let session_set: Vec<String> = test_setup
            .redis
            .lock()
            .await
            .smembers(&session_set_key)
            .await
            .unwrap();
        let (_, uuid) = session_set.get(0).unwrap().split_at(9);

        let session: Option<String> = test_setup
            .redis
            .lock()
            .await
            .hget(session_set.get(0).unwrap(), "data")
            .await
            .unwrap();

        assert!(session.is_some());

        let url = format!("{}/admin/session/{}", base_url(&test_setup.app_env), uuid);
        let client = reqwest::Client::new();

        let result = client
            .delete(&url)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::OK);

        let url = format!("{}/user", base_url(&test_setup.app_env),);
        let client = reqwest::Client::new();

        let result = client
            .get(&url)
            .header("cookie", &anon_cookie)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");

        let session: Option<String> = test_setup
            .redis
            .lock()
            .await
            .hget(session_set.get(0).unwrap(), "data")
            .await
            .unwrap();

        assert!(session.is_none());

        let session_set: Vec<String> = test_setup
            .redis
            .lock()
            .await
            .smembers(session_set_key)
            .await
            .unwrap();

        assert!(session_set.is_empty());
    }

    #[tokio::test]
    // Authenticated admin user, error - unknown user
    async fn api_router_admin_session_param_get_unknown_user() {
        let mut test_setup = start_server().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.make_user_admin().await;

        let url = format!(
            "{}/admin/session/{}",
            base_url(&test_setup.app_env),
            ANON_EMAIL
        );
        let client = reqwest::Client::new();

        let result = client
            .get(&url)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::BAD_REQUEST);

        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "unknown user");
    }

    #[tokio::test]
    // Authenticated admin user, empty array when no sessions
    async fn api_router_admin_session_param_get_no_sessions() {
        let mut test_setup = start_server().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.insert_anon_user().await;
        test_setup.make_user_admin().await;

        let url = format!(
            "{}/admin/session/{}",
            base_url(&test_setup.app_env),
            ANON_EMAIL
        );
        let client = reqwest::Client::new();

        let result = client
            .get(&url)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::OK);

        let result = result.json::<Response>().await.unwrap().response;

        assert!(result.is_array());
        assert!(result.as_array().unwrap().is_empty());
    }

    #[tokio::test]
    // Authenticated admin user, get list of current session for a given email address
    async fn api_router_admin_session_param_get_session() {
        let mut test_setup = start_server().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.make_user_admin().await;

        let url = format!(
            "{}/admin/session/{}",
            base_url(&test_setup.app_env),
            TEST_EMAIL
        );
        let client = reqwest::Client::new();

        let result = client
            .get(&url)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::OK);

        let result = result.json::<Response>().await.unwrap().response;

        assert!(result.is_array());
        let result = result.as_array().unwrap()[0].as_object().unwrap();
        assert!(result.get("end_date").is_some());
        assert!(result.get("ip").is_some());
        assert!(result.get("login_date").is_some());
        assert!(result.get("user_agent").is_some());
        assert!(result.get("uuid").is_some());
    }

    // LIMITS

    #[tokio::test]
    // Unauthenticated user unable to [DELETE, GET] "/session/:limit" route
    async fn api_router_admin_limit_unauthenticated() {
        let test_setup = start_server().await;
        let url = format!("{}/admin/limit", base_url(&test_setup.app_env),);

        let client = reqwest::Client::new();

        let result = client.delete(&url).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");

        let result = client.get(&url).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");
    }

    #[tokio::test]
    // Unauthenticated user unable to [DELETE, GET] "/session/:limit" route
    async fn api_router_admin_session_limit_not_admin() {
        let test_setup = start_server().await;
        let url = format!("{}/admin/limit", base_url(&test_setup.app_env),);
        let client = reqwest::Client::new();

        let result = client.delete(&url).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");

        let result = client.get(&url).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");
    }

    #[tokio::test]
    // Authenticated admin user get list of rate limits
    async fn api_router_admin_session_limit_get() {
        let mut test_setup = start_server().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.make_user_admin().await;
        let url = format!("{}/admin/limit", base_url(&test_setup.app_env),);
        let client = reqwest::Client::new();

        for _ in 0..=8 {
            client
                .get(&url)
                .header("cookie", &authed_cookie)
                .send()
                .await
                .unwrap();
        }

        let result = client
            .get(&url)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::OK);
        let result = result.json::<Response>().await.unwrap().response;

        assert!(result.is_array());
        let result = result.as_array().unwrap();
        assert!(result.len() == 2);

        let test_index = result.iter().position(|i| {
            i.as_object().unwrap().get("key").unwrap().as_str().unwrap() == TEST_EMAIL
        });
        assert!(test_index.is_some());
        assert_eq!(
            result[test_index.unwrap()]
                .as_object()
                .unwrap()
                .get("points")
                .unwrap()
                .as_i64()
                .unwrap(),
            10
        );

        let ip_index = result.iter().position(|i| {
            i.as_object().unwrap().get("key").unwrap().as_str().unwrap() == "127.0.0.1"
        });
        assert!(ip_index.is_some());
        assert_eq!(
            result[ip_index.unwrap()]
                .as_object()
                .unwrap()
                .get("points")
                .unwrap()
                .as_i64()
                .unwrap(),
            1
        );
    }

    #[tokio::test]
    // Authenticated admin user able to delete rate limit for a single user
    async fn api_router_admin_session_limit_delete_email() {
        let mut test_setup = start_server().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.make_user_admin().await;
        let url = format!("{}/admin/limit", base_url(&test_setup.app_env),);
        let client = reqwest::Client::new();

        for _ in 0..=8 {
            client
                .get(&url)
                .header("cookie", &authed_cookie)
                .send()
                .await
                .unwrap();
        }

        let body = HashMap::from([("key", TEST_EMAIL)]);
        let result = client
            .delete(&url)
            .json(&body)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::OK);

        let result = client
            .get(&url)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();

        let result = result.json::<Response>().await.unwrap().response;
        assert!(result.is_array());
        let result = result.as_array().unwrap();
        assert!(result.len() == 2);

        let index = result.iter().position(|i| {
            i.as_object().unwrap().get("key").unwrap().as_str().unwrap() == TEST_EMAIL
        });
        assert!(index.is_some());
        assert_eq!(
            result[index.unwrap()]
                .as_object()
                .unwrap()
                .get("points")
                .unwrap()
                .as_i64()
                .unwrap(),
            1
        );
    }

    #[tokio::test]
    // Authenticated admin user able to delete rate limit for a ip
    async fn api_router_admin_session_limit_delete_ip() {
        let mut test_setup = start_server().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.make_user_admin().await;
        let url = format!("{}/admin/limit", base_url(&test_setup.app_env),);
        let client = reqwest::Client::new();

        for _ in 0..=8 {
            client.get(&url).send().await.unwrap();
        }

        let body = HashMap::from([("key", "127.0.0.1")]);
        let result = client
            .delete(&url)
            .json(&body)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::OK);

        let result = client
            .get(&url)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();

        let result = result.json::<Response>().await.unwrap().response;
        let result = result.as_array().unwrap();
        assert!(result.len() == 1);
    }

    // EMAIL
    #[tokio::test]
    // Unauthenticated user unable to [GET, POST] "/email" route
    async fn api_router_admin_email_unauthenticated() {
        let test_setup = start_server().await;
        let url = format!("{}/admin/email", base_url(&test_setup.app_env),);
        let client = reqwest::Client::new();

        let result = client.get(&url).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");

        let result = client.post(&url).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");
    }

    #[tokio::test]
    // Unauthenticated user unable to [GET, POST] "/email" route
    async fn api_router_email_session_limit_not_admin() {
        let test_setup = start_server().await;
        let url = format!("{}/admin/email", base_url(&test_setup.app_env),);
        let client = reqwest::Client::new();

        let result = client.get(&url).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");

        let result = client.post(&url).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");
    }

    #[tokio::test]
    // Authenticated admin user able to get array of email address, contains TEST and ANON emails
    async fn api_router_admin_email_get() {
        let mut test_setup = start_server().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.make_user_admin().await;
        test_setup.insert_anon_user().await;

        let url = format!("{}/admin/email", base_url(&test_setup.app_env),);
        let client = reqwest::Client::new();

        let result = client
            .get(&url)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();

        let result = result.json::<Response>().await.unwrap().response;
        assert!(result.is_array());
        let result = result.as_array().unwrap();
        assert!(!result.is_empty());
        assert!(result.contains(&serde_json::Value::String(TEST_EMAIL.to_string())));
        assert!(result.contains(&serde_json::Value::String(ANON_EMAIL.to_string())));
    }

    #[tokio::test]
    // Authenticated admin user doesn't send email when email address is unknown
    async fn api_router_admin_email_post_unknown_email() {
        let mut test_setup = start_server().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.make_user_admin().await;

        let title = gen_random_hex(12);
        let line_one = gen_random_hex(24);
        let body = EmailPost {
            emails: vec![ANON_EMAIL.to_owned()],
            title,
            line_one,
            line_two: None,
            button_text: None,
            link: None,
        };

        let url = format!("{}/admin/email", base_url(&test_setup.app_env),);
        let client = reqwest::Client::new();

        let result = client
            .post(&url)
            .json(&body)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::OK);

        let result = std::fs::metadata("/dev/shm/email_headers.txt");
        assert!(result.is_err());
        let result = std::fs::metadata("/dev/shm/email_body.txt");
        assert!(result.is_err());
    }

    #[tokio::test]
    // Authenticated admin user sends email
    async fn api_router_admin_email_post_ok() {
        let mut test_setup = start_server().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.make_user_admin().await;
        test_setup.insert_anon_user().await;

        let title = gen_random_hex(12);
        let line_one = gen_random_hex(24);
        let body = EmailPost {
            emails: vec![ANON_EMAIL.to_owned()],
            title: title.clone(),
            line_one: line_one.clone(),
            line_two: None,
            button_text: None,
            link: None,
        };

        let url = format!("{}/admin/email", base_url(&test_setup.app_env),);
        let client = reqwest::Client::new();

        let result = client
            .post(&url)
            .json(&body)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::OK);

        let result = std::fs::metadata("/dev/shm/email_headers.txt");
        assert!(result.is_ok());
        let result = std::fs::metadata("/dev/shm/email_body.txt");
        assert!(result.is_ok());
        assert!(std::fs::read_to_string("/dev/shm/email_body.txt")
            .unwrap()
            .contains(&line_one));

        assert!(std::fs::read_to_string("/dev/shm/email_headers.txt")
            .unwrap()
            .contains(&title));

        let email_to = format!("To: \"{ANON_FULL_NAME}\" <{ANON_EMAIL}>");
        assert!(std::fs::read_to_string("/dev/shm/email_headers.txt")
            .unwrap()
            .contains(&email_to));
    }

    // Logs

    #[tokio::test]
    // Unauthenticated user unable to [GET] "/logs" route
    async fn api_router_admin_logs_unauthenticated() {
        let test_setup = start_server().await;
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            AdminRoutes::Logs.addr(),
        );
        let client = reqwest::Client::new();

        let result = client.get(&url).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");
    }

    #[tokio::test]
    // Unauthenticated user unable to [GET] "/logs" route
    async fn api_router_admin_logs_not_admin() {
        let test_setup = start_server().await;
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            AdminRoutes::Logs.addr(),
        );
        let client = reqwest::Client::new();

        let result = client.get(&url).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");
    }

    #[tokio::test]
    // Authenticated admin user get array of logs
    async fn api_router_admin_logs_ok() {
        let mut test_setup = start_server().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.make_user_admin().await;
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            AdminRoutes::Logs.addr(),
        );
        let client = reqwest::Client::new();
        let result = client
            .get(&url)
            .header("cookie", authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::OK);
        let result = result.json::<Response>().await.unwrap().response;
        assert!(result.is_array());
        assert!(result.as_array().unwrap()[0]
            .as_object()
            .unwrap()
            .get("level")
            .is_some());
    }
}
