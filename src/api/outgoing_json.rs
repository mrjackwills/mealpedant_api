pub mod oj {
    use axum::Json;
    use serde::Serialize;

    use crate::database::{ModelFoodCategory, ModelMeal};

    pub type AsJsonRes<T> = Json<OutgoingJson<T>>;

    #[derive(serde::Serialize, Debug, PartialEq, PartialOrd)]
    pub struct OutgoingJson<T> {
        response: T,
    }

    impl<T> OutgoingJson<T> {
        pub fn new(response: T) -> Json<OutgoingJson<T>> {
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

    #[derive(Serialize)]
    pub struct LastId {
        pub last_id: i64,
    }

    #[derive(Serialize)]
    pub struct Categories {
        pub categories: Vec<ModelFoodCategory>,
    }

    #[derive(Serialize)]
    pub struct AuthenticatedUser {
        pub email: String,
        pub admin: bool,
        pub two_fa_active: bool,
        pub two_fa_always_required: bool,
        pub two_fa_count: i64,
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
                date: meal.meal_date.to_string(),
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

    #[derive(Serialize, Debug)]
    pub struct Limit {
        pub key: String,
        pub points: u64,
    }
}
