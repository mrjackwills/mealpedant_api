import { Pool } from 'pg';
import { postgresql } from '../../config/db_postgres';
import * as types from '../../types';
import format from 'pg-format';

class Food_Queries {
	#db!: Pool;

	constructor (db:Pool) {
		this.#db = db;
	}
	
	async select_allCategory (): Promise<Array<types.TCategories>> {
		const query = format(
			// eslint-disable-next-line indent
	`SELECT
		im.meal_category_id AS id,
		mc.category AS c,
		count(mc.category) AS n
	FROM individual_meal im
	JOIN meal_category mc
		ON im.meal_category_id = mc.meal_category_id
	GROUP BY c, id ORDER BY n DESC`);
		const { rows } = await this.#db.query(query);
		return rows;
	}
	
	async select_allMeal (): Promise<Array<types.TAllMealVue>> {
		const query = format (
			// TODO change this to run on every mealdate and join from there
			// eslint-disable-next-line indent
	`SELECT
		md.date_of_meal::text as date,
		mpe.person as person,
		im.meal_category_id as category, im.restaurant as restaurant, im.takeaway as takeaway, im.vegetarian as vegetarian,
		mde.description as description,
		mp.photo_original as photo_original, mp.photo_converted AS photo_converted
	FROM individual_meal im
	JOIN meal_date md
		ON im.meal_date_id = md.meal_date_id
	JOIN meal_description mde
		ON im.meal_description_id = mde.meal_description_id
	JOIN meal_person mpe
		ON mpe.meal_person_id = im.meal_person_id
	LEFT JOIN meal_photo mp
		ON im.meal_photo_id = mp.meal_photo_id
	ORDER BY date DESC, person ASC`
		);
		// Messy
		const rows: Array<types.TMealRow> = (await this.#db.query(query)).rows;
		const cleanedData: Array<types.TAllMealVue>= [];
		// TODO refactor this into it's own method, query should just return raw postgres data!
		for (const item of rows) {
			const cleanDataIndex = cleanedData.findIndex((o) => o.ds === item.date);
			const p = item.person === 'Dave' ? 'D' : 'J';
			const baseMeal: types.TBaseMealVue = {
				md: item.description,
				c: item.category
			};
			if (item.restaurant) baseMeal.r = item.restaurant;
			if (item.takeaway) baseMeal.t = item.takeaway;
			if (item.vegetarian) baseMeal.v = item.vegetarian;
			if (item.photo_original && item.photo_converted) baseMeal.p = {
				o: item.photo_original,
				c: item.photo_converted
			};
			if (cleanDataIndex >= 0 && cleanedData) {
				const mealDate = cleanedData[cleanDataIndex];
				if (mealDate) {
					mealDate[p] = baseMeal ;
					cleanedData[cleanDataIndex] = mealDate;
				}
			}
			else cleanedData.push({
				ds: item.date,
				[p]: baseMeal
			});
		}
		return cleanedData;
	}
	async select_lastId (): Promise<types.lastId|undefined> {
		const query = format(`SELECT individual_meal_audit_id FROM individual_meal_audit ORDER BY individual_meal_audit_id DESC LIMIT 1`);
		const { rows } = await this.#db.query(query);
		return rows[0] ? rows[0].individual_meal_audit_id : undefined;
	}
	
}

export const foodQueries = new Food_Queries(postgresql);
