use axum::{
    extract::State,
    middleware,
    routing::{delete, get},
    Router,
};
use reqwest::StatusCode;

use crate::{
    api::{
        authentication::{is_admin, is_authenticated},
        oj, ApiRouter, ApplicationState, Outgoing,
    },
    api_error::ApiError,
    database::{
        IndividualFoodJson, ModelFoodCategory, ModelFoodLastId, ModelIndividualFood, ModelMeal,
    },
    define_routes,
};

define_routes! {
    FoodRoutes,
    "/food",
    All => "/all",
    Cache => "/cache",
    Category => "/category",
    Last => "/last"
}

pub struct FoodRouter;

impl ApiRouter for FoodRouter {
    fn create_router(state: &ApplicationState) -> Router<ApplicationState> {
        Router::new()
            .route(&FoodRoutes::All.addr(), get(Self::all_get))
            .route(&FoodRoutes::Category.addr(), get(Self::category_get))
            .route(&FoodRoutes::Last.addr(), get(Self::last_get))
            // Never need the user object in any of the routes, can can just blanket apply is_authenticated to all routes
            .layer(middleware::from_fn_with_state(
                state.clone(),
                is_authenticated,
            ))
            .route(
                &FoodRoutes::Cache.addr(),
                delete(Self::cache_delete)
                    .layer(middleware::from_fn_with_state(state.clone(), is_admin)),
            )
    }
}

impl FoodRouter {
    /// get individual meals, sorted by date
    async fn all_get(
        State(state): State<ApplicationState>,
    ) -> Result<Outgoing<Vec<IndividualFoodJson>>, ApiError> {
        Ok((
            axum::http::StatusCode::OK,
            oj::OutgoingJson::new(
                ModelIndividualFood::get_all(&state.postgres, &state.redis).await?,
            ),
        ))
    }

    /// Delete the all meals cache - only available to admin users
    async fn cache_delete(State(state): State<ApplicationState>) -> Result<StatusCode, ApiError> {
        ModelMeal::delete_cache(&state.redis).await?;
        Ok(axum::http::StatusCode::OK)
    }

    /// Get vec of all categories, includes ID and also counts for each
    async fn category_get(
        State(state): State<ApplicationState>,
    ) -> Result<Outgoing<oj::Categories>, ApiError> {
        Ok((
            axum::http::StatusCode::OK,
            oj::OutgoingJson::new(oj::Categories {
                categories: ModelFoodCategory::get_all(&state.postgres, &state.redis).await?,
            }),
        ))
    }

    /// Just return the last id from the individual_meal_audit
    async fn last_get(
        State(state): State<ApplicationState>,
    ) -> Result<Outgoing<oj::LastId>, ApiError> {
        Ok((
            axum::http::StatusCode::OK,
            oj::OutgoingJson::new(oj::LastId {
                last_id: ModelFoodLastId::get(&state.postgres, &state.redis)
                    .await?
                    .last_id,
            }),
        ))
    }
}

// Use reqwest to test against real server
// cargo watch -q -c -w src/ -x 'test api_router_food -- --test-threads=1 --nocapture'
#[cfg(test)]
#[allow(clippy::pedantic, clippy::nursery, clippy::unwrap_used)]
mod tests {

    use super::FoodRoutes;
    use crate::api::api_tests::{base_url, start_server, Response};

    use redis::AsyncCommands;
    use reqwest::StatusCode;

    #[tokio::test]
    // Unauthenticated user unable to access "/cache" route
    async fn api_router_food_cache_unauthenticated() {
        let test_setup = start_server().await;
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            FoodRoutes::Cache.addr()
        );
        let client = reqwest::Client::new();

        let result = client.delete(&url).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");
    }

    #[tokio::test]
    // Authenticated, but not admin user, unable to access "/cache" route
    async fn api_router_food_cache_not_admin() {
        let test_setup = start_server().await;
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            FoodRoutes::Cache.addr()
        );
        let client = reqwest::Client::new();

        let result = client.delete(&url).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");
    }

    #[tokio::test]
    // Delete all food caches, redis keys no longer there
    async fn api_router_food_cache_admin_valid() {
        let mut test_setup = start_server().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.make_user_admin().await;
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            FoodRoutes::Cache.addr()
        );
        let client = reqwest::Client::new();

        let result = client
            .delete(&url)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::OK);

        let all_meals_cache: Option<String> = test_setup
            .redis
            .lock()
            .await
            .hget("cache::all_meals", "data")
            .await
            .unwrap();
        assert!(all_meals_cache.is_none());

        // Check redis cache
        let category_cache: Option<String> = test_setup
            .redis
            .lock()
            .await
            .hget("cache::category", "data")
            .await
            .unwrap();
        assert!(category_cache.is_none());

        // Check redis cache
        let las_id_cache: Option<i64> = test_setup
            .redis
            .lock()
            .await
            .get("cache::last_id")
            .await
            .unwrap();
        assert!(las_id_cache.is_none());
    }

    #[tokio::test]
    // Unauthenticated user unable to access "/all" route
    async fn api_router_food_all_unauthenticated() {
        let test_setup = start_server().await;
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
    // Get the food all food (descriptions + person + date) object, check that it gets inserted into redis cache
    async fn api_router_food_all_ok() {
        let mut test_setup = start_server().await;
        let authed_cookie = test_setup.authed_user_cookie().await;

        let client = reqwest::Client::new();

        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            FoodRoutes::All.addr()
        );

        // Make two request, to make sure the cache is used and works
        client
            .get(&url)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();

        let result = client
            .get(url)
            .header("cookie", authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::OK);

        let result = result.json::<Response>().await.unwrap().response;
        assert!(result.is_array());

        // has at least 100 meals
        assert!(result.as_array().as_ref().unwrap().len() > 20);

        let result = result.as_array().as_ref().unwrap()[0].clone();

        assert!(result["D"]["md"].is_string());
        assert!(result["D"]["c"].is_i64());

        assert!(result["J"]["md"].is_string());
        assert!(result["J"]["c"].is_i64());

        assert!(result["da"].is_string());

        // Check redis cache
        let redis_cache: Option<String> = test_setup
            .redis
            .lock()
            .await
            .hget("cache::all_meals", "data")
            .await
            .unwrap();
        assert!(redis_cache.is_some());
    }

    #[tokio::test]
    // Unauthenticated user unable to access "/category" route
    async fn api_router_food_category_unauthenticated() {
        let test_setup = start_server().await;
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            FoodRoutes::Category.addr()
        );
        let client = reqwest::Client::new();

        let result = client.get(&url).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");
    }

    #[tokio::test]
    // Get the categories object, check that it gets inserted into redis cache
    async fn api_router_food_category_ok() {
        let mut test_setup = start_server().await;
        let authed_cookie = test_setup.authed_user_cookie().await;

        let client = reqwest::Client::new();

        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            FoodRoutes::Category.addr()
        );
        // Make two request, to make sure the cache is used and works
        client
            .get(&url)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();

        let result = client
            .get(url)
            .header("cookie", authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::OK);

        let result = result.json::<Response>().await.unwrap().response;
        assert!(result["categories"].is_array());

        // has atleast categories
        assert!(result["categories"].as_array().as_ref().unwrap().len() > 20);

        // First category has valid id, c (category) and n (count)
        // Should maybe choose a random one rather than the first?
        let category = &result["categories"].as_array().as_ref().unwrap()[0];
        assert!(category["id"].is_number());
        assert!(category["id"].as_i64().unwrap() > 1);
        assert!(category["c"].is_string());
        assert!(category["n"].is_number());
        assert!(category["n"].as_i64().unwrap() > 1);

        // Check redis cache
        let redis_cache: Option<String> = test_setup
            .redis
            .lock()
            .await
            .hget("cache::category", "data")
            .await
            .unwrap();
        assert!(redis_cache.is_some());
    }

    #[tokio::test]
    // Unauthenticated user unable to access "/last" route
    async fn api_router_food_last_unauthenticated() {
        let test_setup = start_server().await;
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            FoodRoutes::Last.addr()
        );
        let client = reqwest::Client::new();

        let result = client.get(&url).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");
    }

    #[tokio::test]
    // Get the latest id of a meal object, check that it gets inserted into redis cache
    async fn api_router_food_last_ok() {
        let mut test_setup = start_server().await;
        let authed_cookie = test_setup.authed_user_cookie().await;

        let client = reqwest::Client::new();

        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            FoodRoutes::Last.addr()
        );

        // Make two request, to make sure the cache is used and works
        client
            .get(&url)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();

        let result = client
            .get(url)
            .header("cookie", authed_cookie)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::OK);
        let result = result.json::<Response>().await.unwrap().response;
        assert!(result["last_id"].is_i64());
        assert!(result["last_id"].as_i64().as_ref().unwrap() > &1000);

        // Check redis cache
        let redis_cache: Option<i64> = test_setup
            .redis
            .lock()
            .await
            .get("cache::last_id")
            .await
            .unwrap();
        assert!(redis_cache.is_some());
        assert_eq!(redis_cache.unwrap(), result["last_id"].as_i64().unwrap());
    }
}
