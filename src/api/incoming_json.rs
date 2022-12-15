pub mod ij {
    use crate::{
        api::deserializer::IncomingDeserializer as is,
        api_error::ApiError,
        database::{FromModel, ModelMeal, Person},
    };

    use std::{error::Error, fmt, net::IpAddr};

    use axum::{
        async_trait,
        extract::{rejection::JsonRejection, FromRequest, FromRequestParts},
        http::{request::Parts, Request},
    };
    use serde::{self, de::DeserializeOwned, Deserialize};
    use time::Date;
    use tracing::trace;

    #[cfg(test)]
    use serde::Serialize;

    /// attempt to extract the inner `serde_json::Error`, if that succeeds we can
    /// provide a more specific error
    // see https://docs.rs/axum/latest/axum/extract/index.html#accessing-inner-errors
    fn extract_serde_error<E>(e: E) -> ApiError
    where
        E: Error + 'static,
    {
        if let Some(err) = find_error_source::<serde_json::Error>(&e) {
            if err.to_string().contains("missing field") {
                return ApiError::MissingKey(
                    err.to_string()
                        .split_once("missing field `")
                        .unwrap_or_default()
                        .1
                        .split_once('`')
                        .unwrap_or_default()
                        .0
                        .trim()
                        .to_owned(),
                );
            } else if err.to_string().contains("unknown field") {
                return ApiError::InvalidValue("invalid input".to_owned());
            } else if err.to_string().contains("at line") {
                return ApiError::InvalidValue(
                    err.to_string()
                        .split_once("at line")
                        .unwrap_or_default()
                        .0
                        .trim()
                        .to_owned(),
                );
            }
        }
        ApiError::Internal("downcast".to_owned())
    }

    /// attempt to downcast `err` into a `T` and if that fails recursively try and
    /// downcast `err`'s source
    fn find_error_source<'a, T>(err: &'a (dyn Error + 'static)) -> Option<&'a T>
    where
        T: Error + 'static,
    {
        err.downcast_ref::<T>().map_or(
            err.source().and_then(|source| find_error_source(source)),
            Some,
        )
    }

    /// Two Factor Backup tokens can either be totp - [0-9]{6}, or backup tokens - [A-F0-9]{16}
    #[derive(Debug, Deserialize)]
    #[cfg_attr(test, derive(Serialize))]
    pub enum Token {
        Totp(String),
        Backup(String),
    }

    impl fmt::Display for Token {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let x = match self {
                Self::Totp(token) | Self::Backup(token) => token,
            };
            write!(f, "{x}")
        }
    }

    #[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
    #[cfg_attr(test, derive(Serialize))]
    pub enum PhotoName {
        Original(String),
        Converted(String),
    }

    impl fmt::Display for PhotoName {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let x = match self {
                Self::Original(token) | Self::Converted(token) => token,
            };
            write!(f, "{x}")
        }
    }

    #[derive(Debug, Deserialize, PartialEq, Eq)]
    #[cfg_attr(test, derive(Serialize))]
    pub enum LimitKey {
        Ip(IpAddr),
        Email(String),
    }

    pub struct IncomingJson<T>(pub T);

    /// Implement custom error handing for JSON extraction on incoming JSON
    /// Either return valid json (meeting a struct spec listed below), or return an ApiError
    /// Then each route handler, can use `IncomingJson(body): IncomingJson<T>`, to extract T into param body
    #[async_trait]
    impl<S, B, T> FromRequest<S, B> for IncomingJson<T>
    where
        axum::Json<T>: FromRequest<S, B, Rejection = JsonRejection>,
        S: Send + Sync,
        B: Send + 'static,
    {
        type Rejection = ApiError;

        async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
            match axum::Json::<T>::from_request(req, state).await {
                Ok(value) => Ok(Self(value.0)),
                Err(rejection) => match rejection {
                    JsonRejection::JsonDataError(e) => Err(extract_serde_error(e)),
                    JsonRejection::JsonSyntaxError(_) => {
                        Err(ApiError::InvalidValue("JSON".to_owned()))
                    }
                    JsonRejection::MissingJsonContentType(e) => {
                        trace!(%e);
                        Err(ApiError::InvalidValue(
                            "\"application/json\" header".to_owned(),
                        ))
                    }
                    JsonRejection::BytesRejection(e) => {
                        trace!(%e);
                        trace!("BytesRejection");
                        Err(ApiError::InvalidValue("Bytes Rejected".to_owned()))
                    }
                    _ => Err(ApiError::Internal(String::from(
                        "IncomingJson from_request error",
                    ))),
                },
            }
        }
    }
    pub struct Path<T>(pub T);

    #[async_trait]
    impl<S, T> FromRequestParts<S> for Path<T>
    where
        // these trait bounds are copied from `impl FromRequest for axum::extract::path::Path`
        T: DeserializeOwned + Send,
        S: Send + Sync,
    {
        type Rejection = ApiError;
        async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
            match axum::extract::Path::<T>::from_request_parts(parts, state).await {
                Ok(value) => Ok(Self(value.0)),
                Err(e) => Err(ApiError::InvalidValue(format!("invalid {e} param"))),
            }
        }
    }

    #[derive(Deserialize, Debug)]
    #[serde(deny_unknown_fields)]
    pub struct Register {
        #[serde(deserialize_with = "is::name")]
        pub full_name: String,
        #[serde(deserialize_with = "is::email")]
        pub email: String,
        #[serde(deserialize_with = "is::password")]
        pub password: String,
        #[serde(deserialize_with = "is::invite")]
        pub invite: String,
    }

    #[derive(Deserialize, Debug)]
    #[serde(deny_unknown_fields)]
    pub struct Signin {
        #[serde(deserialize_with = "is::email")]
        pub email: String,
        pub password: String,
        #[serde(default)]
        #[serde(deserialize_with = "is::option_token")]
        pub token: Option<Token>,
        pub remember: bool,
    }

    #[derive(Deserialize, Debug)]
    #[serde(deny_unknown_fields)]
    pub struct Reset {
        #[serde(deserialize_with = "is::email")]
        pub email: String,
    }

    #[derive(Deserialize, Debug)]
    #[serde(deny_unknown_fields)]
    pub struct PasswordToken {
        #[serde(deserialize_with = "is::password")]
        pub password: String,
        #[serde(default)]
        #[serde(deserialize_with = "is::option_token")]
        pub token: Option<Token>,
    }

    #[derive(Deserialize, Debug)]
    #[serde(deny_unknown_fields)]
    pub struct TwoFA {
        #[serde(deserialize_with = "is::token")]
        pub token: Token,
    }

    #[derive(Deserialize, Debug)]
    #[serde(deny_unknown_fields)]
    pub struct TwoFAAlwaysRequired {
        #[serde(default)]
        #[serde(deserialize_with = "is::option_password")]
        pub password: Option<String>,
        pub always_required: bool,
        #[serde(default)]
        #[serde(deserialize_with = "is::option_token")]
        pub token: Option<Token>,
    }

    #[derive(Deserialize, Debug)]
    #[serde(deny_unknown_fields)]
    pub struct BothPhoto {
        #[serde(deserialize_with = "is::photo_name_hex")]
        pub original: PhotoName,
        #[serde(deserialize_with = "is::photo_name_hex")]
        pub converted: PhotoName,
    }

    #[derive(Deserialize, Debug)]
    #[serde(deny_unknown_fields)]
    pub struct PatchPassword {
        #[serde(deserialize_with = "is::password")]
        pub current_password: String,
        #[serde(deserialize_with = "is::password")]
        pub new_password: String,
        #[serde(default)]
        #[serde(deserialize_with = "is::option_token")]
        pub token: Option<Token>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct DatePerson {
        #[serde(deserialize_with = "is::date")]
        pub date: Date,
        #[serde(deserialize_with = "is::person")]
        pub person: Person,
    }

    #[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
    #[serde(deny_unknown_fields)]
    pub struct Meal {
        #[serde(deserialize_with = "is::date")]
        pub date: Date,
        pub category: String,
        #[serde(deserialize_with = "is::person")]
        pub person: Person,
        pub restaurant: bool,
        pub takeaway: bool,
        pub vegetarian: bool,
        pub description: String,
        #[serde(deserialize_with = "is::option_photo_name_hex")]
        #[serde(default)]
        pub photo_original: Option<PhotoName>,
        #[serde(deserialize_with = "is::option_photo_name_hex")]
        #[serde(default)]
        pub photo_converted: Option<PhotoName>,
    }

    impl FromModel<ModelMeal> for Meal {
        type Item = Self;
        fn from_model(meal: ModelMeal) -> Result<Self, ApiError> {
            Ok(Self {
                date: meal.meal_date,
                category: meal.category,
                person: Person::try_from(meal.person.as_str())?,
                restaurant: meal.restaurant,
                takeaway: meal.takeaway,
                vegetarian: meal.vegetarian,
                description: meal.description,
                photo_original: meal.photo_original.map(PhotoName::Original),
                photo_converted: meal.photo_converted.map(PhotoName::Converted),
            })
        }
    }

    impl FromModel<&ModelMeal> for Meal {
        type Item = Self;
        fn from_model(meal: &ModelMeal) -> Result<Self, ApiError> {
            Ok(Self {
                date: meal.meal_date,
                category: meal.category.clone(),
                person: Person::try_from(meal.person.as_str())?,
                restaurant: meal.restaurant,
                takeaway: meal.takeaway,
                vegetarian: meal.vegetarian,
                description: meal.description.clone(),
                photo_original: meal
                    .photo_original
                    .as_ref()
                    .map(|original| PhotoName::Original(original.clone())),
                photo_converted: meal
                    .photo_converted
                    .as_ref()
                    .map(|converted| PhotoName::Converted(converted.clone())),
            })
        }
    }

    #[derive(Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct MealPatch {
        #[serde(deserialize_with = "is::date")]
        pub original_date: Date,
        pub meal: Meal,
    }

    #[derive(Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct BackupPost {
        pub with_photos: bool,
    }

    #[derive(Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct BackupDelete {
        #[serde(deserialize_with = "is::backup_name")]
        pub file_name: String,
    }

    #[derive(Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    #[cfg_attr(test, derive(Serialize))]
    pub struct UserPatch {
        pub active: Option<bool>,
        pub attempt: Option<bool>,
        #[serde(default, deserialize_with = "is::option_id")]
        pub password_reset_id: Option<i64>,
        pub reset: Option<bool>,
        pub two_fa_secret: Option<bool>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    #[cfg_attr(test, derive(Serialize))]
    pub struct AdminUserPatch {
        pub patch: UserPatch,
        #[serde(deserialize_with = "is::email")]
        pub email: String,
    }

    #[derive(Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    #[cfg_attr(test, derive(Serialize))]
    pub struct LimitDelete {
        #[serde(deserialize_with = "is::limit")]
        pub key: LimitKey,
    }

    #[derive(Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    #[cfg_attr(test, derive(Serialize))]
    pub struct EmailPost {
        #[serde(deserialize_with = "is::vec_email")]
        pub emails: Vec<String>,
        pub title: String,
        pub line_one: String,
        pub line_two: Option<String>,
        pub button_text: Option<String>,
        pub link: Option<String>,
    }
}
