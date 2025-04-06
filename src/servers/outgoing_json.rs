pub mod oj {
    use std::collections::HashMap;

    use axum::Json;
    use serde::{Deserialize, Serialize};

    use crate::{
        S,
        api_error::ApiError,
        database::{ModelDateMeal, ModelMeal, ModelMissingFood, ModelUser, Person},
    };

    pub type AsJsonRes<T> = Json<OutgoingJson<T>>;

    /// Used to skip serializtion if value is None or false
    #[expect(clippy::trivially_copy_pass_by_ref, clippy::ref_option)]
    pub fn none_or_false(x: &Option<bool>) -> bool {
        if let Some(value) = x {
            return !value;
        }
        true
    }

    /// Used to skip serializtion if value is None or 0
    #[expect(clippy::trivially_copy_pass_by_ref, clippy::ref_option)]
    pub fn none_or_zero(x: &Option<i32>) -> bool {
        if let Some(value) = x {
            return value == &0;
        }
        true
    }

    #[derive(serde::Serialize, Debug, PartialEq, Eq, PartialOrd)]
    pub struct OutgoingJson<T> {
        response: T,
    }

    impl<T> OutgoingJson<T> {
        pub const fn new(response: T) -> Json<Self> {
            Json(Self { response })
        }
    }

    #[derive(Serialize)]
    pub struct Online {
        pub uptime: u64,
        pub api_version: String,
    }

    #[derive(Serialize)]
    pub struct PasswordReset {
        pub two_fa_active: bool,
        pub two_fa_backup: bool,
    }

    #[derive(Serialize)]
    pub struct SigninAccepted {
        pub two_fa_backup: bool,
    }

    #[derive(Serialize)]
    pub struct Photo {
        pub converted: String,
        pub original: String,
    }

    #[derive(Debug, Clone, Serialize, PartialEq, Eq)]
    pub struct MissingFood {
        pub date: String,
        pub person: Person,
    }

    impl MissingFood {
        pub fn from_model(data: &[ModelMissingFood]) -> Result<Vec<Self>, ApiError> {
            let mut output = vec![];
            for entry in data {
                output.push(Self {
                    date: entry.missing_date.to_jiff().to_string(),
                    person: Person::try_from(entry.person.as_str())?,
                });
            }
            Ok(output)
        }
    }

    #[derive(Serialize)]
    pub struct AuthenticatedUser {
        pub email: String,
        pub admin: bool,
        pub two_fa_active: bool,
        pub two_fa_always_required: bool,
        pub two_fa_count: i64,
    }

    impl From<ModelUser> for AuthenticatedUser {
        fn from(user: ModelUser) -> Self {
            Self {
                email: user.email,
                admin: user.admin,
                two_fa_active: user.two_fa_secret.is_some(),
                two_fa_always_required: user.two_fa_always_required,
                two_fa_count: user.two_fa_backup_count,
            }
        }
    }

    #[derive(Serialize)]
    pub struct TwoFASetup {
        pub secret: String,
    }

    #[derive(Serialize)]
    pub struct TwoFaBackup {
        pub backups: Vec<String>,
    }

    #[derive(Serialize)]
    pub struct Meal {
        pub date: String,
        pub category: String,
        pub person: String,
        pub restaurant: bool,
        pub takeaway: bool,
        pub vegetarian: bool,
        pub description: String,
        pub photo_original: Option<String>,
        pub photo_converted: Option<String>,
    }

    impl From<ModelMeal> for Meal {
        fn from(meal: ModelMeal) -> Self {
            Self {
                date: meal.meal_date.to_jiff().to_string(),
                category: meal.category,
                person: meal.person,
                restaurant: meal.restaurant,
                takeaway: meal.takeaway,
                vegetarian: meal.vegetarian,
                description: meal.description,
                photo_original: meal.photo_original,
                photo_converted: meal.photo_converted,
            }
        }
    }

    #[derive(Serialize)]
    pub struct BackupFile {
        pub file_name: String,
        pub file_size: u64,
    }

    #[derive(Serialize)]
    pub struct Backups {
        pub backups: Vec<BackupFile>,
    }

    #[derive(Serialize)]
    pub struct AdminMeal {
        pub meal: Option<Meal>,
    }

    #[derive(Serialize)]
    pub struct AdminMemory {
        pub uptime: u64,
        pub uptime_app: u64,
        pub virt: usize,
        pub rss: usize,
    }

    #[derive(Debug, Serialize)]
    pub struct AdminPhoto {
        pub file_name_original: Option<String>,
        pub file_name_converted: Option<String>,
        pub size_in_bytes_original: Option<u64>,
        pub size_in_bytes_converted: Option<u64>,
        pub person: Option<String>,
        pub meal_date: Option<String>,
    }

    #[derive(Serialize, Debug)]
    pub struct Limit {
        pub key: String,
        pub points: u64,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Logs {
        pub timestamp: String,
        pub level: String,
        pub fields: Option<HashMap<String, String>>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash, Eq)]
    pub struct PersonPhoto {
        #[serde(rename = "o", skip_serializing_if = "Option::is_none")]
        pub original: Option<String>,
        #[serde(rename = "c", skip_serializing_if = "Option::is_none")]
        pub converted: Option<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash, Eq)]
    pub struct PersonFood {
        #[serde(rename = "md")]
        // this is an usize relating to an ID
        pub meal_description: String,
        #[serde(rename = "c")]
        pub category: i64,
        #[serde(rename = "r", skip_serializing_if = "none_or_false")]
        pub restaurant: Option<bool>,
        #[serde(rename = "v", skip_serializing_if = "none_or_false")]
        pub vegetarian: Option<bool>,
        #[serde(rename = "t", skip_serializing_if = "none_or_false")]
        pub takeaway: Option<bool>,
        #[serde(rename = "p", skip_serializing_if = "Option::is_none")]
        pub photo: Option<PersonPhoto>,
    }

    #[allow(non_snake_case)]
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash, Eq)]
    pub struct IndividualFoodJson {
        #[serde(rename = "da")]
        pub date: String,
        #[serde(rename = "D", skip_serializing_if = "Option::is_none")]
        pub Dave: Option<PersonFood>,
        #[serde(rename = "J", skip_serializing_if = "Option::is_none")]
        pub Jack: Option<PersonFood>,
    }

    pub type MealDescriptionMap = HashMap<i64, String>;
    pub type MealCategoryMap = HashMap<i64, String>;

    #[allow(non_snake_case)]
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash, Eq)]
    pub struct DateMeal {
        #[serde(rename = "a")]
        pub date: String,
        #[serde(rename = "d", skip_serializing_if = "Option::is_none")]
        pub Dave: Option<PersonMeal>,
        #[serde(rename = "j", skip_serializing_if = "Option::is_none")]
        pub Jack: Option<PersonMeal>,
    }

    impl From<ModelDateMeal> for DateMeal {
        fn from(value: ModelDateMeal) -> Self {
            // TODO if let Some chain here
            let photo = if let (Some(original), Some(converted)) =
                (&value.photo_original, &value.photo_converted)
            {
                Some(PersonPhoto {
                    original: Some(S!(original)),
                    converted: Some(S!(converted)),
                })
            } else {
                value.photo_converted.map(|converted| PersonPhoto {
                    original: None,
                    converted: Some(converted),
                })
            };

            if value.person == "Jack" {
                Self {
                    Dave: None,
                    Jack: Some(PersonMeal {
                        meal_description_id: value.meal_description_id,
                        category_id: value.meal_category_id,
                        restaurant: value.restaurant,
                        vegetarian: value.vegetarian,
                        takeaway: value.takeaway,
                        photo,
                    }),
                    date: value
                        .date_of_meal
                        .chars()
                        .skip(2)
                        .collect::<String>()
                        .replace('-', ""),
                }
            } else {
                Self {
                    Dave: Some(PersonMeal {
                        meal_description_id: value.meal_description_id,
                        category_id: value.meal_category_id,
                        restaurant: value.restaurant,
                        vegetarian: value.vegetarian,
                        takeaway: value.takeaway,
                        photo,
                    }),
                    Jack: None,
                    date: value
                        .date_of_meal
                        .chars()
                        .skip(2)
                        .collect::<String>()
                        .replace('-', ""),
                }
            }
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash, Eq)]
    pub struct PersonMeal {
        #[serde(rename = "m")]
        pub meal_description_id: i64,
        #[serde(rename = "c")]
        pub category_id: i64,
        #[serde(rename = "r", skip_serializing_if = "none_or_zero")]
        pub restaurant: Option<i32>,
        #[serde(rename = "v", skip_serializing_if = "none_or_zero")]
        pub vegetarian: Option<i32>,
        #[serde(rename = "t", skip_serializing_if = "none_or_zero")]
        pub takeaway: Option<i32>,
        #[serde(rename = "p", skip_serializing_if = "Option::is_none")]
        pub photo: Option<PersonPhoto>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct MealInfo {
        #[serde(rename = "d")]
        pub meal_descriptions: MealDescriptionMap,
        #[serde(rename = "c")]
        pub meal_categories: MealCategoryMap,
        #[serde(rename = "m")]
        pub date_meals: Vec<DateMeal>,
    }
}
