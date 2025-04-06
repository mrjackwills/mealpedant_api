use axum::{Router, extract::State, middleware, routing::get};

use crate::{
    C,
    api::{ApiRouter, ApiState},
    api_error::ApiError,
    database::MealResponse,
    define_routes,
    servers::{
        Outgoing,
        authentication::is_authenticated,
        oj::{self, MealInfo},
    },
};

define_routes! {
    FoodRoutes,
    "/food",
    All => "/all",
    Hash => "/hash"
}

pub struct FoodRouter;

impl ApiRouter for FoodRouter {
    fn create_router(state: &ApiState) -> Router<ApiState> {
        Router::new()
            .route(&FoodRoutes::All.addr(), get(Self::all_get))
            .route(&FoodRoutes::Hash.addr(), get(Self::hash_get))
            .layer(middleware::from_fn_with_state(C!(state), is_authenticated))
    }
}

impl FoodRouter {
    /// get individual meals, sorted by date
    async fn all_get(State(state): State<ApiState>) -> Result<Outgoing<MealInfo>, ApiError> {
        Ok((
            axum::http::StatusCode::OK,
            oj::OutgoingJson::new(
                MealResponse::get_all(&state.postgres, &state.redis, Some(())).await?,
            ),
        ))
    }

    /// Just return the last id from the individual_meal_audit
    async fn hash_get(State(state): State<ApiState>) -> Result<Outgoing<String>, ApiError> {
        Ok((
            axum::http::StatusCode::OK,
            oj::OutgoingJson::new(
                MealResponse::get_hash(&state.postgres, &state.redis, Some(())).await?,
            ),
        ))
    }
}

// Use reqwest to test against real server
// cargo watch -q -c -w src/ -x 'test api_router_food -- --test-threads=1 --nocapture'
#[cfg(test)]
#[expect(clippy::unwrap_used)]
mod tests {

    use super::FoodRoutes;
    use crate::servers::{
        api_tests::{Response, base_url, start_both_servers},
        deserializer::IncomingDeserializer,
    };

    use fred::interfaces::{HashesInterface, KeysInterface};
    use reqwest::StatusCode;

    #[tokio::test]
    /// Unauthenticated user unable to access "/all" route
    async fn api_router_food_all_unauthenticated() {
        let test_setup = start_both_servers().await;
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            FoodRoutes::All.addr()
        );
        let client = reqwest::Client::new();

        let result = client.get(&url).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");
    }

    #[tokio::test]
    /// Get the food all food (descriptions + person + date) object, check that it gets inserted into redis cache
    #[allow(clippy::too_many_lines)]
    async fn api_router_food_all_ok() {
        let mut test_setup = start_both_servers().await;
        let authed_cookie = test_setup.authed_user_cookie().await;

        let client = reqwest::Client::new();

        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            FoodRoutes::All.addr()
        );

        let result = client
            .get(url)
            .header("cookie", authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::OK);

        let result = result.json::<Response>().await.unwrap().response;

        let descriptions = result.get("d");
        assert!(descriptions.is_some());
        let descriptions = descriptions.unwrap().as_object().unwrap();

        for (id, item) in descriptions {
            assert!(id.parse::<u64>().is_ok());
            assert!(item.as_str().is_some());
        }

        let categories = result.get("c");
        assert!(categories.is_some());
        let categories = categories.unwrap().as_object().unwrap();

        for (id, item) in categories {
            assert!(id.parse::<u64>().is_ok());
            assert!(item.is_string());
            assert!(
                item.as_str()
                    .unwrap()
                    .replace(' ', "")
                    .chars()
                    .all(char::is_uppercase)
            );
        }

        let meal_dates = result.get("m");
        assert!(meal_dates.is_some());
        let meal_dates = meal_dates.unwrap().as_array().unwrap();

        assert!(meal_dates.len() > 200);

        for i in meal_dates {
            // assert each has a d, and j object, and a c object, and each j & d object should have a c, and m
            let entry = i.as_object().unwrap();
            let meal_date = entry.get("a");
            assert!(meal_date.is_some());
            let meal_date = meal_date.unwrap();
            assert!(meal_date.is_string());
            assert!(meal_date.as_str().unwrap().chars().count() == 6);
            assert!(meal_date.as_str().unwrap().chars().all(char::is_numeric));

            for i in ["d", "j"] {
                let person = entry.get(i);
                assert!(person.is_some());
                let person = person.unwrap();
                assert!(person.is_object());
                let person = person.as_object().unwrap();
                assert!(person.get("m").is_some());
                assert!(person.get("m").unwrap().is_i64());
                let m_id = person.get("m").unwrap().as_i64().unwrap();
                assert!(descriptions.contains_key(&m_id.to_string()));

                assert!(person.get("c").is_some());
                assert!(person.get("c").unwrap().is_i64());
                let c_id = person.get("c").unwrap().as_i64().unwrap();
                assert!(categories.contains_key(&c_id.to_string()));

                for i in ["v", "t", "r"] {
                    if let Some(v) = person.get(i) {
                        assert!(v.as_i64().unwrap() == 1);
                    }
                }

                if let Some(p) = person.get("p") {
                    assert!(p.is_object());
                    let p = p.as_object().unwrap();

                    if let Some(original) = p.get("o") {
                        let original = original.as_str().unwrap();
                        assert_eq!(original.chars().nth(27), Some('0'));
                        assert!(
                            std::path::Path::new(original)
                                .extension()
                                .is_some_and(|ext| ext.eq_ignore_ascii_case("jpg"))
                        );

                        let converted = p.get("c");
                        assert!(converted.is_some());
                        let converted = converted.unwrap().as_str().unwrap();
                        assert_eq!(converted.chars().nth(27), Some('1'));
                        assert!(
                            std::path::Path::new(converted)
                                .extension()
                                .is_some_and(|ext| ext.eq_ignore_ascii_case("jpg"))
                        );
                    }
                }
            }
        }

        // Check redis cache
        let redis_cache: Option<String> = test_setup
            .redis
            .hget("cache::all_meals", "data")
            .await
            .unwrap();
        assert!(redis_cache.is_some());
    }

    #[tokio::test]
    /// Unauthenticated user unable to access "/last" route
    async fn api_router_food_hash_unauthenticated() {
        let test_setup = start_both_servers().await;
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            FoodRoutes::Hash.addr()
        );
        let client = reqwest::Client::new();

        let result = client.get(&url).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");
        let all_meals_cache: Option<String> =
            test_setup.redis.get("cache::all_meals").await.unwrap();
        assert!(all_meals_cache.is_none());
    }

    #[tokio::test]
    /// Get the current hash of all meals, check that it gets inserted into redis cache
    async fn api_router_food_hash_ok() {
        let mut test_setup = start_both_servers().await;
        let authed_cookie = test_setup.authed_user_cookie().await;

        let client = reqwest::Client::new();

        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            FoodRoutes::Hash.addr()
        );

        let result = client
            .get(url)
            .header("cookie", authed_cookie)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::OK);
        let result = result.json::<Response>().await.unwrap().response;

        assert!(result.is_string());
        let result = result.as_str().unwrap();
        assert!(IncomingDeserializer::is_hex(result, 64));

        // Check redis cache
        let redis_cache: Option<String> =
            test_setup.redis.get("cache::all_meals_hash").await.unwrap();
        assert!(redis_cache.is_some());
        assert_eq!(redis_cache.unwrap(), result);
    }
}
