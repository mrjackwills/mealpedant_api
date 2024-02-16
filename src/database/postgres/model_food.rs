use fred::{
    clients::RedisPool,
    interfaces::{HashesInterface, KeysInterface},
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::{collections::BTreeMap, hash::Hash};
use time::Date;

use crate::{
    api_error::ApiError,
    database::redis::{RedisKey, HASH_FIELD},
    helpers::genesis_date,
    hmap, redis_hash_to_struct,
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
    fn key() -> String {
        RedisKey::Category.to_string()
    }

    async fn insert_cache(categories: &[Self], redis: &RedisPool) -> Result<(), ApiError> {
        Ok(redis
            .hset(Self::key(), hmap!(serde_json::to_string(&categories)?))
            .await?)
    }

    async fn get_cache(redis: &RedisPool) -> Result<Option<Vec<Self>>, ApiError> {
        match redis
            .hget::<Option<String>, String, &str>(Self::key(), HASH_FIELD)
            .await?
        {
            Some(r) => Ok(Some(serde_json::from_str(&r)?)),
            None => Ok(None),
        }
    }

    pub async fn delete_cache(redis: &RedisPool) -> Result<(), ApiError> {
        Ok(redis.del(Self::key()).await?)
    }

    pub async fn get_all(postgres: &PgPool, redis: &RedisPool) -> Result<Vec<Self>, ApiError> {
        if let Some(categories) = Self::get_cache(redis).await? {
            Ok(categories)
        } else {
            let query = "
SELECT
	im.meal_category_id AS id,
	mc.category AS category,
	count(mc.category) AS count
FROM
	individual_meal im
	JOIN meal_category mc USING(meal_category_id)
GROUP BY
	category,
	id
ORDER BY
	count DESC";
            let data = sqlx::query_as::<_, Self>(query).fetch_all(postgres).await?;
            Self::insert_cache(&data, redis).await?;
            Ok(data)
        }
    }
}

/// Used to skip serializtion if value is None or false
#[allow(clippy::trivially_copy_pass_by_ref)]
fn none_or_false(x: &Option<bool>) -> bool {
    if let Some(value) = x {
        return !value;
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
            let photo = if let (Some(converted), Some(original)) =
                (row.photo_converted.as_ref(), row.photo_original.as_ref())
            {
                Some(PersonPhoto {
                    original: original.clone(),
                    converted: converted.clone(),
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
        Ok(output.into_iter().rev().map(|x| x.1).collect::<Vec<_>>())
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
    fn key() -> String {
        RedisKey::AllMeals.to_string()
    }

    async fn insert_cache(
        all_meals: &[IndividualFoodJson],
        redis: &RedisPool,
    ) -> Result<(), ApiError> {
        Ok(redis
            .hset(Self::key(), hmap!(serde_json::to_string(&all_meals)?))
            .await?)
    }

    async fn get_cache(redis: &RedisPool) -> Result<Option<Vec<IndividualFoodJson>>, ApiError> {
        match redis
            .hget::<Option<String>, String, &str>(Self::key(), HASH_FIELD)
            .await?
        {
            Some(r) => Ok(Some(serde_json::from_str(&r)?)),
            None => Ok(None),
        }
    }

    pub async fn delete_cache(redis: &RedisPool) -> Result<(), ApiError> {
        Ok(redis.del(Self::key()).await?)
    }

    pub async fn get_all(
        postgres: &PgPool,
        redis: &RedisPool,
    ) -> Result<Vec<IndividualFoodJson>, ApiError> {
        if let Some(categories) = Self::get_cache(redis).await? {
            Ok(categories)
        } else {
            let query = "
SELECT
	md.date_of_meal::text as meal_date,
	mpe.person as person,
	im.meal_category_id as category_id, im.restaurant as restaurant, im.takeaway as takeaway, im.vegetarian as vegetarian,
	mde.description as description,
	mp.photo_original as photo_original, mp.photo_converted AS photo_converted
FROM
	individual_meal im
LEFT JOIN meal_date md USING(meal_date_id)
LEFT JOIN meal_description mde USING(meal_description_id)
LEFT JOIN meal_person mpe USING(meal_person_id)
LEFT JOIN meal_photo mp USING(meal_photo_id)
ORDER BY
	meal_date DESC, person";
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

redis_hash_to_struct!(ModelFoodLastId);

impl ModelFoodLastId {
    fn key() -> String {
        RedisKey::LastID.to_string()
    }

    async fn insert_cache(&self, redis: &RedisPool) -> Result<(), ApiError> {
        Ok(redis.hset(Self::key(), hmap!(self.last_id)).await?)
    }

    async fn get_cache(redis: &RedisPool) -> Result<Option<i64>, ApiError> {
        Ok(redis.hget(Self::key(), HASH_FIELD).await?)
    }

    pub async fn delete_cache(redis: &RedisPool) -> Result<(), ApiError> {
        Ok(redis.del(Self::key()).await?)
    }

    pub async fn get(postgres: &PgPool, redis: &RedisPool) -> Result<Self, ApiError> {
        if let Some(id) = Self::get_cache(redis).await? {
            Ok(Self { last_id: id })
        } else {
            let query = "SELECT individual_meal_audit_id as last_id FROM individual_meal_audit ORDER BY individual_meal_audit_id DESC LIMIT 1";
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
    pub async fn get(postgres: &PgPool) -> Result<Vec<MissingFoodJson>, ApiError> {
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
ORDER BY missing_date DESC, person ASC
";
        let data = sqlx::query_as::<_, Self>(query)
            .bind(genesis_date())
            .fetch_all(postgres)
            .await?;
        let as_json = MissingFoodJson::from_model(&data)?;
        Ok(as_json)
    }
}
