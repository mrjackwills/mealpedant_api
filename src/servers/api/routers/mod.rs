mod admin;
mod food;
mod incognito;
mod meal;
mod photo;
mod user;

pub use admin::AdminRouter as Admin;
pub use food::FoodRouter as Food;
pub use incognito::IncognitoRouter as Incognito;
pub use meal::MealRouter as Meal;
pub use photo::PhotoRouter as Photo;
pub use user::UserRouter as User;
