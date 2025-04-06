use std::collections::HashMap;

use blake3::Hash;
use fred::{
    clients::Pool,
    interfaces::{HashesInterface, KeysInterface},
};
use jiff_sqlx::ToSqlx;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::{
    S,
    api_error::ApiError,
    database::redis::{HASH_FIELD, RedisKey},
    helpers::genesis_date,
    hmap,
    servers::oj::{DateMeal, MealInfo, MissingFood, none_or_zero},
};

#[derive(sqlx::FromRow, Debug, Clone, Serialize, Deserialize, PartialEq, Hash, Eq)]
pub struct MealDescription {
    #[serde(rename = "i")]
    meal_description_id: i64,
    #[serde(rename = "d")]
    description: String,
}

impl MealDescription {
    /// Get all the meal descriptions as a hashmap, with the id as a key
    /// If both is Some(()), search for meal descriptions from both Jack and Dave, else just Jack
    pub async fn get(
        postgres: &PgPool,
        both: Option<()>,
    ) -> Result<HashMap<i64, String>, ApiError> {
        let query = match both {
            Some(()) => {
                sqlx::query_as!(
                    Self,
                    r#"
SELECT DISTINCT
    md.meal_description_id,
    md.description AS "description!"
FROM
    meal_description md
JOIN
    individual_meal im USING(meal_description_id)
JOIN
    meal_person mpe USING(meal_person_id)
ORDER BY
    md.meal_description_id DESC"#
                )
                .fetch_all(postgres)
                .await
            }
            None => {
                sqlx::query_as!(
                    Self,
                    r#"
SELECT DISTINCT
    md.meal_description_id,
    md.description AS "description!"
FROM
    meal_description md
JOIN
    individual_meal im USING(meal_description_id)
JOIN
    meal_person mpe USING(meal_person_id)
WHERE
    mpe.person = 'Jack'
ORDER BY
    md.meal_description_id DESC"#
                )
                .fetch_all(postgres)
                .await
            }
        };

        Ok(query?
            .into_iter()
            .map(|i| (i.meal_description_id, i.description))
            .collect::<HashMap<i64, String>>())
    }
}

#[derive(sqlx::FromRow, Debug, Clone, Serialize, Deserialize, PartialEq, Hash, Eq)]
pub struct MealCategory {
    #[serde(rename = "i")]
    category_id: i64,
    #[serde(rename = "d")]
    category: String,
}

impl MealCategory {
    /// Get all the meal categories as a hashmap, with the id as a key, and (name,count) as value
    /// If both is Some(()), search for categories from both Jack and Dave, else just Jack
    pub async fn get(
        postgres: &PgPool,
        both: Option<()>,
    ) -> Result<HashMap<i64, String>, ApiError> {
        let query = match both {
            Some(()) => {
                sqlx::query_as!(
                    Self,
                    r#"
SELECT DISTINCT
    im.meal_category_id AS category_id,
    mc.category AS category
FROM
    individual_meal im
JOIN
    meal_category mc USING(meal_category_id)
JOIN
    meal_person mpe USING(meal_person_id)
ORDER BY
    category DESC"#
                )
                .fetch_all(postgres)
                .await?
            }
            None => {
                sqlx::query_as!(
                    Self,
                    r#"
SELECT DISTINCT
    im.meal_category_id AS category_id,
    mc.category AS category
FROM
    individual_meal im
JOIN
    meal_category mc USING(meal_category_id)
JOIN
    meal_person mpe USING(meal_person_id)
WHERE
    mpe.person = 'Jack'
ORDER BY
    category DESC"#
                )
                .fetch_all(postgres)
                .await?
            }
        };
        Ok(query
            .into_iter()
            .map(|i| (i.category_id, i.category))
            .collect::<HashMap<_, _>>())
    }
}

#[derive(sqlx::FromRow, Debug, Clone, Serialize, Deserialize, PartialEq, Hash, Eq)]
pub struct ModelDateMeal {
    #[serde(rename = "d")]
    pub date_of_meal: String,
    #[serde(rename = "c")]
    pub meal_category_id: i64,
    #[serde(rename = "p")]
    pub person: String,
    #[serde(rename = "r", skip_serializing_if = "none_or_zero")]
    pub restaurant: Option<i32>,
    #[serde(rename = "t", skip_serializing_if = "none_or_zero")]
    pub takeaway: Option<i32>,
    #[serde(rename = "v", skip_serializing_if = "none_or_zero")]
    pub vegetarian: Option<i32>,
    #[serde(rename = "e")]
    pub meal_description_id: i64,
    #[serde(rename = "o", skip_serializing_if = "Option::is_none")]
    pub photo_original: Option<String>,
    #[serde(rename = "n", skip_serializing_if = "Option::is_none")]
    pub photo_converted: Option<String>,
}

impl ModelDateMeal {
    /// Get all date meals
    /// if both is Some(()), search for meals from both Jack and Dave, else just Jack
    /// the "x?" is a temporary fix due to a bug in the the sqlx query_as! macrock
    pub async fn get_all(postgres: &PgPool, both: Option<()>) -> Result<Vec<Self>, ApiError> {
        match both {
            Some(()) => Ok(sqlx::query_as!(
                Self,
                r#"
SELECT
    md.date_of_meal::text AS "date_of_meal!",
    im.meal_category_id,
    mpe.person as person,
    im.restaurant::INT,
    im.takeaway::INT,
    im.vegetarian::INT,
    mde.meal_description_id,
    mp.photo_converted AS "photo_converted?",
    mp.photo_original AS "photo_original?"
FROM
    individual_meal im
JOIN
    meal_date md USING(meal_date_id)
JOIN
    meal_description mde USING(meal_description_id)
JOIN
    meal_person mpe USING(meal_person_id)
LEFT JOIN
    meal_photo mp USING(meal_photo_id)
ORDER BY
    date_of_meal DESC,
    person"#
            )
            .fetch_all(postgres)
            .await?),
            None => Ok(sqlx::query_as!(
                Self,
                r#"
SELECT
    md.date_of_meal::text AS "date_of_meal!",
    im.meal_category_id,
    mpe.person as person,
    im.restaurant::INT,
    im.takeaway::INT,
    im.vegetarian::INT,
    mde.meal_description_id,
    mp.photo_converted AS "photo_converted?",
    NULL AS "photo_original?"
FROM
    individual_meal im
JOIN
    meal_date md USING(meal_date_id)
JOIN
    meal_description mde USING(meal_description_id)
JOIN
    meal_person mpe USING(meal_person_id)
LEFT JOIN
    meal_photo mp USING(meal_photo_id)
WHERE
    person = 'Jack'
ORDER BY
    date_of_meal DESC"#
            )
            .fetch_all(postgres)
            .await?),
        }
    }
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MealResponse {
    #[serde(rename = "d")]
    meal_descriptions: HashMap<i64, String>,
    #[serde(rename = "c")]
    meal_categories: HashMap<i64, (String, i64)>,
    #[serde(rename = "m")]
    meals: Vec<ModelDateMeal>,
}

impl MealResponse {
    /// Get the redis key for the meals data
    fn key(both: Option<()>) -> String {
        match both {
            Some(()) => RedisKey::AllMeals.to_string(),
            None => RedisKey::JackMeals.to_string(),
        }
    }
    /// Get the redis key for the meals_hash
    fn key_hash(both: Option<()>) -> String {
        match both {
            Some(()) => RedisKey::AllMealsHash.to_string(),
            None => RedisKey::JackMealsHash.to_string(),
        }
    }
    /// Delete the cache of the meals and the meals_hash
    /// This deletes all caches for all_meals, jack_all_meals, and the hash associated with each
    pub async fn cache_delete(redis: &Pool) -> Result<(), ApiError> {
        Ok(redis
            .del((
                Self::key(Some(())),
                Self::key_hash(Some(())),
                Self::key(None),
                Self::key_hash(None),
            ))
            .await?)
    }

    /// Check redis for meal cache, and return if present
    async fn cache_get(redis: &Pool, both: Option<()>) -> Result<Option<MealInfo>, ApiError> {
        match redis
            .hget::<Option<String>, String, &str>(Self::key(both), HASH_FIELD)
            .await?
        {
            Some(r) => Ok(Some(serde_json::from_str(&r)?)),
            None => Ok(None),
        }
    }

    /// Insert meals cache
    async fn cache_insert(
        redis: &Pool,
        meals: &MealInfo,
        both: Option<()>,
    ) -> Result<(), ApiError> {
        Ok(redis
            .hset(Self::key(both), hmap!(serde_json::to_string(&meals)?))
            .await?)
    }

    /// Generate a hash for the meals.date_meals, the other entries are unordered hashmaps, whereas date_meals is ordered
    fn hash_generate(meals_descriptions: &MealInfo) -> Result<Hash, ApiError> {
        serde_json::to_string(&meals_descriptions.date_meals).map_or_else(
            |_| Err(ApiError::Internal(S!("Hash error"))),
            |as_str| {
                let mut hasher = blake3::Hasher::new();
                hasher.update(as_str.as_bytes());
                Ok(hasher.finalize())
            },
        )
    }

    /// Insert the meals.date_meals hash into redis
    async fn hash_insert(
        redis: &Pool,
        meals: &MealInfo,
        both: Option<()>,
    ) -> Result<String, ApiError> {
        let hash = Self::hash_generate(meals)?.to_string();
        redis
            .set::<(), String, &str>(Self::key_hash(both), &hash, None, None, false)
            .await?;
        Ok(hash)
    }

    /// Return the meals.date_meals hash if present
    pub async fn get_hash(
        postgres: &PgPool,
        redis: &Pool,
        both: Option<()>,
    ) -> Result<String, ApiError> {
        if let Some(x) = redis.get(Self::key_hash(both)).await? {
            Ok(x)
        } else if let Some(cache) = Self::cache_get(redis, both).await? {
            Self::hash_insert(redis, &cache, both).await
        } else {
            let data = Self::get_all(postgres, redis, both).await?;
            Self::hash_insert(redis, &data, both).await
        }
    }

    /// Return all the meals, will check cache first, if no cache, then inserts into cache
    pub async fn get_all(
        postgres: &PgPool,
        redis: &Pool,
        both: Option<()>,
    ) -> Result<MealInfo, ApiError> {
        if let Some(cache) = Self::cache_get(redis, both).await? {
            Ok(cache)
        } else {
            let mut date_meals: Vec<DateMeal> = vec![];

            for i in ModelDateMeal::get_all(postgres, both)
                .await?
                .into_iter()
                .map(DateMeal::from)
            {
                if let Some(given) = date_meals.iter_mut().find(|x| x.date == i.date) {
                    if let Some(j) = i.Jack {
                        given.Jack = Some(j);
                    }
                    if let Some(d) = i.Dave {
                        given.Dave = Some(d);
                    }
                } else {
                    date_meals.push(i);
                }
            }
            let meal_descriptions = MealInfo {
                meal_descriptions: MealDescription::get(postgres, both).await?,
                meal_categories: MealCategory::get(postgres, both).await?,
                date_meals,
            };

            Self::cache_insert(redis, &meal_descriptions, both).await?;
            Self::hash_insert(redis, &meal_descriptions, both).await?;
            Ok(meal_descriptions)
        }
    }
}

#[derive(sqlx::FromRow, Debug, Clone, PartialEq, Eq)]
pub struct ModelMissingFood {
    pub missing_date: jiff_sqlx::Date,
    pub person: String,
}

impl ModelMissingFood {
    /// sqlx/jiff_sqlx issue with this when using query_as!()
    pub async fn get(postgres: &PgPool) -> Result<Vec<MissingFood>, ApiError> {
        let query = "
WITH
    all_dates
AS
    ( SELECT missing_date::date FROM generate_series($1, current_date - INTEGER '1', interval '1 day') AS missing_date)
SELECT 
    * , 'Jack' as person
FROM
    all_dates
WHERE
    missing_date
NOT IN
    (
        SELECT
            date_of_meal
        FROM
            individual_meal im
        JOIN meal_date md USING(meal_date_id)
        JOIN meal_person mp USING(meal_person_id)
        WHERE
            person = 'Jack'
    )
UNION ALL
SELECT 
    * , 'Dave' as person
FROM
    all_dates
WHERE
    missing_date
NOT IN 
    (
        SELECT
            date_of_meal
        FROM
            individual_meal im
        JOIN meal_date md USING(meal_date_id)
        JOIN meal_person mp USING(meal_person_id)
         WHERE
            person = 'Dave'
        )
ORDER BY
    missing_date DESC, person ASC
";
        let data = sqlx::query_as::<_, Self>(query)
            .bind(genesis_date().to_sqlx())
            .fetch_all(postgres)
            .await?;
        let as_json = MissingFood::from_model(&data)?;
        Ok(as_json)
    }
}
