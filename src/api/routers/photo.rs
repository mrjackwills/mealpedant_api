use std::fmt;
use tower_http::limit::RequestBodyLimitLayer;
use tracing::error;

use crate::{
    api::{
        authentication::is_admin, deserializer::IncomingDeserializer, ij, oj, ApiRouter,
        ApplicationState, Outgoing,
    }, api_error::ApiError, define_routes, photo_convertor::{Photo, PhotoConvertor}, C, S
};

use axum::{
    extract::{DefaultBodyLimit, Multipart, State},
    handler::Handler,
    middleware,
    routing::delete,
    Router,
};

const TEN_MB: usize = 10 * 1024 * 1024;

define_routes! {
    PhotoRoutes,
    "/photo",
    Base => ""
}

enum PhotoResponses {
    ImageInvalid,
}

impl fmt::Display for PhotoResponses {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let disp = match self {
            Self::ImageInvalid => S!("Image invalid"),
        };
        write!(f, "{disp}")
    }
}
pub struct PhotoRouter;

impl ApiRouter for PhotoRouter {
    fn create_router(state: &ApplicationState) -> Router<ApplicationState> {
        Router::new()
            .route(
                &PhotoRoutes::Base.addr(),
                delete(Self::photo_delete).post(
                    Self::photo_post
                        .layer(DefaultBodyLimit::disable())
                        .layer(RequestBodyLimitLayer::new(TEN_MB)),
                ),
            )
            .layer(middleware::from_fn_with_state(C!(state), is_admin))
    }
}

impl PhotoRouter {
    /// Only allow jpg images
    fn validate_mime_type(mime_type: &str) -> bool {
        mime_type.to_lowercase() == "image/jpeg" || mime_type.to_lowercase() == "image/jpg"
    }

    /// Convert, and save, an uploaded photo
    async fn photo_post(
        State(state): State<ApplicationState>,
        mut multipart: Multipart,
    ) -> Result<Outgoing<oj::Photo>, ApiError> {
        if let Some(field) = multipart.next_field().await? {
            let file_name = field
                .file_name()
                .unwrap_or_default()
                .to_string()
                .split_once('.')
                .unwrap_or_default()
                .0
                .to_owned();

            let content_type = field.content_type().unwrap_or_default().to_string();
            let data = field.bytes().await?;

            if !Self::validate_mime_type(&content_type)
                || !IncomingDeserializer::parse_photo_name(&file_name)
                || data.is_empty()
            {
                return Err(ApiError::InvalidValue(
                    PhotoResponses::ImageInvalid.to_string(),
                ));
            }

            let converted =
                PhotoConvertor::convert_photo(Photo { file_name, data }, &state.photo_env).await?;
            Ok((
                axum::http::StatusCode::OK,
                oj::OutgoingJson::new(oj::Photo {
                    converted: converted.converted,
                    original: converted.original,
                }),
            ))
        } else {
            Err(ApiError::InvalidValue(
                PhotoResponses::ImageInvalid.to_string(),
            ))
        }
    }

    /// Delete original & converted photos
    async fn photo_delete(
        State(state): State<ApplicationState>,
        ij::IncomingJson(body): ij::IncomingJson<ij::BothPhoto>,
    ) -> Result<axum::http::StatusCode, ApiError> {
        // this can be an issue regarding body length
        let converted_path = state.photo_env.get_path(body.converted);
        let original_path = state.photo_env.get_path(body.original);
        for path in [original_path, converted_path] {
            match tokio::fs::remove_file(path).await {
                Ok(()) => (),
                Err(e) => {
                    error!(%e);
                    return Err(ApiError::InvalidValue(S!("unknown image")));
                }
            }
        }
        Ok(axum::http::StatusCode::OK)
    }
}

/// Use reqwest to test against real server
/// cargo watch -q -c -w src/ -x 'test api_router_photo -- --test-threads=1 --nocapture'
#[cfg(test)]
#[expect(clippy::pedantic, clippy::unwrap_used)]
mod tests {

    use std::collections::HashMap;

    use reqwest::StatusCode;

    use super::{PhotoRouter, PhotoRoutes};
    use crate::api::api_tests::{base_url, start_server, Response};
    use crate::helpers::gen_random_hex;
    use crate::C;

    #[test]
    // Only allow jpg or JPEG as mime types
    fn mime_test() {
        // All Valid
        assert!(PhotoRouter::validate_mime_type("IMAGE/JPG"));
        assert!(PhotoRouter::validate_mime_type("IMAGE/JPEG"));
        assert!(PhotoRouter::validate_mime_type("image/jpg"));
        assert!(PhotoRouter::validate_mime_type("ImAGe/JPeG"));

        // All Invalid
        assert!(!PhotoRouter::validate_mime_type("img/jpg"));
        assert!(!PhotoRouter::validate_mime_type("image/gif"));
        assert!(!PhotoRouter::validate_mime_type(&gen_random_hex(8)));
    }

    #[tokio::test]
    // Unauthenticated user unable to access "/" route
    async fn api_router_photo_unauthenticated() {
        let test_setup = start_server().await;
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            PhotoRoutes::Base.addr()
        );
        let client = reqwest::Client::new();

        let result = client.post(&url).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");

        let result = client.delete(&url).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");
    }

    #[tokio::test]
    // non-admin user unable to access "/" route
    async fn api_router_photo_not_admin() {
        let mut test_setup = start_server().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            PhotoRoutes::Base.addr()
        );
        let client = reqwest::Client::new();

        let result = client
            .post(&url)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");

        let result = client
            .delete(&url)
            .header("cookie", authed_cookie)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");
    }

    #[tokio::test]
    /// Invalid image name
    async fn api_router_photo_post_invalid_name() {
        let mut test_setup = start_server().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.make_user_admin().await;
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            PhotoRoutes::Base.addr()
        );
        let client = reqwest::Client::new();

        let test_file =
            std::fs::read("/workspaces/mealpedant_api/docker/data/test_image.jpg").unwrap();
        let part = reqwest::multipart::Part::bytes(C!(test_file))
            .file_name("2022-01-01_J")
            .mime_str("image/jpeg")
            .unwrap();

        let form = reqwest::multipart::Form::new().part("file", part);

        let result = client
            .post(&url)
            .multipart(form)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::BAD_REQUEST);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!("Image invalid", result);

        let part = reqwest::multipart::Part::bytes(C!(test_file))
            .file_name("2022-01-01_I.jpg")
            .mime_str("image/jpeg")
            .unwrap();
        let form = reqwest::multipart::Form::new().part("file", part);
        let result = client
            .post(&url)
            .multipart(form)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::BAD_REQUEST);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!("Image invalid", result);

        let part = reqwest::multipart::Part::bytes(test_file)
            .file_name("2022-13-01_J.jpg")
            .mime_str("image/jpeg")
            .unwrap();
        let form = reqwest::multipart::Form::new().part("file", part);
        let result = client
            .post(&url)
            .multipart(form)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::BAD_REQUEST);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!("Image invalid", result);
    }

    #[tokio::test]
    /// Invalid image mime
    async fn api_router_photo_post_invalid_mime() {
        let mut test_setup = start_server().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.make_user_admin().await;
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            PhotoRoutes::Base.addr()
        );
        let client = reqwest::Client::new();

        let test_file =
            std::fs::read("/workspaces/mealpedant_api/docker/data/test_image.jpg").unwrap();
        let part = reqwest::multipart::Part::bytes(C!(test_file))
            .file_name("2022-01-01_J")
            .mime_str("imag/jpeg")
            .unwrap();

        let form = reqwest::multipart::Form::new().part("file", part);

        let result = client
            .post(&url)
            .multipart(form)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::BAD_REQUEST);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!("Image invalid", result);

        let part = reqwest::multipart::Part::bytes(C!(test_file))
            .file_name("2022-01-01_J")
            .mime_str("image/gif")
            .unwrap();

        let form = reqwest::multipart::Form::new().part("file", part);

        let result = client
            .post(&url)
            .multipart(form)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::BAD_REQUEST);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!("Image invalid", result);

        let part = reqwest::multipart::Part::bytes(C!(test_file))
            .file_name("2022-01-01_J")
            .mime_str("image/jpe")
            .unwrap();

        let form = reqwest::multipart::Form::new().part("file", part);

        let result = client
            .post(&url)
            .multipart(form)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::BAD_REQUEST);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!("Image invalid", result);
    }

    #[tokio::test]
    /// Invalid image size
    async fn api_router_photo_post_invalid_size() {
        let mut test_setup = start_server().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.make_user_admin().await;
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            PhotoRoutes::Base.addr()
        );
        let client = reqwest::Client::new();

        // empty
        let len = 0;
        let test_file = vec![0u8; len];

        let part = reqwest::multipart::Part::bytes(C!(test_file))
            .file_name("2022-01-01_J")
            .mime_str("image/jpeg")
            .unwrap();

        let form = reqwest::multipart::Form::new().part("file", part);

        let result = client
            .post(&url)
            .multipart(form)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::BAD_REQUEST);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!("Image invalid", result);

        //  > 10mb
        let len = 11 * 1024 * 1024;
        let test_file = vec![0u8; len];

        let part = reqwest::multipart::Part::bytes(C!(test_file))
            .file_name("2022-01-01_J")
            .mime_str("image/jpeg")
            .unwrap();

        let form = reqwest::multipart::Form::new().part("file", part);

        let result = client
            .post(&url)
            .multipart(form)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::PAYLOAD_TOO_LARGE);
        let result = result.text().await.unwrap();
        assert_eq!("length limit exceeded", result);
    }

    #[tokio::test]
    // no image, or multipart provided, error returned
    async fn api_router_photo_post_ok() {
        let mut test_setup = start_server().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.make_user_admin().await;
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            PhotoRoutes::Base.addr()
        );
        let client = reqwest::Client::new();

        let test_file =
            std::fs::read("/workspaces/mealpedant_api/docker/data/test_image.jpg").unwrap();
        let part = reqwest::multipart::Part::bytes(test_file)
            .file_name("2022-01-01_J.jpg")
            .mime_str("image/jpeg")
            .unwrap();

        let form = reqwest::multipart::Form::new().part("file", part);

        let result = client
            .post(&url)
            .multipart(form)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::OK);
        let result = result.json::<Response>().await.unwrap().response;
        assert!(result["converted"].is_string());
        assert!(result["original"].is_string());

        // Make sure original is on disk as a file
        let original_photo = std::fs::metadata(
            format!(
                "{}/{}",
                &test_setup.app_env.location_photo_original, result["original"]
            )
            .replace('"', ""),
        );
        assert!(original_photo.is_ok());
        assert!(original_photo.as_ref().unwrap().is_file());

        // Make sure converted is on disk as a file
        let converted_photo = std::fs::metadata(
            format!(
                "{}/{}",
                &test_setup.app_env.location_photo_converted, result["converted"]
            )
            .replace('"', ""),
        );
        assert!(converted_photo.is_ok());
        assert!(converted_photo.as_ref().unwrap().is_file());

        // Check original is larger file size than converted
        assert!(original_photo.unwrap().len() > converted_photo.unwrap().len());
    }

    #[tokio::test]
    // valid filename, but not on disk, returns 400 bad request
    async fn api_router_photo_delete_unknown_image() {
        let mut test_setup = start_server().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.make_user_admin().await;
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            PhotoRoutes::Base.addr()
        );
        let client = reqwest::Client::new();

        let body = HashMap::from([
            ("original", "2020-01-22_J_C_abcdef1234567890.jpg"),
            ("converted", "2020-01-22_J_O_abcdef1234567890.jpg"),
        ]);

        let result = client
            .delete(&url)
            .json(&body)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::BAD_REQUEST);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "unknown image");
    }

    #[tokio::test]
    // images correctly delete from disk
    async fn api_router_photo_delete_ok() {
        let mut test_setup = start_server().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.make_user_admin().await;
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            PhotoRoutes::Base.addr()
        );
        let client = reqwest::Client::new();

        let test_file =
            std::fs::read("/workspaces/mealpedant_api/docker/data/test_image.jpg").unwrap();
        let part = reqwest::multipart::Part::bytes(test_file)
            .file_name("2022-01-01_J.jpg")
            .mime_str("image/jpeg")
            .unwrap();

        let form = reqwest::multipart::Form::new().part("file", part);

        let result = client
            .post(&url)
            .multipart(form)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::OK);
        let result = result.json::<Response>().await.unwrap().response;
        let converted = result["converted"].to_string().replace('"', "");
        let original = result["original"].to_string().replace('"', "");

        let body = HashMap::from([("original", &original), ("converted", &converted)]);

        let result = client
            .delete(&url)
            .json(&body)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::OK);

        let original_path = format!(
            "{}/{}",
            &test_setup.app_env.location_photo_original, original
        );
        let converted_path = format!(
            "{}/{}",
            &test_setup.app_env.location_photo_converted, converted
        );
        assert!(std::fs::metadata(original_path).is_err());
        assert!(std::fs::metadata(converted_path).is_err());
    }
}
