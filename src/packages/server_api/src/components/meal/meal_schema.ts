import { schema_shared } from '../shared/shared_schema';
import * as joi from 'types-joi';

class Meal {
	// eslint-disable-next-line max-len
	private readonly yymmddRegex = /^(?:(?:(?:(?:(?:[1-9]\d)(?:0[48]|[2468][048]|[13579][26])|(?:(?:[2468][048]|[13579][26])00))(\/|-|\.)(?:0?2\1(?:29)))|(?:(?:[1-9]\d{3})(\/|-|\.)(?:(?:(?:0?[13578]|1[02])\2(?:31))|(?:(?:0?[13-9]|1[0-2])\2(?:29|30))|(?:(?:0?[1-9])|(?:1[0-2]))\2(?:0?[1-9]|1\d|2[0-8])))))$/;
	private readonly id = joi.string().regex(/^\d+$/).label('ID required');
	private readonly person = joi.string().regex(/^Dave$|^Jack$/).required().label('person unrecognised');
	private readonly yyyymmdd = joi.string().regex(this.yymmddRegex).required().label('date invalid');

	private readonly baseMeal = {
		date: this.yyyymmdd,
		person: this.person,
		category: joi.string().required().label('category'),
		description: joi.string().required().label('description'),
		restaurant: joi.boolean().strict(true).required().label('restaurant'),
		takeaway: joi.boolean().strict(true).required().label('takeaway'),
		vegetarian: joi.boolean().strict(true).required().label('vegetarian'),
		photoNameOriginal: schema_shared.imageNameOriginal.optional().allow(null).label('original photo'),
		photoNameConverted: schema_shared.imageNameConverted.optional().allow(null).label('converted photo'),
	};

	// add new meal schema
	readonly addMeal = joi.object({
		meal: joi.object({
			...this.baseMeal
		}).required()
	}).required();

	// Delete meal schema, & admin/activePatch only id required
	readonly deleteMeal = joi.object({
		id: this.id.required(),
		password: schema_shared.password,
		token: schema_shared.eitherToken.allow(null),
		// Possibly nullable?
		twoFABackup: joi.boolean().strict(true).allow(null),
	}).required();

	// Edit meal schema
	readonly editMeal = joi.object({
		meal: joi.object({
			id: this.id.required(),
			meal_photo_id: this.id.allow(null).label('meal_photo_id'),
			...this.baseMeal,

		}),
		originalDate: this.yyyymmdd,
	}).required();

	readonly singleMeal = joi.object({
		date: this.yyyymmdd,
		person: this.person,
	}).required();

}

export const schema_meal = new Meal();

export type TMealSchemaAddMeal = joi.InterfaceFrom<typeof schema_meal.addMeal>
export type TMealSchemaEditMeal = joi.InterfaceFrom<typeof schema_meal.editMeal>
export type TMealSchemaDeleteMeal = joi.InterfaceFrom<typeof schema_meal.deleteMeal>
export type TMealSchemaSingleMeal = joi.InterfaceFrom<typeof schema_meal.singleMeal>