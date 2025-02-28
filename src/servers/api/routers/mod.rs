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

// Define a set of routes as an enum with a base path.
// This macro generates the enum and an `addr()` method to get the route address.
// Usage: define_routes! { EnumName, BasePath, Variant1 => "route1", Variant2 => "route2", ... }
#[macro_export]
macro_rules! define_routes {
    ($enum_name:ident, $base_path:expr, $($variant:ident => $route:expr),*) => {
        enum $enum_name {
            $($variant,)*
        }

        impl $enum_name {
            fn addr(&self) -> String {
                let route_name = match self {
                    $(Self::$variant => $route,)*
                };
                format!("{}{}", $base_path, route_name)
            }
        }
    };
}
