use redis::{aio::Connection, AsyncCommands, Value};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::{collections::BTreeMap, hash::Hash, sync::Arc};
use time::Date;
use tokio::sync::Mutex;

use crate::{
    api_error::ApiError,
    database::redis::{string_to_struct, RedisKey, HASH_FIELD},
    helpers::genesis_date,
};

use super::{FromModel, Person};

#[derive(sqlx::FromRow, Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ModelFoodCategory {
    pub id: i64,
    #[serde(rename = "c")]
    pub category: String,
    #[serde(rename = "n")]
    pub count: i64,
}

impl ModelFoodCategory {
    async fn insert_cache(
        categories: &[Self],
        redis: &Arc<Mutex<Connection>>,
    ) -> Result<(), ApiError> {
        redis
            .lock()
            .await
            .hset(
                RedisKey::Category.to_string(),
                HASH_FIELD,
                serde_json::to_string(&categories)?,
            )
            .await?;
        Ok(())
    }

    async fn get_cache(redis: &Arc<Mutex<Connection>>) -> Result<Option<Vec<Self>>, ApiError> {
        Ok(
            if let Some(cache) = redis
                .lock()
                .await
                .hget::<'_, String, &str, Option<Value>>(RedisKey::Category.to_string(), HASH_FIELD)
                .await?
            {
                Some(string_to_struct::<Vec<Self>>(&cache)?)
            } else {
                None
            },
        )
    }

    pub async fn delete_cache(redis: &Arc<Mutex<Connection>>) -> Result<(), ApiError> {
        Ok(redis
            .lock()
            .await
            .del(RedisKey::Category.to_string())
            .await?)
    }

    pub async fn get_all(
        postgres: &PgPool,
        redis: &Arc<Mutex<Connection>>,
    ) -> Result<Vec<Self>, ApiError> {
        if let Some(categories) = Self::get_cache(redis).await? {
            Ok(categories)
        } else {
            let query = r#"
SELECT
	im.meal_category_id AS id,
	mc.category AS category,
	count(mc.category) AS count
FROM
	individual_meal im
JOIN
	meal_category mc
ON
	im.meal_category_id = mc.meal_category_id
GROUP BY
	category, id ORDER BY count DESC"#;
            let data = sqlx::query_as::<_, Self>(query).fetch_all(postgres).await?;
            Self::insert_cache(&data, redis).await?;
            Ok(data)
        }
    }
}

// TODO move this to outgoing json?

/// Used to skip serializtion if value is None or false
#[allow(clippy::trivially_copy_pass_by_ref)]
fn none_or_false(x: &Option<bool>) -> bool {
    if let Some(value) = x {
        return !value.to_owned();
    }
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash, Eq)]
struct PersonPhoto {
    #[serde(rename = "o")]
    original: String,
    #[serde(rename = "c")]
    converted: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash, Eq)]
struct PersonFood {
    #[serde(rename = "md")]
    meal_description: String,
    #[serde(rename = "c")]
    category: i64,
    #[serde(rename = "r", skip_serializing_if = "none_or_false")]
    restaurant: Option<bool>,
    #[serde(rename = "v", skip_serializing_if = "none_or_false")]
    vegetarian: Option<bool>,
    #[serde(rename = "t", skip_serializing_if = "none_or_false")]
    takeaway: Option<bool>,
    #[serde(rename = "p", skip_serializing_if = "Option::is_none")]
    photo: Option<PersonPhoto>,
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash, Eq)]
pub struct IndividualFoodJson {
    #[serde(rename = "da")]
    date: String,
    #[serde(rename = "D", skip_serializing_if = "Option::is_none")]
    Dave: Option<PersonFood>,
    #[serde(rename = "J", skip_serializing_if = "Option::is_none")]
    Jack: Option<PersonFood>,
}

impl FromModel<&[ModelIndividualFood]> for IndividualFoodJson {
    type Item = Vec<Self>;

    /// Probably inefficient
    /// Convert to reduced json data to send to client, combines meals of same date, uses BTreeMap to keep order,
    /// much quicker than using a vec - 10ms v 600ms
    fn from_model(data: &[ModelIndividualFood]) -> Result<Vec<Self>, ApiError> {
        let mut output: BTreeMap<String, Self> = BTreeMap::new();
        for row in data {
            let person = Person::try_from(row.person.as_str())?;
            let photo = if let (Some(photo_converted), Some(photo_original)) =
                (row.photo_converted.as_ref(), row.photo_original.as_ref())
            {
                Some(PersonPhoto {
                    original: photo_original.clone(),
                    converted: photo_converted.clone(),
                })
            } else {
                None
            };

            let food = PersonFood {
                meal_description: row.description.clone(),
                category: row.category_id,
                restaurant: row.restaurant,
                vegetarian: row.vegetarian,
                takeaway: row.takeaway,
                photo,
            };

            if let Some(entry) = output.get_mut(&row.meal_date) {
                match person {
                    Person::Dave => entry.Dave = Some(food),
                    Person::Jack => entry.Jack = Some(food),
                }
            } else {
                // Always do it in alphabetical order
                let person_values = match person {
                    Person::Dave => (Some(food), None),
                    Person::Jack => (None, Some(food)),
                };
                let item = Self {
                    date: row.meal_date.clone(),
                    Dave: person_values.0,
                    Jack: person_values.1,
                };
                output.insert(row.meal_date.clone(), item);
            }
        }
        // Convert to a vec, reverse as to do in newest to oldest, postgres query does oldest to newest - could reverse that
        Ok(output.iter().rev().map(|x| x.1.clone()).collect::<Vec<_>>())
    }
}

#[derive(sqlx::FromRow, Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ModelIndividualFood {
    pub meal_date: String,
    pub category_id: i64,
    pub person: String,
    pub restaurant: Option<bool>,
    pub takeaway: Option<bool>,
    pub vegetarian: Option<bool>,
    pub description: String,
    pub photo_original: Option<String>,
    pub photo_converted: Option<String>,
}

impl ModelIndividualFood {
    async fn insert_cache(
        all_meals: &[IndividualFoodJson],
        redis: &Arc<Mutex<Connection>>,
    ) -> Result<(), ApiError> {
        redis
            .lock()
            .await
            .hset(
                RedisKey::AllMeals.to_string(),
                "data",
                serde_json::to_string(&all_meals)?,
            )
            .await?;
        Ok(())
    }

    async fn get_cache(
        redis: &Arc<Mutex<Connection>>,
    ) -> Result<Option<Vec<IndividualFoodJson>>, ApiError> {
        let op_data: Option<Value> = redis
            .lock()
            .await
            .hget(RedisKey::AllMeals.to_string(), "data")
            .await?;

        if let Some(data) = op_data {
            Ok(Some(string_to_struct::<Vec<IndividualFoodJson>>(&data)?))
        } else {
            Ok(None)
        }
    }

    pub async fn delete_cache(redis: &Arc<Mutex<Connection>>) -> Result<(), ApiError> {
        Ok(redis
            .lock()
            .await
            .del(RedisKey::AllMeals.to_string())
            .await?)
    }

    pub async fn get_all(
        postgres: &PgPool,
        redis: &Arc<Mutex<Connection>>,
    ) -> Result<Vec<IndividualFoodJson>, ApiError> {
        if let Some(categories) = Self::get_cache(redis).await? {
            Ok(categories)
        } else {
            let query = r#"
SELECT
	md.date_of_meal::text as meal_date,
	mpe.person as person,
	im.meal_category_id as category_id, im.restaurant as restaurant, im.takeaway as takeaway, im.vegetarian as vegetarian,
	mde.description as description,
	mp.photo_original as photo_original, mp.photo_converted AS photo_converted
FROM
	individual_meal im
JOIN
	meal_date md
ON
	im.meal_date_id = md.meal_date_id
JOIN
	meal_description mde
ON
	im.meal_description_id = mde.meal_description_id
JOIN
	meal_person mpe
ON
	mpe.meal_person_id = im.meal_person_id
LEFT JOIN
	meal_photo mp
ON
	im.meal_photo_id = mp.meal_photo_id
ORDER BY
	meal_date DESC, person"#;
            let data = sqlx::query_as::<_, Self>(query).fetch_all(postgres).await?;
            let reduced_json = IndividualFoodJson::from_model(&data)?;
            Self::insert_cache(&reduced_json, redis).await?;
            Ok(reduced_json)
        }
    }
}

#[derive(sqlx::FromRow, Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct ModelFoodLastId {
    pub last_id: i64,
}

impl ModelFoodLastId {
    fn key() -> String {
        RedisKey::LastID.to_string()
    }

    async fn insert_cache(&self, redis: &Arc<Mutex<Connection>>) -> Result<(), ApiError> {
        redis.lock().await.set(Self::key(), self.last_id).await?;
        Ok(())
    }

    async fn get_cache(redis: &Arc<Mutex<Connection>>) -> Result<Option<i64>, ApiError> {
        Ok(redis.lock().await.get(Self::key()).await?)
    }

    pub async fn delete_cache(redis: &Arc<Mutex<Connection>>) -> Result<(), ApiError> {
        Ok(redis.lock().await.del(Self::key()).await?)
    }

    pub async fn get(postgres: &PgPool, redis: &Arc<Mutex<Connection>>) -> Result<Self, ApiError> {
        if let Some(id) = Self::get_cache(redis).await? {
            Ok(Self { last_id: id })
        } else {
            let query = r#"SELECT individual_meal_audit_id as last_id FROM individual_meal_audit ORDER BY individual_meal_audit_id DESC LIMIT 1"#;
            let last_id = sqlx::query_as::<_, Self>(query).fetch_one(postgres).await?;
            last_id.insert_cache(redis).await?;
            Ok(last_id)
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct MissingFoodJson {
    pub date: String,
    pub person: Person,
}

impl MissingFoodJson {
    fn from_model(data: &[ModelMissingFood]) -> Result<Vec<Self>, ApiError> {
        let mut output = vec![];
        for entry in data {
            output.push(Self {
                date: entry.missing_date.to_string(),
                person: Person::try_from(entry.person.as_str())?,
            });
        }
        Ok(output)
    }
}

#[derive(sqlx::FromRow, Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct ModelMissingFood {
    pub missing_date: Date,
    pub person: String,
}

impl ModelMissingFood {
    // SELECT current_date - INTEGER '1' AS yesterday_date;
    // ( SELECT missing_date::date FROM generate_series($1, now() - interval '1 day', interval '1 day') AS missing_date)
    pub async fn get(postgres: &PgPool) -> Result<Vec<MissingFoodJson>, ApiError> {
        let query = r#"
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
		JOIN
			meal_date md ON md.meal_date_id = im.meal_date_id
		JOIN
			meal_person mp ON mp.meal_person_id = im.meal_person_id
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
		JOIN
			meal_date md ON md.meal_date_id = im.meal_date_id
		JOIN
			meal_person mp ON mp.meal_person_id = im.meal_person_id
		 WHERE
			person = 'Dave'
		)
ORDER BY missing_date DESC, person ASC
"#;
        let data = sqlx::query_as::<_, Self>(query)
            .bind(genesis_date())
            .fetch_all(postgres)
            .await?;
        let as_json = MissingFoodJson::from_model(&data)?;
        Ok(as_json)
    }
}
