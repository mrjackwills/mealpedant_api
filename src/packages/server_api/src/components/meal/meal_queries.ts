import { customTypeError } from '../../config/customError';
import { Pool, PoolClient } from 'pg';
import { postgresql } from '../../config/db_postgres';
import * as types from '../../types';
import format from 'pg-format';

class Meal_Queries {
	#db!: Pool;

	constructor (db:Pool) {
		this.#db = db;
	}
	
	async #select_individualMeal (mealId: types.mealId, Client: PoolClient): Promise<types.TIndividualMeal> {
		if (!mealId || !Client) throw customTypeError('select_individualMeal(): !mealId || !Client');
		const individualMeal_query = format (
			// eslint-disable-next-line indent
	`SELECT
		meal_date_id, meal_category_id, meal_description_id, meal_photo_id
	FROM
		individual_meal
	WHERE
		individual_meal_id = %1$L`,
			mealId);
		const { rows } = await Client.query(individualMeal_query);
		return rows[0];
	}
	
	async #select_insert_category ({ category, userId, Client }: types.TSelectInsertCategory): Promise<types.mealCategoryId> {
		if (!category || !userId || !Client) throw customTypeError('select_insert_category(): !category || !userId || !Client');
		const select_categoryId_query = format(`SELECT meal_category_id FROM meal_category WHERE category = upper(%1$L)`,
			category);
		const { rows: categoryId } = await Client.query(select_categoryId_query);
		if (categoryId[0]?.meal_category_id) return categoryId[0].meal_category_id;
		const insert_category_query = format(`INSERT INTO meal_category(category, registered_user_id) VALUES(upper(%1$L), %2$L) RETURNING meal_category_id`,
			category, userId);
		const { rows } = await Client.query(insert_category_query);
		return rows[0].meal_category_id;
	}
	
	async #select_insert_date ({ date, userId, Client }: types.TSelectInsertDate): Promise<types.mealDateId> {
		if (!date || !userId || !Client) throw customTypeError('select_insert_date(): !date || !userId || !Client');
		const select_dateId_query = format(`SELECT meal_date_id FROM meal_date WHERE date_of_meal = %1$L`,
			date);
		const { rows: dateId } = await Client.query(select_dateId_query);
		if (dateId[0]?.meal_date_id) return dateId[0].meal_date_id;
		const insert_date_query = format(`INSERT INTO meal_date(date_of_meal, registered_user_id) VALUES(%1$L, %2$L) RETURNING meal_date_id`,
			date, userId);
		const { rows } = await Client.query(insert_date_query);
		return rows[0].meal_date_id;
	}
	
	async #select_insert_description ({ description, userId, Client }: types.TSelectInsertDescription): Promise<types.mealDescriptionId> {
		if (!description || !userId || !Client) throw customTypeError('select_insert_description(): !description || !userId || !Client');
		const select_descriptionId_query = format(`SELECT meal_description_id FROM meal_description WHERE description = %1$L`,
			description);
		const { rows: descriptionId } = await Client.query(select_descriptionId_query);
		if (descriptionId[0]?.meal_description_id) return descriptionId[0].meal_description_id;
		const insert_description_query = format(`INSERT INTO meal_description(description, registered_user_id) VALUES(%1$L, %2$L) RETURNING meal_description_id`,
			description, userId);
		const { rows } = await Client.query(insert_description_query);
		return rows[0].meal_description_id;
	}
	
	async #select_insert_person ({ person, userId, Client }: types.TSelectInsertPerson): Promise<types.personId> {
		if (!person || !userId || !Client) throw customTypeError('select_insert_person(): !person || !userId || !Client');
		const select_personId_query = format(`SELECT meal_person_id FROM meal_person WHERE person = %1$L`,
			person);
		const { rows: personId } = await Client.query(select_personId_query);
		if (personId[0]?.meal_person_id) return personId[0].meal_person_id;
		const insert_person_query = format(`INSERT INTO meal_person(person, registered_user_id) VALUES(%1$L, %2$L) RETURNING meal_person_id`,
			person, userId);
		const { rows } = await Client.query(insert_person_query);
		return rows[0].meal_person_id;
	}
	
	async #select_insert_photo ({ original, converted, userId, Client }: types.TSelectInsertPhoto): Promise<types.mealPhotoId> {
		if (!original || !converted|| !userId || !Client) throw customTypeError('select_insert_photo(): !original || !converted|| !userId || !Client');
		const select_photoId_query = format(`SELECT meal_photo_id FROM meal_photo WHERE photo_original = %1$L AND photo_converted = %2$L`,
			original, converted);
		const { rows: photoId } = await Client.query(select_photoId_query);
		if (photoId[0]?.meal_photo_id) return photoId[0].meal_photo_id;
		const insert_photo_query = format(`INSERT INTO meal_photo(photo_original, photo_converted, registered_user_id) VALUES(%1$L, %2$L, %3$L) RETURNING meal_photo_id`,
			original, converted, userId);
		const { rows } = await Client.query(insert_photo_query);
		return rows[0].meal_photo_id;
	}
	
	async #delete_selectCount_category (categoryId: string, Client: PoolClient): Promise<void> {
		if (!categoryId || !Client) throw customTypeError('delete_category(): !categoryId || !Client');
		const categoryCount_query = format(`SELECT count(*) from individual_meal WHERE meal_category_id = %1$L`,
			categoryId);
		const { rows: categoryCount } = await Client.query(categoryCount_query);
		if (categoryCount[0]?.count >=1) return;
		const delete_category_query = format(`DELETE FROM meal_category WHERE meal_category_id = %1$L`,
			categoryId);
		await Client.query(delete_category_query);
	}
	
	async #delete_selectCount_date (dateId: types.mealDateId, Client: PoolClient): Promise<void> {
		if (!dateId || !Client) throw customTypeError('delete_selectCount_date(): !dateId || !Client');
		const dateCount_query = format(`SELECT count(*) from individual_meal WHERE meal_date_id = %1$L`,
			dateId);
		const { rows: dateCount } = await Client.query(dateCount_query);
		if (dateCount[0]?.count >=1) return;
		const delete_date_query = format(`DELETE FROM meal_date WHERE meal_date_id = %1$L`,
			dateId);
		await Client.query(delete_date_query);
	}
	
	async #delete_selectCount_description (descriptionId: types.mealDescriptionId, Client: PoolClient): Promise<void> {
		if (!descriptionId || !Client) throw customTypeError('delete_selectCount_description(): !descriptionId || !Client');
		const descriptionCount_query = format(`SELECT count(*) from individual_meal WHERE meal_description_id = %1$L`,
			descriptionId);
		const { rows: descriptionCount } = await Client.query(descriptionCount_query);
		if (descriptionCount[0]?.count >=1) return;
		const delete_description_query = format(`DELETE FROM meal_description WHERE meal_description_id = %1$L`,
			descriptionId);
		await Client.query(delete_description_query);
	}
	
	async #delete_selectCount_photo (photoId: types.mealPhotoId, Client: PoolClient): Promise<void> {
		if (!photoId || !Client) throw customTypeError('delete_selectCount_photo(): !photoId || !Client');
		const descriptionCount_query = format(`SELECT count(*) from individual_meal WHERE meal_photo_id = %1$L`,
			photoId);
		const { rows: photoCount } = await Client.query(descriptionCount_query);
		if (photoCount[0]?.count >=1) return;
		const delete_photo_query = format(`DELETE FROM meal_photo WHERE meal_photo_id = %1$L`,
			photoId);
		await Client.query(delete_photo_query);
	}
	
	async select_missing_meals (): Promise<Array<types.TMissingMeals>> {
		const difference_query = format (
			// eslint-disable-next-line indent
	`WITH
		all_dates
		AS
		( SELECT missing_date::date FROM generate_series('2015-05-09', now() - interval '1 day', interval '1 day') AS missing_date)
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
	ORDER BY missing_date DESC, person ASC;
	`);
		const { rows } = await this.#db.query(difference_query);
		return rows;
	}
	
	async delete_meal_transaction (mealId: types.mealId): Promise<void> {
		if (!mealId) throw customTypeError('delete_meal_transaction: !mealId');
		const Client = await this.#db.connect();
		try {
			await Client.query('BEGIN');
			const meal_detail = await this.#select_individualMeal(mealId, Client);
			if (!meal_detail) throw customTypeError('No meal found with that id');
			const [ categoryId, dateId, descriptionId, photoId ] = [
				meal_detail.meal_category_id,
				meal_detail.meal_date_id,
				meal_detail.meal_description_id,
				meal_detail.meal_photo_id,
			];
	
			// Delete invidiual meal
			const delete_query = format(`DELETE FROM individual_meal WHERE individual_meal_id = %1$L`,
				mealId);
			await Client.query(delete_query);
				
			// Get counts of category, date, and description, if zero, then delete
			const promiseList = [
				this.#delete_selectCount_category(categoryId, Client),
				this.#delete_selectCount_date(dateId, Client),
				this.#delete_selectCount_description(descriptionId, Client),
			];
			if (photoId) promiseList.push(this.#delete_selectCount_photo(photoId, Client));
			await Promise.all(promiseList);
			await Client.query('COMMIT');
		} catch (e) {
			await Client.query('ROLLBACK');
			throw e;
		} finally {
			Client.release();
		}
	}
	
	async insert_meal_transaction (meal: types.TInsertMeal, userId: types.UserId): Promise<void> {
		if (!meal || !userId || !meal.person || !meal.date || !meal.category || !meal.description) {
			throw customTypeError('insert_meal_transaction(): !meal || !userId || !meal.person || !meal.date || !meal.category || !meal.description');
		}
		const Client = await this.#db.connect();
		try {
			await Client.query('BEGIN');
	
			const promiseList = [
				this.#select_insert_person({ person: meal.person, userId, Client }),
				this.#select_insert_category({ category: meal.category, userId, Client }),
				this.#select_insert_date({ date: meal.date, userId, Client }),
				this.#select_insert_description({ description: meal.description, userId, Client }),
			] as const;
			const [ personId, categoryId, dateId, descriptionId ] = await Promise.all(promiseList);
	
			const photoId = meal.photoNameConverted && meal.photoNameOriginal? await this.#select_insert_photo({ original: meal.photoNameOriginal, converted: meal.photoNameConverted, userId, Client }) : undefined;
	
			const insert_meal = format (
				// eslint-disable-next-line indent
	`INSERT INTO individual_meal
		(registered_user_id, meal_category_id, meal_date_id, meal_description_id, meal_person_id, meal_photo_id, restaurant, takeaway, vegetarian)
	VALUES
		(%1$L, %2$L, %3$L, %4$L, %5$L, %6$L, %7$L, %8$L, %9$L)`,
				userId, categoryId, dateId, descriptionId, personId, photoId, meal.restaurant, meal.takeaway, meal.vegetarian,
			);
	
			await Client.query(insert_meal);
			await Client.query('COMMIT');
		} catch (e) {
			await Client.query('ROLLBACK');
			throw e;
		} finally {
			Client.release();
		}
	}
	
	async select_meal_byDatePerson ({ person, date }: types.TPersonDate): Promise<types.TMealDatePerson|undefined> {
		if (!person || !date) throw customTypeError('select_meal_byDatePerson: !person || !date');
		const query = format (
			// eslint-disable-next-line indent
	`SELECT
		im.individual_meal_id AS id,
		md.date_of_meal::text as date,
		p.person,
		mc.category,
		mde.description,
		CASE WHEN im.restaurant IS null THEN false ELSE im.restaurant END AS restaurant,
		CASE WHEN im.takeaway IS null THEN false ELSE im.takeaway END AS takeaway,
		CASE WHEN im.vegetarian IS null THEN false ELSE im.vegetarian END AS vegetarian,
		im.meal_photo_id,
		mp.photo_original as "photoNameOriginal",
		mp.photo_converted AS "photoNameConverted"
	FROM individual_meal im
	JOIN meal_person p
		ON im.meal_person_id = p.meal_person_id
	JOIN meal_date md
		ON im.meal_date_id = md.meal_date_id
	JOIN meal_category mc
		ON im.meal_category_id = mc.meal_category_id
	JOIN meal_description mde
		ON im.meal_description_id = mde.meal_description_id
	LEFT JOIN meal_photo mp
		ON im.meal_photo_id = mp.meal_photo_id
	WHERE
		md.date_of_meal = %1$L
		AND p.person = %2$L`,
			date, person
		);
		const { rows } = await this.#db.query(query);
		return rows[0];
	}
	
	// CASE WHEN au.admin IS null THEN false ELSE CASE WHEN au.admin IS true THEN true
	async select_meal_ById (mealId: types.mealId): Promise<types.TMealDatePerson|undefined> {
		if (!mealId) throw customTypeError('select_meal_ById: !mealId');
	
		const query = format (
			// eslint-disable-next-line indent
	`SELECT
		im.individual_meal_id AS id,
		md.date_of_meal::text as date,
		p.person,
		mc.category,
		mde.description,
		CASE WHEN im.restaurant IS null THEN false ELSE im.restaurant END AS restaurant,
		CASE WHEN im.takeaway IS null THEN false ELSE im.takeaway END AS takeaway,
		CASE WHEN im.vegetarian IS null THEN false ELSE im.vegetarian END AS vegetarian,
		im.meal_photo_id,
		mp.photo_original as "photoNameOriginal",
		mp.photo_converted AS "photoNameConverted"
	FROM individual_meal im
	JOIN meal_person p
		ON im.meal_person_id = p.meal_person_id
	JOIN meal_date md
		ON im.meal_date_id = md.meal_date_id
	JOIN meal_category mc
		ON im.meal_category_id = mc.meal_category_id
	JOIN meal_description mde
		ON im.meal_description_id = mde.meal_description_id
	LEFT JOIN meal_photo mp
		ON im.meal_photo_id = mp.meal_photo_id
	WHERE
		im.individual_meal_id = %1$L`,
			mealId
		);
		const { rows } = await this.#db.query(query);
		return rows[0];
	}
	
	async update_meal_transaction ({ mealId, newMeal, userId }: types.TUpdateMeal): Promise<void> {
		if (!mealId || !newMeal || !userId) throw customTypeError('update_meal_transaction(): !mealId || !newMeal || !userId');
		const Client = await this.#db.connect();
		try {
			await Client.query('BEGIN');
			const original_meal = await this.#select_individualMeal(mealId, Client);
	
			const promiseList = [
				this.#select_insert_person({ person: newMeal.person, userId, Client }),
				this.#select_insert_category({ category: newMeal.category, userId, Client }),
				this.#select_insert_date({ date: newMeal.date, userId, Client }),
				this.#select_insert_description({ description: newMeal.description, userId, Client })
			] as const;
			const photoId = newMeal.photoNameConverted && newMeal.photoNameOriginal ? await this.#select_insert_photo({ original: newMeal.photoNameOriginal, converted: newMeal.photoNameConverted, userId, Client }): undefined;
			const [ personId, categoryId, dateId, descriptionId ] = await Promise.all(promiseList);
	
			// Update individual meal with ids that were already in db, or the newly inserted id
			const update_query = format(
				// eslint-disable-next-line indent
	`UPDATE
		individual_meal
	SET
		meal_category_id = %1$L,
		meal_date_id = %2$L,
		meal_description_id = %3$L,
		meal_person_id = %4$L,
		meal_photo_id = %5$L,
		registered_user_id = %6$L,
		restaurant = %7$L,
		takeaway = %8$L,
		vegetarian = %9$L
	WHERE
		individual_meal_id = %10$L`,
				categoryId, dateId, descriptionId, personId, photoId, userId, newMeal.restaurant, newMeal.takeaway, newMeal.vegetarian, mealId);
	
			await Client.query(update_query);
	
			// Count original id's, delete if count = 0
			const delete_promiseList = [
				this.#delete_selectCount_category(original_meal.meal_category_id, Client),
				this.#delete_selectCount_date(original_meal.meal_date_id, Client),
				this.#delete_selectCount_description(original_meal.meal_description_id, Client),
			];
			if (original_meal.meal_photo_id) delete_promiseList.push(this.#delete_selectCount_photo(original_meal.meal_photo_id, Client));
			await Promise.all(delete_promiseList);
	
			await Client.query('COMMIT');
		} catch (e) {
			await Client.query('ROLLBACK');
			throw e;
		} finally {
			Client.release();
		}
	}

}
export const mealQueries = new Meal_Queries(postgresql);