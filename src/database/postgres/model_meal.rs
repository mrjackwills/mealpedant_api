use jiff_sqlx::ToSqlx;
use sqlx::{PgPool, Postgres, Transaction};

use crate::{C, S, api_error::ApiError, servers::ij};

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
    pub meal_date: jiff_sqlx::Date,
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
        if let Some(id) = sqlx::query_as!(
            Id,
            "SELECT meal_date_id AS id FROM meal_date WHERE date_of_meal::TEXT = $1",
            meal.date.to_string()
        )
        .fetch_optional(&mut **transaction)
        .await?
        {
            Ok(id.id)
        } else {
            Ok(sqlx::query_as!(Id, "INSERT INTO meal_date(date_of_meal, registered_user_id) VALUES(($1::TEXT)::DATE, $2) RETURNING meal_date_id AS id", meal.date.to_string(), user.registered_user_id).fetch_one(&mut **transaction).await?.id)
        }
    }

    async fn insert_category(
        transaction: &mut Transaction<'_, Postgres>,
        meal: &ij::Meal,
        user: &ModelUser,
    ) -> Result<i64, ApiError> {
        if let Some(id) = sqlx::query_as!(
            Id,
            "SELECT meal_category_id AS id FROM meal_category WHERE category = upper($1)",
            &meal.category
        )
        .fetch_optional(&mut **transaction)
        .await?
        {
            Ok(id.id)
        } else {
            Ok(sqlx::query_as!(Id, "INSERT INTO meal_category(category, registered_user_id) VALUES(upper($1), $2) RETURNING meal_category_id AS id", &meal.category, user.registered_user_id).fetch_one(&mut ** transaction).await?.id)
        }
    }

    async fn insert_description(
        transaction: &mut Transaction<'_, Postgres>,
        meal: &ij::Meal,
        user: &ModelUser,
    ) -> Result<i64, ApiError> {
        if let Some(id) = sqlx::query_as!(
            Id,
            "SELECT meal_description_id AS id FROM meal_description WHERE description = $1",
            &meal.description
        )
        .fetch_optional(&mut **transaction)
        .await?
        {
            Ok(id.id)
        } else {
            Ok(sqlx::query_as!(Id, "INSERT INTO meal_description(description, registered_user_id) VALUES($1, $2) RETURNING meal_description_id AS id", &meal.description, user.registered_user_id).fetch_one(&mut ** transaction).await?.id)
        }
    }

    async fn insert_person(
        transaction: &mut Transaction<'_, Postgres>,
        meal: &ij::Meal,
        user: &ModelUser,
    ) -> Result<i64, ApiError> {
        if let Some(id) = sqlx::query_as!(
            Id,
            "SELECT meal_person_id AS id FROM meal_person WHERE person = $1",
            meal.person.to_string()
        )
        .fetch_optional(&mut **transaction)
        .await?
        {
            Ok(id.id)
        } else {
            Ok(sqlx::query_as!(Id, "INSERT INTO meal_person(person, registered_user_id) VALUES($1, $2) RETURNING meal_person_id AS id", meal.person.to_string(), user.registered_user_id).fetch_one(&mut **transaction).await?.id)
        }
    }

    async fn insert_photo(
        transaction: &mut Transaction<'_, Postgres>,
        converted: &ij::PhotoName,
        original: &ij::PhotoName,
        user: &ModelUser,
    ) -> Result<i64, ApiError> {
        if let Some(id) = sqlx::query_as!(Id,"SELECT meal_photo_id AS id FROM meal_photo WHERE photo_original = $1 AND photo_converted = $2",
            original.to_string(),
            converted.to_string())
            .fetch_optional(&mut **transaction)
            .await?
        {
            Ok(id.id)
        } else {
            Ok(sqlx::query_as!(Id, "INSERT INTO meal_photo(photo_original, photo_converted, registered_user_id) VALUES($1, $2, $3) RETURNING meal_photo_id AS id",
                original.to_string(),
                converted.to_string(),
                user.registered_user_id)
                .fetch_one(&mut **transaction)
                .await?
                .id)
        }
    }

    /// Search for categories, dates, photos, and descriptions, that are dangling, and delete them from postgres
    async fn delete_empty(
        transaction: &mut Transaction<'_, Postgres>,
        meal: &Self,
    ) -> Result<Option<(String, String)>, ApiError> {
        sqlx::query!("DELETE FROM meal_category WHERE meal_category_id = $1 AND (SELECT count(*) from individual_meal WHERE meal_category_id = $1) = 0", 
            meal.meal_category_id)
            .execute(&mut **transaction)
            .await?;

        sqlx::query!("DELETE FROM meal_date WHERE meal_date_id = $1 AND (SELECT count(*) from individual_meal WHERE meal_date_id = $1) = 0",
            meal.meal_date_id)
            .execute(&mut **transaction)
            .await?;

        sqlx::query!("DELETE FROM meal_description WHERE meal_description_id = $1 AND (SELECT count(*) from individual_meal WHERE meal_description_id = $1) = 0",
            meal.meal_description_id)
            .execute(&mut **transaction)
            .await?;
        Ok(
            if let Some(photo_id) = &meal.meal_photo_id
                && let Some(converted) = &meal.photo_converted
                && let Some(original) = &meal.photo_original
            {
                sqlx::query!("DELETE FROM meal_photo WHERE meal_photo_id = $1 AND (SELECT count(*) from individual_meal WHERE meal_photo_id = $1) = 0",
                photo_id)
                .execute(&mut **transaction)
                .await?;
                Some((C!(converted), C!(original)))
            } else {
                None
            },
        )
    }

    /// Get a single meal by date + person
    pub async fn get_by_date_person(
        postgres: &PgPool,
        person: &Person,
        date: jiff::civil::Date,
    ) -> Result<Option<Self>, ApiError> {
        let query = "
SELECT
    im.individual_meal_id,
    md.date_of_meal as meal_date,
    md.meal_date_id,
    p.person,
    mc.category,
    mc.meal_category_id,
    mde.description,
    mde.meal_description_id,
    CASE
        WHEN im.restaurant IS null THEN false
        ELSE im.restaurant
    END AS restaurant,
    CASE
        WHEN im.takeaway IS null THEN false
        ELSE im.takeaway
    END AS takeaway,
    CASE
        WHEN im.vegetarian IS null THEN false
        ELSE im.vegetarian
    END AS vegetarian,
    im.meal_photo_id,
    mp.photo_original,
    mp.photo_converted
FROM
    individual_meal im
    JOIN meal_person p USING(meal_person_id)
    JOIN meal_date md USING(meal_date_id)
    JOIN meal_category mc USING(meal_category_id)
    JOIN meal_description mde USING(meal_description_id)
    LEFT JOIN meal_photo mp USING(meal_photo_id)
WHERE
    md.date_of_meal = $1
    AND p.person = $2";

        Ok(sqlx::query_as::<_, Self>(query)
            .bind(date.to_sqlx())
            .bind(person.to_string())
            .fetch_optional(postgres)
            .await?)
    }

    /// Insert a new meal, and also clear the redis meal cache
    pub async fn insert(
        postgres: &PgPool,
        meal: &ij::Meal,
        user: &ModelUser,
    ) -> Result<(), ApiError> {
        let mut transaction = postgres.begin().await?;

        let description_id = Self::insert_description(&mut transaction, meal, user).await?;
        let category_id = Self::insert_category(&mut transaction, meal, user).await?;
        let date_id = Self::insert_date(&mut transaction, meal, user).await?;
        let meal_person_id = Self::insert_person(&mut transaction, meal, user).await?;

        let photo_id = if let Some(converted) = &meal.photo_converted
            && let Some(original) = &meal.photo_original
        {
            Some(Self::insert_photo(&mut transaction, converted, original, user).await?)
        } else {
            None
        };

        sqlx::query!("
INSERT INTO individual_meal
    (registered_user_id, meal_category_id, meal_date_id, meal_description_id, meal_person_id, meal_photo_id, restaurant, takeaway, vegetarian)
VALUES
    ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            user.registered_user_id,
            category_id,
            date_id,
            description_id,
            meal_person_id,
            photo_id,
            meal.restaurant,
            meal.takeaway,
            meal.vegetarian)
            .execute(&mut *transaction)
            .await?;

        transaction.commit().await?;
        Ok(())
    }

    pub async fn update(
        postgres: &PgPool,
        meal: &ij::Meal,
        user: &ModelUser,
        original_meal: &Self,
    ) -> Result<(), ApiError> {
        let mut transaction = postgres.begin().await?;

        let description_id = Self::insert_description(&mut transaction, meal, user).await?;
        let category_id = Self::insert_category(&mut transaction, meal, user).await?;
        let date_id = Self::insert_date(&mut transaction, meal, user).await?;
        let meal_person_id = Self::insert_person(&mut transaction, meal, user).await?;

        let photo_id = if let Some(converted) = &meal.photo_converted
            && let Some(original) = &meal.photo_original
        {
            Some(Self::insert_photo(&mut transaction, converted, original, user).await?)
        } else {
            None
        };

        sqlx::query!(
            "
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
    individual_meal_id = $9",
            category_id,
            date_id,
            description_id,
            meal_person_id,
            photo_id,
            meal.restaurant,
            meal.takeaway,
            meal.vegetarian,
            original_meal.individual_meal_id
        )
        .execute(&mut *transaction)
        .await?;
        Self::delete_empty(&mut transaction, original_meal).await?;
        transaction.commit().await?;
        Ok(())
    }

    pub async fn delete(
        postgres: &PgPool,
        person: &Person,
        date: jiff::civil::Date,
    ) -> Result<Option<(String, String)>, ApiError> {
        match Self::get_by_date_person(postgres, person, date).await? {
            Some(meal) => {
                let mut transaction = postgres.begin().await?;
                sqlx::query!(
                    "DELETE FROM individual_meal WHERE individual_meal_id = $1",
                    meal.individual_meal_id
                )
                .execute(&mut *transaction)
                .await?;
                let output = Self::delete_empty(&mut transaction, &meal).await?;
                transaction.commit().await?;
                Ok(output)
            }
            _ => Err(ApiError::InvalidValue(S!("Unknown meal"))),
        }
    }
}
