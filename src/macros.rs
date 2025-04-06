/// Simple macro to create a new String, or convert from a &str to a String - basically just gets rid of String::from() / .to_owned() etc
#[macro_export]
macro_rules! S {
    () => {
        String::new()
    };
    ($s:expr) => {
        String::from($s)
    };
}

/// Simple macro to call `.clone()` on whatever is passed in
#[macro_export]
macro_rules! C {
    ($i:expr) => {
        $i.clone()
    };
}

#[macro_export]
/// Sleep for a given number of milliseconds, is an async fn.
/// If no parameter supplied, defaults to 1000ms
macro_rules! sleep {
    () => {
        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
    };
    ($ms:expr) => {
        tokio::time::sleep(std::time::Duration::from_millis($ms)).await;
    };
}

#[macro_export]
/// Return the internal server error, with a basic { response: "$prefix" }
macro_rules! internal {
    ($prefix:expr) => {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            OutgoingJson::new($prefix),
        )
    };
}

/// Generate a hashmap with a fixed key, used for redis hset
#[macro_export]
macro_rules! hmap {
    ($x:expr) => {{ std::collections::HashMap::from([(HASH_FIELD, $x)]) }};
}

#[macro_export]
/// Macro to convert a stringified struct back into the struct
macro_rules! redis_hash_to_struct {
    ($struct_name:ident) => {
        impl fred::types::FromValue for $struct_name {
            fn from_value(value: fred::prelude::Value) -> Result<Self, fred::prelude::Error> {
                value.as_str().map_or(
                    Err(fred::error::Error::new(
                        fred::error::ErrorKind::Parse,
                        format!("FromRedis: {}", stringify!(struct_name)),
                    )),
                    |i| {
                        serde_json::from_str::<Self>(&i).map_err(|_e| {
                            fred::error::Error::new(fred::error::ErrorKind::Parse, "serde")
                        })
                    },
                )
            }
        }
    };
}

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
