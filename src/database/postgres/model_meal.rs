use std::sync::Arc;
use redis::aio::Connection;
use sqlx::{PgPool, Postgres, Transaction};
use time::Date;
use tokio::sync::Mutex;

use crate::{
    api::ij,
    api_error::ApiError,
    database::{ModelFoodCategory, ModelFoodLastId, ModelIndividualFood},
};

use super::{ModelUser, Person};

#[derive(sqlx::FromRow)]
struct Id {
    id: i64,
}

#[derive(sqlx::FromRow, Debug, Clone, PartialEq, Eq)]
pub struct ModelMeal {
    pub meal_category_id: i64,
    pub meal_description_id: i64,
    pub meal_date_id: i64,
    pub meal_photo_id: Option<i64>,
    pub individual_meal_id: i64,
    pub meal_date: Date,
    pub category: String,
    pub person: String,
    pub restaurant: bool,
    pub takeaway: bool,
    pub vegetarian: bool,
    pub description: String,
    pub photo_original: Option<String>,
    pub photo_converted: Option<String>,
}

impl ModelMeal {
    async fn insert_date(
        transaction: &mut Transaction<'_, Postgres>,
        meal: &ij::Meal,
        user: &ModelUser,
    ) -> Result<i64, ApiError> {
        let query = "SELECT meal_date_id AS id FROM meal_date WHERE date_of_meal = $1";
        if let Some(id) = sqlx::query_as::<_, Id>(query)
            .bind(meal.date)
            .bind(user.registered_user_id)
            .fetch_optional(&mut *transaction)
            .await?
        {
            Ok(id.id)
        } else {
            let query = "INSERT INTO meal_date(date_of_meal, registered_user_id) VALUES($1, $2) RETURNING meal_date_id AS id";
            Ok(sqlx::query_as::<_, Id>(query)
                .bind(meal.date)
                .bind(user.registered_user_id)
                .fetch_one(transaction)
                .await?
                .id)
        }
    }

    async fn insert_category(
        transaction: &mut Transaction<'_, Postgres>,
        meal: &ij::Meal,
        user: &ModelUser,
    ) -> Result<i64, ApiError> {
        let query = "SELECT meal_category_id AS id FROM meal_category WHERE category = upper($1)";
        if let Some(id) = sqlx::query_as::<_, Id>(query)
            .bind(&meal.category)
            .bind(user.registered_user_id)
            .fetch_optional(&mut *transaction)
            .await?
        {
            Ok(id.id)
        } else {
            let query = "INSERT INTO meal_category(category, registered_user_id) VALUES(upper($1), $2) RETURNING meal_category_id AS id";
            Ok(sqlx::query_as::<_, Id>(query)
                .bind(&meal.category)
                .bind(user.registered_user_id)
                .fetch_one(transaction)
                .await?
                .id)
        }
    }

    async fn insert_description(
        transaction: &mut Transaction<'_, Postgres>,
        meal: &ij::Meal,
        user: &ModelUser,
    ) -> Result<i64, ApiError> {
        let query = "SELECT meal_description_id AS id FROM meal_description WHERE description = $1";
        if let Some(id) = sqlx::query_as::<_, Id>(query)
            .bind(&meal.description)
            .bind(user.registered_user_id)
            .fetch_optional(&mut *transaction)
            .await?
        {
            Ok(id.id)
        } else {
            let query = "INSERT INTO meal_description(description, registered_user_id) VALUES($1, $2) RETURNING meal_description_id AS id";
            Ok(sqlx::query_as::<_, Id>(query)
                .bind(&meal.description)
                .bind(user.registered_user_id)
                .fetch_one(transaction)
                .await?
                .id)
        }
    }

    async fn insert_person(
        transaction: &mut Transaction<'_, Postgres>,
        meal: &ij::Meal,
        user: &ModelUser,
    ) -> Result<i64, ApiError> {
        let query = "SELECT meal_person_id AS id FROM meal_person WHERE person = $1";
        if let Some(id) = sqlx::query_as::<_, Id>(query)
            .bind(meal.person.to_string())
            .fetch_optional(&mut *transaction)
            .await?
        {
            Ok(id.id)
        } else {
            let query = "INSERT INTO meal_person(person, registered_user_id) VALUES(%1$L, %2$L) RETURNING meal_person_id AS id";
            Ok(sqlx::query_as::<_, Id>(query)
                .bind(meal.person.to_string())
                .bind(user.registered_user_id)
                .fetch_one(transaction)
                .await?
                .id)
        }
    }

    async fn insert_photo(
        transaction: &mut Transaction<'_, Postgres>,
        converted: &ij::PhotoName,
        original: &ij::PhotoName,
        user: &ModelUser,
    ) -> Result<i64, ApiError> {
        let query = "SELECT meal_photo_id AS id FROM meal_photo WHERE photo_original = $1 AND photo_converted = $2";
        if let Some(id) = sqlx::query_as::<_, Id>(query)
            .bind(original.to_string())
            .bind(converted.to_string())
            .fetch_optional(&mut *transaction)
            .await?
        {
            Ok(id.id)
        } else {
            let query = "INSERT INTO meal_photo(photo_original, photo_converted, registered_user_id) VALUES($1, $2, $3) RETURNING meal_photo_id AS id";
            Ok(sqlx::query_as::<_, Id>(query)
                .bind(original.to_string())
                .bind(converted.to_string())
                .bind(user.registered_user_id)
                .fetch_one(transaction)
                .await?
                .id)
        }
    }

    async fn delete_empty(
        transaction: &mut Transaction<'_, Postgres>,
        meal: &Self,
    ) -> Result<Option<(String, String)>, ApiError> {
        let query = "DELETE FROM meal_category WHERE meal_category_id = $1 AND (SELECT count(*) from individual_meal WHERE meal_category_id = $1) = 0";
        sqlx::query(query)
            .bind(meal.meal_category_id)
            .execute(&mut *transaction)
            .await?;

        let query = "DELETE FROM meal_date WHERE meal_date_id = $1 AND (SELECT count(*) from individual_meal WHERE meal_date_id = $1) = 0";
        sqlx::query(query)
            .bind(meal.meal_date_id)
            .execute(&mut *transaction)
            .await?;

        let query = "DELETE FROM meal_description WHERE meal_description_id = $1 AND (SELECT count(*) from individual_meal WHERE meal_description_id = $1) = 0";
        sqlx::query(query)
            .bind(meal.meal_description_id)
            .execute(&mut *transaction)
            .await?;

        let output = if let (Some(photo_id), Some(converted), Some(original)) = (
            meal.meal_photo_id.as_ref(),
            meal.photo_converted.as_ref(),
            meal.photo_original.as_ref(),
        ) {
            let query = "DELETE FROM meal_photo WHERE meal_photo_id = $1 AND (SELECT count(*) from individual_meal WHERE meal_photo_id = $1) = 0";
            sqlx::query(query)
                .bind(photo_id)
                .execute(&mut *transaction)
                .await?;
            Some((converted.clone(), original.clone()))
        } else {
            None
        };

        Ok(output)
    }

    // Delete all redis meal caches - when delete/insert/update a meal - or admin user from /food/cache route
    pub async fn delete_cache(redis: &Arc<Mutex<Connection>>) -> Result<(), ApiError> {
        tokio::try_join!(
            ModelIndividualFood::delete_cache(redis),
            ModelFoodLastId::delete_cache(redis),
            ModelFoodCategory::delete_cache(redis),
        )?;
        Ok(())
    }

    pub async fn get(
        postgres: &PgPool,
        person: &Person,
        date: Date,
    ) -> Result<Option<Self>, ApiError> {
        let query = "
SELECT
	im.individual_meal_id,
	md.date_of_meal as meal_date, md.meal_date_id,
	p.person,
	mc.category, mc.meal_category_id,
	mde.description, mde.meal_description_id,
	CASE WHEN im.restaurant IS null THEN false ELSE im.restaurant END AS restaurant,
	CASE WHEN im.takeaway IS null THEN false ELSE im.takeaway END AS takeaway,
	CASE WHEN im.vegetarian IS null THEN false ELSE im.vegetarian END AS vegetarian,
	im.meal_photo_id,
	mp.photo_original, mp.photo_converted
FROM
	individual_meal im
JOIN
	meal_person p
ON
	im.meal_person_id = p.meal_person_id
JOIN
	meal_date md
ON
	im.meal_date_id = md.meal_date_id
JOIN
	meal_category mc
ON
	 im.meal_category_id = mc.meal_category_id
JOIN
	meal_description mde
ON
	im.meal_description_id = mde.meal_description_id
LEFT JOIN
	meal_photo mp
ON
	im.meal_photo_id = mp.meal_photo_id
WHERE
	md.date_of_meal = $1
AND
	p.person = $2";

        Ok(sqlx::query_as::<_, Self>(query)
            .bind(date)
            .bind(person.to_string())
            .fetch_optional(postgres)
            .await?)
    }

    //     pub async fn get_by_id(postgres: &PgPool, id: i64) -> Result<Option<Self>, ApiError> {
    //         let query = "
    // SELECT
    // 	im.individual_meal_id,
    // 	md.date_of_meal as meal_date, md.meal_date_id,
    // 	p.person,
    // 	mc.category, mc.meal_category_id,
    // 	mde.description, mde.meal_description_id,
    // 	CASE WHEN im.restaurant IS null THEN false ELSE im.restaurant END AS restaurant,
    // 	CASE WHEN im.takeaway IS null THEN false ELSE im.takeaway END AS takeaway,
    // 	CASE WHEN im.vegetarian IS null THEN false ELSE im.vegetarian END AS vegetarian,
    // 	im.meal_photo_id,
    // 	mp.photo_original, mp.photo_converted
    // FROM
    // 	individual_meal im
    // JOIN
    // 	meal_person p
    // ON
    // 	im.meal_person_id = p.meal_person_id
    // JOIN
    // 	meal_date md
    // ON
    // 	im.meal_date_id = md.meal_date_id
    // JOIN
    // 	meal_category mc
    // ON
    // 	 im.meal_category_id = mc.meal_category_id
    // JOIN
    // 	meal_description mde
    // ON
    // 	im.meal_description_id = mde.meal_description_id
    // LEFT JOIN
    // 	meal_photo mp
    // ON
    // 	im.meal_photo_id = mp.meal_photo_id
    // WHERE
    // 	im.individual_meal_id = $1";
    //         Ok(sqlx::query_as::<_, Self>(query)
    //             .bind(id)
    //             .fetch_optional(postgres)
    //             .await?)
    //     }

    /// Insert a new meal, and also clear the redis meal cache
    pub async fn insert(
        postgres: &PgPool,
        redis: &Arc<Mutex<Connection>>,
        meal: &ij::Meal,
        user: &ModelUser,
    ) -> Result<(), ApiError> {
        let mut transaction = postgres.begin().await?;

        let description_id = Self::insert_description(&mut transaction, meal, user).await?;
        let category_id = Self::insert_category(&mut transaction, meal, user).await?;
        let date_id = Self::insert_date(&mut transaction, meal, user).await?;
        let meal_person_id = Self::insert_person(&mut transaction, meal, user).await?;

        let photo_id = if let (Some(converted), Some(original)) =
            (meal.photo_converted.as_ref(), meal.photo_original.as_ref())
        {
            Some(Self::insert_photo(&mut transaction, converted, original, user).await?)
        } else {
            None
        };

        let query = r#"
INSERT INTO individual_meal
	(registered_user_id, meal_category_id, meal_date_id, meal_description_id, meal_person_id, meal_photo_id, restaurant, takeaway, vegetarian)
VALUES
	($1, $2, $3, $4, $5, $6, $7, $8, $9)"#;
        sqlx::query(query)
            .bind(user.registered_user_id)
            .bind(category_id)
            .bind(date_id)
            .bind(description_id)
            .bind(meal_person_id)
            .bind(photo_id)
            .bind(meal.restaurant)
            .bind(meal.takeaway)
            .bind(meal.vegetarian)
            .execute(&mut *transaction)
            .await?;

        Self::delete_cache(redis).await?;
        Ok(transaction.commit().await?)
    }

    pub async fn update(
        postgres: &PgPool,
        redis: &Arc<Mutex<Connection>>,
        meal: &ij::Meal,
        user: &ModelUser,
        original_meal: &Self,
    ) -> Result<(), ApiError> {
        let mut transaction = postgres.begin().await?;

        let description_id = Self::insert_description(&mut transaction, meal, user).await?;
        let category_id = Self::insert_category(&mut transaction, meal, user).await?;
        let date_id = Self::insert_date(&mut transaction, meal, user).await?;
        let meal_person_id = Self::insert_person(&mut transaction, meal, user).await?;

        let photo_id = if let (Some(converted), Some(original)) =
            (meal.photo_converted.as_ref(), meal.photo_original.as_ref())
        {
            Some(Self::insert_photo(&mut transaction, converted, original, user).await?)
        } else {
            None
        };

        let query = r#"
UPDATE
	individual_meal
SET
	meal_category_id = $1,
	meal_date_id = $2,
	meal_description_id = $3,
	meal_person_id = $4,
	meal_photo_id = $5,
	restaurant = $6,
	takeaway = $7,
	vegetarian = $8
WHERE
	individual_meal_id = $9"#;
        sqlx::query(query)
            .bind(category_id)
            .bind(date_id)
            .bind(description_id)
            .bind(meal_person_id)
            .bind(photo_id)
            .bind(meal.restaurant)
            .bind(meal.takeaway)
            .bind(meal.vegetarian)
            .bind(original_meal.individual_meal_id)
            .execute(&mut *transaction)
            .await?;
        Self::delete_empty(&mut transaction, original_meal).await?;
        Self::delete_cache(redis).await?;
        Ok(transaction.commit().await?)
    }

    pub async fn delete(
        postgres: &PgPool,
        redis: &Arc<Mutex<Connection>>,
        person: &Person,
        date: Date,
    ) -> Result<Option<(String, String)>, ApiError> {
        if let Some(meal) = Self::get(postgres, person, date).await? {
            let mut transaction = postgres.begin().await?;
            let query = "DELETE FROM individual_meal WHERE individual_meal_id = $1";
            sqlx::query(query)
                .bind(meal.individual_meal_id)
                .execute(&mut transaction)
                .await?;
            let output = Self::delete_empty(&mut transaction, &meal).await?;
            Self::delete_cache(redis).await?;
            transaction.commit().await?;
            Ok(output)
        } else {
            Err(ApiError::InvalidValue("Unknown meal".to_owned()))
        }
    }
}
