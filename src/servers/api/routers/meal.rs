use axum::{
    Router,
    extract::State,
    middleware,
    routing::{delete, get, patch},
};

use crate::{
    C, S,
    api::{ApiRouter, ApiState},
    api_error::ApiError,
    database::{FromModel, MealResponse, ModelMeal, ModelMissingFood, ModelUser},
    define_routes,
    servers::{
        Outgoing,
        authentication::{authenticate_password_token, is_admin},
        ij, oj,
    },
};

define_routes! {
    MealRoutes,
    "/meal",
    Base => "",
    Missing => "/missing",
    ParamDatePerson => "/{date}/{person}"
}

pub struct MealRouter;

impl ApiRouter for MealRouter {
    fn create_router(state: &ApiState) -> axum::Router<ApiState> {
        Router::new()
            .route(&MealRoutes::Missing.addr(), get(Self::missing_get))
            .route(
                &MealRoutes::Base.addr(),
                patch(Self::base_patch).post(Self::base_post),
            )
            .route(
                &MealRoutes::ParamDatePerson.addr(),
                delete(Self::param_date_person_delete).get(Self::param_date_person_get),
            )
            .layer(middleware::from_fn_with_state(C!(state), is_admin))
    }
}

impl MealRouter {
    /// Update meal
    async fn base_patch(
        State(state): State<ApiState>,
        user: ModelUser,
        ij::IncomingJson(body): ij::IncomingJson<ij::MealPatch>,
    ) -> Result<axum::http::StatusCode, ApiError> {
        match ModelMeal::get_by_date_person(&state.postgres, &body.meal.person, body.original_date)
            .await?
        {
            Some(original_meal) => {
                if ij::Meal::from_model(&original_meal)? == body.meal {
                    return Err(ApiError::InvalidValue(S!("no changes")));
                }
                ModelMeal::update(&state.postgres, &body.meal, &user, &original_meal).await?;
                MealResponse::cache_delete(&state.redis).await?;
                Ok(axum::http::StatusCode::OK)
            }
            _ => Err(ApiError::InvalidValue(S!("unknown meal"))),
        }
    }

    /// insert new meal
    async fn base_post(
        State(state): State<ApiState>,
        user: ModelUser,
        ij::IncomingJson(body): ij::IncomingJson<ij::Meal>,
    ) -> Result<axum::http::StatusCode, ApiError> {
        if ModelMeal::get_by_date_person(&state.postgres, &body.person, body.date)
            .await?
            .is_some()
        {
            Err(ApiError::InvalidValue(S!(
                "Meal already exists on date and person given"
            )))
        } else {
            ModelMeal::insert(&state.postgres, &body, &user).await?;
            MealResponse::cache_delete(&state.redis).await?;
            Ok(axum::http::StatusCode::OK)
        }
    }

    /// get list of missing meals
    async fn missing_get(
        State(state): State<ApiState>,
    ) -> Result<Outgoing<Vec<oj::MissingFood>>, ApiError> {
        Ok((
            axum::http::StatusCode::OK,
            oj::OutgoingJson::new(ModelMissingFood::get(&state.postgres).await?),
        ))
    }

    /// Get the information on a single meal, based on date and person
    async fn param_date_person_get(
        State(state): State<ApiState>,
        ij::Path(ij::DatePerson { date, person }): ij::Path<ij::DatePerson>,
    ) -> Result<Outgoing<oj::AdminMeal>, ApiError> {
        Ok((
            axum::http::StatusCode::OK,
            oj::OutgoingJson::new(oj::AdminMeal {
                meal: ModelMeal::get_by_date_person(&state.postgres, &person, date)
                    .await?
                    .map(oj::Meal::from),
            }),
        ))
    }

    /// Delete a single meal, based on date and person, requires password/token
    async fn param_date_person_delete(
        State(state): State<ApiState>,
        user: ModelUser,
        ij::Path(ij::DatePerson { date, person }): ij::Path<ij::DatePerson>,
        ij::IncomingJson(body): ij::IncomingJson<ij::PasswordToken>,
    ) -> Result<axum::http::StatusCode, ApiError> {
        if !authenticate_password_token(&user, &body.password, body.token, &state.postgres).await? {
            return Err(ApiError::Authorization);
        }
        ModelMeal::delete(&state.postgres, &person, date).await?;
        MealResponse::cache_delete(&state.redis).await?;
        Ok(axum::http::StatusCode::OK)
    }
}

// Use reqwest to test against real server
// cargo watch -q -c -w src/ -x 'test api_router_meal -- --test-threads=1 --nocapture'
#[cfg(test)]
#[expect(clippy::pedantic, clippy::unwrap_used)]
mod tests {

    use std::collections::HashMap;

    use super::MealRoutes;
    use crate::{
        C,
        helpers::gen_random_hex,
        servers::api_tests::{
            Response, TEST_PASSWORD, TestBodyMealPatch, base_url, start_both_servers,
        },
    };

    use fred::interfaces::KeysInterface;
    use reqwest::StatusCode;

    #[tokio::test]
    /// Unauthenticated user unable to [PATCH, POST] "/" route
    async fn api_router_meal_base_unauthenticated() {
        let test_setup = start_both_servers().await;
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            MealRoutes::Base.addr()
        );
        let client = reqwest::Client::new();

        let result = client.patch(&url).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");

        let result = client.post(&url).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");
    }

    #[tokio::test]
    /// Authenticated, but not admin user, user unable to [PATCH, POST] "/" route
    async fn api_router_meal_base_not_admin() {
        let mut test_setup = start_both_servers().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            MealRoutes::Base.addr()
        );
        let client = reqwest::Client::new();

        let result = client
            .patch(&url)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");

        let result = client
            .post(&url)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");
    }

    #[tokio::test]
    /// Authenticated admin user able to add new meal
    async fn api_router_meal_base_admin_valid() {
        let mut test_setup = start_both_servers().await;

        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.make_user_admin().await;

        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            MealRoutes::Base.addr()
        );
        let client = reqwest::Client::new();

        let body = test_setup.gen_meal(false);
        let result = client
            .post(&url)
            .header("cookie", &authed_cookie)
            .json(&body)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::OK);

        let meal = test_setup.query_meal().await;
        assert!(meal.is_some());
        let meal = meal.unwrap();
        let test_meal = || test_setup.test_meal.as_ref().unwrap();

        assert_eq!(meal.category, test_meal().category);
        assert_eq!(meal.description, test_meal().description);
        assert_eq!(meal.person, test_meal().person);
        assert_eq!(meal.category, test_meal().category);
        assert_eq!(meal.takeaway, test_meal().takeaway);
        assert_eq!(meal.vegetarian, test_meal().vegetarian);
        assert_eq!(meal.restaurant, test_meal().restaurant);
        assert!(meal.photo_converted.is_none());
        assert!(meal.photo_original.is_none());
    }

    #[tokio::test]
    /// Authenticated admin user able to add new meal, and the category and description is trimmed
    async fn api_router_meal_base_admin_valid_trimmed() {
        let mut test_setup = start_both_servers().await;

        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.make_user_admin().await;

        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            MealRoutes::Base.addr()
        );
        let client = reqwest::Client::new();

        let mut body = test_setup.gen_meal(false);
        body.description.push(' ');
        body.category.push('\n');

        let result = client
            .post(&url)
            .header("cookie", &authed_cookie)
            .json(&body)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::OK);

        let meal = test_setup.query_meal().await;
        assert!(meal.is_some());
        let meal = meal.unwrap();
        let test_meal = || test_setup.test_meal.as_ref().unwrap();

        assert_eq!(meal.category, test_meal().category);
        assert_eq!(meal.description, test_meal().description);
        assert_eq!(meal.person, test_meal().person);
        assert_eq!(meal.category, test_meal().category);
        assert_eq!(meal.takeaway, test_meal().takeaway);
        assert_eq!(meal.vegetarian, test_meal().vegetarian);
        assert_eq!(meal.restaurant, test_meal().restaurant);
        assert!(meal.photo_converted.is_none());
        assert!(meal.photo_original.is_none());
    }

    #[tokio::test]
    /// Authenticated admin user able to add new meal - with photo
    async fn api_router_meal_base_admin_valid_with_photo() {
        let mut test_setup = start_both_servers().await;

        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.make_user_admin().await;

        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            MealRoutes::Base.addr()
        );
        let client = reqwest::Client::new();

        let body = test_setup.gen_meal(true);
        let result = client
            .post(&url)
            .header("cookie", &authed_cookie)
            .json(&body)
            .send()
            .await
            .unwrap();

        assert_eq!(result.status(), StatusCode::OK);

        let meal = test_setup.query_meal().await;
        assert!(meal.is_some());
        let meal = meal.unwrap();
        let test_meal = || test_setup.test_meal.as_ref().unwrap();

        assert_eq!(meal.category, test_meal().category);
        assert_eq!(meal.description, test_meal().description);
        assert_eq!(meal.person, test_meal().person);
        assert_eq!(meal.category, test_meal().category);
        assert_eq!(meal.takeaway, test_meal().takeaway);
        assert_eq!(meal.vegetarian, test_meal().vegetarian);
        assert_eq!(meal.restaurant, test_meal().restaurant);
        assert!(meal.photo_converted.is_some());
        assert!(meal.photo_original.is_some());
    }

    #[tokio::test]
    /// Authenticated admin unable to post a meal on date that meal already exists
    async fn api_router_meal_base_admin_invalid_mealdate_already_exists() {
        let mut test_setup = start_both_servers().await;

        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.make_user_admin().await;

        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            MealRoutes::Base.addr()
        );
        let client = reqwest::Client::new();

        let body = test_setup.gen_meal(true);
        client
            .post(&url)
            .header("cookie", &authed_cookie)
            .json(&body)
            .send()
            .await
            .unwrap();
        let result = client
            .post(&url)
            .header("cookie", &authed_cookie)
            .json(&body)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::BAD_REQUEST);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Meal already exists on date and person given");
    }

    #[tokio::test]
    /// Authenticated admin unable to patch is new meal matches previous meal
    async fn api_router_meal_base_patch_no_changes() {
        let mut test_setup = start_both_servers().await;

        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.make_user_admin().await;

        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            MealRoutes::Base.addr()
        );

        let client = reqwest::Client::new();

        let body = test_setup.gen_meal(false);
        client
            .post(&url)
            .header("cookie", &authed_cookie)
            .json(&body)
            .send()
            .await
            .unwrap();

        let body = TestBodyMealPatch {
            original_date: C!(body.date),
            meal: body,
        };

        let result = client
            .patch(&url)
            .header("cookie", &authed_cookie)
            .json(&body)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::BAD_REQUEST);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "no changes");
    }

    #[tokio::test]
    /// Authenticated admin unable to patch is new meal matches previous meal, old categories & description no longer in database, redis cache emptied
    async fn api_router_meal_base_patch_valid_patch() {
        let mut test_setup = start_both_servers().await;

        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.make_user_admin().await;
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            MealRoutes::Base.addr()
        );

        let client = reqwest::Client::new();

        let body = test_setup.gen_meal(true);
        client
            .post(&url)
            .header("cookie", &authed_cookie)
            .json(&body)
            .send()
            .await
            .unwrap();

        let new_category = gen_random_hex(8);
        let new_description = gen_random_hex(8);

        let mut new_meal = C!(body);
        new_meal.description.clone_from(&new_description);
        new_meal.category.clone_from(&new_category);
        new_meal.vegetarian = !body.vegetarian;
        new_meal.takeaway = !body.takeaway;
        new_meal.restaurant = !body.restaurant;
        new_meal.photo_converted = None;
        new_meal.photo_original = None;

        let new_body = TestBodyMealPatch {
            original_date: C!(body.date),
            meal: new_meal,
        };

        let result = client
            .patch(&url)
            .header("cookie", &authed_cookie)
            .json(&new_body)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::OK);

        let result = sqlx::query!(
            "SELECT * FROM meal_description WHERE description = $1",
            body.description
        )
        .fetch_optional(&test_setup.postgres)
        .await
        .unwrap();
        assert!(result.is_none());

        let result = sqlx::query!(
            "SELECT * FROM meal_category WHERE category = $1",
            body.category
        )
        .fetch_optional(&test_setup.postgres)
        .await
        .unwrap();
        assert!(result.is_none());

        for i in ["cache::all_meals", "cache::last_id", "cache::category"] {
            let redis_cache: Option<String> = test_setup.redis.get(i).await.unwrap();
            assert!(redis_cache.is_none());
        }

        let url = format!("{}/meal/{}/Jack", base_url(&test_setup.app_env), body.date);
        let result = client
            .get(&url)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();

        let result = result.json::<Response>().await.unwrap().response;
        let result = result["meal"].as_object().unwrap();

        assert_eq!(
            result.get("category").unwrap().as_str().unwrap(),
            new_body.meal.category
        );
        assert_eq!(
            result.get("description").unwrap().as_str().unwrap(),
            new_body.meal.description
        );

        assert_eq!(
            result.get("vegetarian").unwrap().as_bool().unwrap(),
            new_body.meal.vegetarian
        );
        assert_eq!(
            result.get("restaurant").unwrap().as_bool().unwrap(),
            new_body.meal.restaurant
        );
        assert_eq!(
            result.get("takeaway").unwrap().as_bool().unwrap(),
            new_body.meal.takeaway
        );

        assert!(result.get("photo_original").unwrap().is_null());
        assert!(result.get("photo_original").unwrap().is_null());
    }

    //////////////////////////////////////////////////

    #[tokio::test]
    /// Unauthenticated user unable to access "/missing" route
    async fn api_router_meal_missing_unauthenticated() {
        let test_setup = start_both_servers().await;
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            MealRoutes::Base.addr()
        );
        let client = reqwest::Client::new();

        let result = client.get(&url).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");
    }

    #[tokio::test]
    /// Authenticated, but not admin user, unable to access "/missing" route
    async fn api_router_meal_missing_not_admin() {
        let mut test_setup = start_both_servers().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            MealRoutes::Missing.addr()
        );
        let client = reqwest::Client::new();

        let result = client
            .get(&url)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");
    }

    #[tokio::test]
    /// Get list of missing meals - assumes that the db_data isn't up to date!
    async fn api_router_meal_missing_admin_valid() {
        let mut test_setup = start_both_servers().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.make_user_admin().await;
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            MealRoutes::Missing.addr()
        );
        let client = reqwest::Client::new();

        let result = client
            .get(&url)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::OK);

        let result = result.json::<Response>().await.unwrap().response;
        assert!(result.is_array());

        assert!(result.as_array().unwrap().len() > 1);
        assert!(result.as_array().unwrap()[0]["date"].is_string());
        assert!(result.as_array().unwrap()[0]["person"].is_string());
    }

    #[tokio::test]
    /// Unauthenticated user unable to [GET, DELETE] access "/missing" route
    async fn api_router_meal_date_person_unauthenticated() {
        let test_setup = start_both_servers().await;
        let url = format!("{}/meal/2020-01-01/Jack", base_url(&test_setup.app_env),);

        let client = reqwest::Client::new();

        let result = client.get(&url).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");

        let result = client.delete(&url).send().await.unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");
    }

    #[tokio::test]
    /// Authenticated, but not admin user, unable to [GET, DELETE] "/missing" route
    async fn api_router_meal_date_person_not_admin() {
        let mut test_setup = start_both_servers().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        let url = format!("{}/meal/2020-01-01/Jack", base_url(&test_setup.app_env),);
        let client = reqwest::Client::new();

        let result = client
            .get(&url)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");

        let result = client
            .delete(&url)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN);
        let result = result.json::<Response>().await.unwrap().response;
        assert_eq!(result, "Invalid Authentication");
    }

    #[tokio::test]
    /// params invalid, return 400 error
    async fn api_router_meal_date_person_invalid_params() {
        let mut test_setup = start_both_servers().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.make_user_admin().await;

        async fn person_test(base_url: &str, cookie: &str, person: &str) {
            let client = reqwest::Client::new();
            let url = format!("{base_url}/meal/2020-01-01/{person}");

            let result = client
                .get(&url)
                .header("cookie", cookie)
                .send()
                .await
                .unwrap();
            assert_eq!(result.status(), StatusCode::BAD_REQUEST);
            let result = result.json::<Response>().await.unwrap().response;
            assert_eq!(result, "invalid person param");
        }

        let base_url = base_url(&test_setup.app_env);

        person_test(&base_url, &authed_cookie, "jack").await;
        person_test(&base_url, &authed_cookie, "dave").await;
        person_test(&base_url, &authed_cookie, "JACK").await;
        person_test(&base_url, &authed_cookie, "DAVE").await;
        person_test(&base_url, &authed_cookie, "2020-01-01").await;
        person_test(&base_url, &authed_cookie, &gen_random_hex(4)).await;

        async fn date_test(base_url: &str, cookie: &str, date: &str) {
            let client = reqwest::Client::new();
            let url = format!("{base_url}/meal/{date}/Jack");

            let result = client
                .get(&url)
                .header("cookie", cookie)
                .send()
                .await
                .unwrap();
            assert_eq!(result.status(), StatusCode::BAD_REQUEST);
            let result = result.json::<Response>().await.unwrap().response;
            assert_eq!(result, "invalid date param");
        }

        date_test(&base_url, &authed_cookie, "2020-14-01").await;
        date_test(&base_url, &authed_cookie, "2020-01-40").await;
        date_test(&base_url, &authed_cookie, "2013-05-05").await;
        date_test(&base_url, &authed_cookie, "2018-*5-05").await;
        date_test(&base_url, &authed_cookie, &gen_random_hex(10)).await;
    }

    #[tokio::test]
    /// Valid params, return known meal object
    async fn api_router_meal_date_person_get_valid() {
        let mut test_setup = start_both_servers().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.make_user_admin().await;
        let client = reqwest::Client::new();
        let url = format!("{}/meal/2020-01-01/Jack", base_url(&test_setup.app_env));
        let result = client
            .get(&url)
            .header("cookie", authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::OK);
        let result = result.json::<Response>().await.unwrap().response;

        assert!(result["meal"].is_object());
        let result = result["meal"].as_object().unwrap();

        assert_eq!(result.get("date").unwrap(), "2020-01-01");
        assert_eq!(result.get("category").unwrap(), "DUCK");
        assert_eq!(result.get("person").unwrap(), "Jack");
        assert_eq!(result.get("restaurant").unwrap(), false);
        assert_eq!(result.get("takeaway").unwrap(), false);
        assert_eq!(result.get("vegetarian").unwrap(), false);
        assert_eq!(
            result.get("description").unwrap(),
            "Peking duck, pancakes, cabbage,"
        );
        assert_eq!(
            result.get("photo_original").unwrap(),
            "01dxh6kawgcs6wbfxppqtryn7p10.jpg"
        );
        assert_eq!(
            result.get("photo_converted").unwrap(),
            "01dxh6kawgpetaws6t9g4946z911.jpg"
        );
    }

    #[tokio::test]
    /// Valid params, valiud body, delete meal
    async fn api_router_meal_date_person_delete_valid() {
        let mut test_setup = start_both_servers().await;
        let authed_cookie = test_setup.authed_user_cookie().await;
        test_setup.make_user_admin().await;
        let client = reqwest::Client::new();
        let url = format!(
            "{}{}",
            base_url(&test_setup.app_env),
            MealRoutes::Base.addr()
        );
        let body = test_setup.gen_meal(false);
        client
            .post(&url)
            .header("cookie", &authed_cookie)
            .json(&body)
            .send()
            .await
            .unwrap();

        let url = format!("{}/meal/{}/Jack", base_url(&test_setup.app_env), body.date);

        let body = HashMap::from([("password", TEST_PASSWORD)]);
        let result = client
            .delete(&url)
            .json(&body)
            .header("cookie", &authed_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::OK);

        let meal = test_setup.query_meal().await;
        assert!(meal.is_none());
    }
}
