use axum_extra::extract::PrivateCookieJar;
use fred::prelude::Pool;
use reqwest::{
    StatusCode,
    header::{self, CACHE_CONTROL},
};
use sqlx::PgPool;
use std::{net::SocketAddr, path::PathBuf};
use tokio_util::io::ReaderStream;
use tower::ServiceBuilder;
use tower_http::services::ServeDir;

use axum::{
    Extension, Router,
    body::Body,
    extract::{Path, State},
    http::{HeaderName, HeaderValue, Request, Response},
    middleware::{self, Next},
    response::{AppendHeaders, IntoResponse},
    routing::get,
};

use crate::{
    C, S,
    api_error::ApiError,
    database::RedisSession,
    define_routes,
    parse_env::AppEnv,
    servers::{create_cors_layer, get_addr, ij::PhotoName, rate_limiting, shutdown_signal},
};

use super::{ApiState, get_cookie_ulid};

const MIME_WEBP: &str = "image/webp";
const MIME_JPEG: &str = "image/jpeg";

const HEADER_NO_CACHE: HeaderValue = HeaderValue::from_static("no-cache");
const HEADER_MAX_AGE: HeaderValue = HeaderValue::from_static("max-age=8640000");

pub struct StaticRouter;

impl StaticRouter {
    /// Serve the application
    pub async fn serve(app_env: AppEnv, postgres: PgPool, redis: Pool) -> Result<(), ApiError> {
        let cors_layer = create_cors_layer(&app_env)?;
        let application_state = ApiState::new(&app_env, C!(postgres), C!(redis));

        let serve_public = ServiceBuilder::new()
            .layer(middleware::from_fn(Self::set_static_cache_control))
            .service(
                ServeDir::new(&application_state.location_public)
                    .precompressed_br()
                    .precompressed_gzip(),
            );

        let app = Router::new()
            .route(&StaticRoutes::Photo.addr(), get(Self::photo_get))
            .layer(
                ServiceBuilder::new()
                    .layer(Extension(C!(application_state.cookie_key)))
                    .layer(middleware::from_fn_with_state(
                        application_state.clone(),
                        rate_limiting,
                    )),
            )
            .layer(cors_layer)
            .fallback_service(serve_public)
            .with_state(C!(application_state));

        let addr = get_addr(&app_env.static_host, app_env.static_port)?;
        tracing::info!("starting static_server @ {addr}");

        match axum::serve(
            tokio::net::TcpListener::bind(&addr).await?,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(shutdown_signal())
        .await
        {
            Ok(()) => Ok(()),
            Err(_) => Err(ApiError::Internal(S!("api_server"))),
        }
    }
}

define_routes! {
    StaticRoutes,
    "/",
    Photo => "photo/{file_name}"
}

impl StaticRouter {
    async fn set_static_cache_control(
        request: Request<axum::body::Body>,
        next: Next,
    ) -> Response<axum::body::Body> {
        let mut response = next.run(request).await;
        response
            .headers_mut()
            .insert(header::CACHE_CONTROL, HEADER_MAX_AGE);
        response
    }

    /// Read a photo file from disk and send to user, cache dependant on file and person type
    async fn serve_photo(
        file_path: PathBuf,
        cache: HeaderValue,
    ) -> Result<(AppendHeaders<[(HeaderName, HeaderValue); 3]>, Body), ()> {
        let Ok(file) = tokio::fs::File::open(&file_path).await else {
            return Err(());
        };

        let len = file.metadata().await.map_err(|_| ())?.len().to_string();
        let stream = ReaderStream::new(file);
        let body = Body::from_stream(stream);
        let Some(ext) = file_path.extension() else {
            return Err(());
        };
        let headers = AppendHeaders([
            (
                header::CONTENT_TYPE,
                HeaderValue::from_static(if ext == "webp" { MIME_WEBP } else { MIME_JPEG }),
            ),
            (
                header::CONTENT_LENGTH,
                HeaderValue::from_str(len.as_str()).unwrap_or(HeaderValue::from_static("512000")),
            ),
            (header::CACHE_CONTROL, cache),
        ]);
        Ok((headers, body))
    }

    /// Send a photo to user, will depend on auth status and photo status
    async fn photo_get(
        State(state): State<ApiState>,
        jar: PrivateCookieJar,
        Path(file_name): Path<String>,
    ) -> impl IntoResponse {
        // closure to return a 404 with correct header if photo not found, or user has wrong auth
        let not_found = || {
            let mut response = StatusCode::NOT_FOUND.into_response();
            response
                .headers_mut()
                .append(CACHE_CONTROL, HEADER_NO_CACHE);
            response
        };

        let Ok(photoname) = PhotoName::try_from(file_name) else {
            return not_found();
        };

        let user = if let Some(ulid) = get_cookie_ulid(&state, &jar) {
            RedisSession::exists(&state.redis, &ulid)
                .await
                .unwrap_or_default()
        } else {
            None
        };
        match &photoname {
            PhotoName::Converted(_) => {
                if photoname.is_dave() && user.is_none() {
                    not_found()
                } else {
                    let cache = if photoname.is_dave() {
                        HEADER_NO_CACHE
                    } else {
                        HEADER_MAX_AGE
                    };
                    (Self::serve_photo(state.photo_env.get_pathbuff(photoname), cache).await)
                        .map_or_else(
                            |()| not_found(),
                            axum::response::IntoResponse::into_response,
                        )
                }
            }
            PhotoName::Original(_) => {
                if user.is_some() {
                    (Self::serve_photo(state.photo_env.get_pathbuff(photoname), HEADER_NO_CACHE)
                        .await)
                        .map_or_else(
                            |()| not_found(),
                            axum::response::IntoResponse::into_response,
                        )
                } else {
                    not_found()
                }
            }
        }
    }
}

// Use reqwest to test against real server
#[cfg(test)]
#[expect(clippy::unwrap_used)]
mod tests {

    use axum::http::HeaderMap;
    use fred::prelude::KeysInterface;
    use rand::seq::SliceRandom;
    use reqwest::{
        StatusCode,
        header::{
            ACCESS_CONTROL_ALLOW_CREDENTIALS, ACCESS_CONTROL_ALLOW_HEADERS,
            ACCESS_CONTROL_ALLOW_ORIGIN, CACHE_CONTROL, CONTENT_LENGTH, CONTENT_TYPE, VARY,
        },
    };
    use ulid::Ulid;

    use crate::{
        C,
        helpers::gen_random_hex,
        parse_env::AppEnv,
        servers::api_tests::{TEST_EMAIL, TestSetup, start_both_servers},
    };

    /// Check rate limit is as expected
    async fn assert_unauth_limit(test_setup: &TestSetup, count: usize) {
        assert_eq!(
            test_setup
                .redis
                .get::<usize, &str>("ratelimit::ip::127.0.0.1")
                .await
                .unwrap(),
            count
        );
    }

    /// Check rate limit is as expected
    async fn assert_authed_limit(test_setup: &TestSetup, count: usize) {
        assert_eq!(
            test_setup
                .redis
                .get::<usize, String>(format!("ratelimit::email::{TEST_EMAIL}"))
                .await
                .unwrap(),
            count
        );
    }

    /// Get a random photo either original, converted, Jack or Dave
    fn get_random_photo(app_env: &AppEnv, original: bool, jack: bool) -> String {
        let mut photo_names = vec![];
        let mut all_dirs = std::fs::read_dir(if original {
            &app_env.location.photo_original
        } else {
            &app_env.location.photo_converted
        })
        .unwrap();
        while let Some(Ok(i)) = all_dirs.next() {
            let mut dx = std::fs::read_dir(i.path()).unwrap();
            while let Some(Ok(x)) = dx.next() {
                let all_files = std::fs::read_dir(x.path()).unwrap();
                let all_names = all_files
                    .into_iter()
                    .map(|i| i.unwrap().file_name().to_str().unwrap().to_string())
                    .collect::<Vec<_>>();

                if let Some(x) = all_names
                    .iter()
                    .find(|i| i.chars().nth(26) == Some(if jack { '1' } else { '0' }))
                {
                    photo_names.push(x.to_owned());
                }
            }
        }
        photo_names.shuffle(&mut rand::thread_rng());
        photo_names[0].clone()
    }

    #[tokio::test]
    /// All files in the public folder are served with correct headers
    async fn static_router_serve_public() {
        let test_setup = start_both_servers().await;

        let client = reqwest::Client::new();

        let all_files = std::fs::read_dir(test_setup.app_env.location.public).unwrap();
        let all_names = all_files
            .into_iter()
            .map(|i| i.unwrap().file_name().to_str().unwrap().to_string())
            .collect::<Vec<_>>();

        for i in all_names {
            let url = format!(
                "http://{}:{}/{i}",
                test_setup.app_env.static_host, test_setup.app_env.static_port
            );
            let result = client.get(&url).send().await.unwrap();
            let headers = result.headers();
            assert_eq!(result.status(), StatusCode::OK);

            let cache_control = headers.get(CACHE_CONTROL);
            assert!(cache_control.is_some());
            assert_eq!(cache_control.unwrap(), "max-age=8640000");
            // ServeDir serves precompressed gzip/br variants, so must vary on accept-encoding
            let vary = headers.get(VARY);
            assert!(vary.is_some());
            assert_eq!(vary.unwrap(), "accept-encoding");
            assert!(headers.get(ACCESS_CONTROL_ALLOW_HEADERS).is_none());
            assert!(headers.get(ACCESS_CONTROL_ALLOW_CREDENTIALS).is_none());
        }

        let count: Option<usize> = test_setup
            .redis
            .get("ratelimit::ip::127.0.0.1")
            .await
            .unwrap();

        assert!(count.is_none());
    }

    /// Test VARY, ALLOW_CREDENTIALS, and ALLOW_ORIGIN
    fn test_header_map(headers: &HeaderMap, app_env: &AppEnv) {
        let vary = headers.get(VARY);
        assert!(vary.is_some());
        assert_eq!(
            vary.unwrap(),
            "accept-charset, accept-encoding, access-control-request-headers, access-control-request-method, origin"
        );

        let allow_creds = headers.get(ACCESS_CONTROL_ALLOW_CREDENTIALS);
        assert!(allow_creds.is_some());
        assert_eq!(allow_creds.unwrap(), "true");

        let allow_creds = headers.get(ACCESS_CONTROL_ALLOW_ORIGIN);
        assert!(allow_creds.is_some());
        assert_eq!(
            allow_creds.unwrap(),
            &format!("http://{}:8002", app_env.static_host)
        );
    }

    #[tokio::test]
    /// Unauthed user, invalid photo name returns 404
    async fn static_router_serve_photo_unauthed_random_file_name() {
        let test_setup = start_both_servers().await;

        let client = reqwest::Client::new();

        let photo_name = gen_random_hex(25);

        let url = format!(
            "http://{}:{}/photo/{photo_name}",
            test_setup.app_env.static_host, test_setup.app_env.static_port
        );
        let result = client.get(&url).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::NOT_FOUND);
        let headers = result.headers();

        let content_len = headers.get(CONTENT_LENGTH);
        assert!(content_len.is_some());
        assert_eq!(content_len.unwrap().to_str().unwrap(), "0");

        let cache_control = headers.get(CACHE_CONTROL);
        assert!(cache_control.is_some());
        assert_eq!(cache_control.unwrap(), "no-cache");

        test_header_map(headers, &test_setup.app_env);

        assert_unauth_limit(&test_setup, 1).await;
    }

    #[tokio::test]
    /// authed user, invalid photo name returns 404
    async fn static_router_serve_photo_authed_random_file_name() {
        let mut test_setup = start_both_servers().await;
        let cookie = test_setup.authed_user_cookie().await;

        let client = reqwest::Client::new();

        let photo_name = gen_random_hex(25);

        let url = format!(
            "http://{}:{}/photo/{photo_name}",
            test_setup.app_env.static_host, test_setup.app_env.static_port
        );
        let result = client
            .get(&url)
            .header("cookie", cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::NOT_FOUND);
        let headers = result.headers();

        let content_len = headers.get(CONTENT_LENGTH);
        assert!(content_len.is_some());
        assert_eq!(content_len.unwrap().to_str().unwrap(), "0");

        let cache_control = headers.get(CACHE_CONTROL);
        assert!(cache_control.is_some());
        assert_eq!(cache_control.unwrap(), "no-cache");

        test_header_map(headers, &test_setup.app_env);
        assert_unauth_limit(&test_setup, 1).await;
    }

    #[tokio::test]
    /// Unauthed user, valid photo name but no file returns 404
    async fn static_router_serve_photo_authed_no_photo() {
        let test_setup = start_both_servers().await;

        let client = reqwest::Client::new();
        let photo_name = format!("{}11.jpg", Ulid::generate().to_string().to_lowercase());

        let url = format!(
            "http://{}:{}/photo/{photo_name}",
            test_setup.app_env.static_host, test_setup.app_env.static_port
        );
        let result = client.get(&url).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::NOT_FOUND);
        let headers = result.headers();

        let content_len = headers.get(CONTENT_LENGTH);
        assert!(content_len.is_some());
        assert_eq!(content_len.unwrap().to_str().unwrap(), "0");

        let cache_control = headers.get(CACHE_CONTROL);
        assert!(cache_control.is_some());
        assert_eq!(cache_control.unwrap(), "no-cache");

        test_header_map(headers, &test_setup.app_env);
        assert_unauth_limit(&test_setup, 1).await;
    }

    #[tokio::test]
    /// Unauthed user, valid photo name but no file returns 404
    async fn static_router_serve_photo_unauthed_no_photo() {
        let mut test_setup = start_both_servers().await;
        let cookie = test_setup.authed_user_cookie().await;

        let client = reqwest::Client::new();
        let photo_name = format!("{}11.jpg", Ulid::generate().to_string().to_lowercase());

        let url = format!(
            "http://{}:{}/photo/{photo_name}",
            test_setup.app_env.static_host, test_setup.app_env.static_port
        );
        let result = client
            .get(&url)
            .header("cookie", cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::NOT_FOUND);
        let headers = result.headers();

        let content_len = headers.get(CONTENT_LENGTH);
        assert!(content_len.is_some());
        assert_eq!(content_len.unwrap().to_str().unwrap(), "0");

        let cache_control = headers.get(CACHE_CONTROL);
        assert!(cache_control.is_some());
        assert_eq!(cache_control.unwrap(), "no-cache");

        test_header_map(headers, &test_setup.app_env);
        assert_unauth_limit(&test_setup, 1).await;
    }

    #[tokio::test]
    /// Unauthed user can only access Jack converted images
    async fn static_router_serve_photos_unauthed() {
        let test_setup = start_both_servers().await;

        let test = |app_env: AppEnv, original: bool, jack: bool, valid: bool| async move {
            let client = reqwest::Client::new();

            let photo_name = get_random_photo(&app_env, original, jack);

            let url = format!(
                "http://{}:{}/photo/{photo_name}",
                app_env.static_host, app_env.static_port
            );
            let result = client.get(&url).send().await.unwrap();

            if valid {
                assert_eq!(result.status(), StatusCode::OK);
                let headers = result.headers();

                let content_type = headers.get(CONTENT_TYPE);
                assert!(content_type.is_some());
                assert_eq!(content_type.unwrap(), "image/webp");

                let content_len = headers.get(CONTENT_LENGTH);
                assert!(content_len.is_some());
                assert_eq!(
                    content_len.unwrap().to_str().unwrap(),
                    std::fs::File::open(format!(
                        "{}/{}/{}/{}",
                        app_env.location.photo_converted,
                        &photo_name[0..3],
                        &photo_name[3..6],
                        photo_name
                    ))
                    .unwrap()
                    .metadata()
                    .unwrap()
                    .len()
                    .to_string()
                );

                let cache_control = headers.get(CACHE_CONTROL);
                assert!(cache_control.is_some());
                assert_eq!(cache_control.unwrap(), "max-age=8640000");

                test_header_map(headers, &app_env);
            } else {
                assert_eq!(result.status(), StatusCode::NOT_FOUND);
                let headers = result.headers();

                let content_len = headers.get(CONTENT_LENGTH);
                assert!(content_len.is_some());
                assert_eq!(content_len.unwrap().to_str().unwrap(), "0");

                let cache_control = headers.get(CACHE_CONTROL);
                assert!(cache_control.is_some());
                assert_eq!(cache_control.unwrap(), "no-cache");

                test_header_map(headers, &app_env);
            }
        };

        let (a1, a2, a3, a4) = (
            C!(test_setup.app_env),
            C!(test_setup.app_env),
            C!(test_setup.app_env),
            C!(test_setup.app_env),
        );
        test(a1, false, false, false).await;
        assert_unauth_limit(&test_setup, 1).await;

        test(a2, true, true, false).await;
        assert_unauth_limit(&test_setup, 2).await;

        test(a3, true, false, false).await;
        assert_unauth_limit(&test_setup, 3).await;

        test(a4, false, true, true).await;
        assert_unauth_limit(&test_setup, 4).await;
    }

    #[tokio::test]
    /// Auth user can only access any photo
    async fn static_router_serve_photos_authed() {
        let test = |app_env: AppEnv, cookie: String, original: bool, jack: bool| async move {
            let client = reqwest::Client::new();
            let photo_name = get_random_photo(&app_env, original, jack);

            let url = format!(
                "http://{}:{}/photo/{photo_name}",
                app_env.static_host, app_env.static_port
            );
            let result = client
                .get(&url)
                .header("cookie", cookie)
                .send()
                .await
                .unwrap();

            assert_eq!(result.status(), StatusCode::OK);
            let headers = result.headers();

            let content_type = headers.get(CONTENT_TYPE);
            assert!(content_type.is_some());
            assert_eq!(
                content_type.unwrap(),
                if original { "image/jpeg" } else { "image/webp" }
            );

            let content_len = headers.get(CONTENT_LENGTH);
            assert!(content_len.is_some());
            assert_eq!(
                content_len.unwrap().to_str().unwrap(),
                std::fs::File::open(format!(
                    "{}/{}/{}/{}",
                    if original {
                        C!(app_env.location.photo_original)
                    } else {
                        C!(app_env.location.photo_converted)
                    },
                    &photo_name[0..3],
                    &photo_name[3..6],
                    photo_name
                ))
                .unwrap()
                .metadata()
                .unwrap()
                .len()
                .to_string()
            );

            let cache_control = headers.get(CACHE_CONTROL);
            assert!(cache_control.is_some());
            assert_eq!(
                cache_control.unwrap(),
                if !original && jack {
                    "max-age=8640000"
                } else {
                    "no-cache"
                }
            );

            test_header_map(headers, &app_env);
        };

        let mut test_setup = start_both_servers().await;
        let cookie = test_setup.authed_user_cookie().await;
        assert_unauth_limit(&test_setup, 1).await;

        let (a1, a2, a3, a4) = (
            C!(test_setup.app_env),
            C!(test_setup.app_env),
            C!(test_setup.app_env),
            C!(test_setup.app_env),
        );
        test(a1, C!(cookie), false, false).await;
        assert_authed_limit(&test_setup, 1).await;

        test(a2, C!(cookie), false, true).await;
        assert_authed_limit(&test_setup, 2).await;

        test(a3, C!(cookie), true, false).await;
        assert_authed_limit(&test_setup, 3).await;

        test(a4, C!(cookie), true, true).await;
        assert_authed_limit(&test_setup, 4).await;
    }
}
