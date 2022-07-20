import { checkPasswordAndToken } from '../../lib/checkAuthentication';
import { customError, customTypeError } from '../../config/customError';
import { ErrorMessages } from '../../types/enum_error';
import { HttpCode } from '../../types/enum_httpCode';
import { isId, isPerson } from '../../types/userGuard';
import { mealQueries } from './meal_queries';
import { schema_meal, TMealSchemaAddMeal, TMealSchemaDeleteMeal, TMealSchemaEditMeal, TMealSchemaSingleMeal } from './meal_schema';
import { send } from '../../lib/send';
import { TMealDatePerson, TPassportDeserializedUser, RequestMethod, mealId } from '../../types';
import { validate_input } from '../../lib/validateinput';
import { foodQueries } from '../food/food_queries';
import { redisQueries } from '../../lib/redisQueries';

export const missingMeals_get: RequestMethod = async (_req, res) => {
	const missingMeals = await mealQueries.select_missing_meals();
	send({ res, response: { missingMeals } });
};

const rebuildCache = async () : Promise<void> => {
	const [ allCategoryData, allMealData, lastId ] = await Promise.all([
		foodQueries.select_allCategory(),
		foodQueries.select_allMeal(),
		foodQueries.select_lastId(),
	]);
	if (!allCategoryData || !allMealData || !lastId) throw customTypeError('!allCategoryData || !allMealData || !lastId');
	await Promise.all([
		redisQueries.insert_allCategory_cache(allCategoryData),
		redisQueries.insert_allMeal_cache(allMealData),
		redisQueries.insert_lastId_cache(lastId),
	]);
};

export const meal_delete: RequestMethod = async (req, res) => {
	const body = <TMealSchemaDeleteMeal>validate_input(req.body, schema_meal.deleteMeal);

	// mealId typeGuard
	if (!isId<mealId>(body.id)) throw customTypeError(ErrorMessages.INVALID_DATA);

	// if twofa then always ask, ideally admins wouldn't be able to work without valid 2fa credentials
	await checkPasswordAndToken(req);
	await mealQueries.delete_meal_transaction(body.id);
	await rebuildCache();
	send({ res });
};

// Get an individual meals details, for editing meal
export const meal_get: RequestMethod = async (req, res) => {
	const params = <TMealSchemaSingleMeal>validate_input(req.params, schema_meal.singleMeal);

	if (!isPerson(params.person)) throw customTypeError(ErrorMessages.UNKNOWN_PERSON);

	// Validate info - same as for delete route - person & date
	const meal = await mealQueries.select_meal_byDatePerson({ date: params.date, person: params.person });
	send({ res, response: meal? { meal } : undefined });
};

// Edit an existing meal
export const meal_patch: RequestMethod = async (req, res) => {
	const user = req.user as TPassportDeserializedUser;
	
	const body = <TMealSchemaEditMeal>validate_input(req.body, schema_meal.editMeal);
	// MAYBE require password for editing meals?
	// await checkPassword({ req });

	const meal = body.meal as TMealDatePerson;
	if (!isId<mealId>(meal.id)) throw customError(HttpCode.BAD_REQUEST, ErrorMessages.UNKNOWN_MEAL);

	const originalDate = req.body.originalDate;

	// Get original meal from db
	const meal_exists = await mealQueries.select_meal_ById(meal.id);
	// Check if individual_meal exists for date & person given
	if (!meal_exists) throw customError(HttpCode.BAD_REQUEST, ErrorMessages.UNKNOWN_MEAL);

	// Make sure original date and id match with db
	if (body.originalDate !== meal_exists.date) throw customError(HttpCode.BAD_REQUEST, ErrorMessages.UNKNOWN_MEAL);
		
	const datesMatch = meal.date === originalDate;

	// If posted meal matched db meal, throw error as no edits made
	if (JSON.stringify(meal_exists) === JSON.stringify(meal)) throw customError(HttpCode.BAD_REQUEST, ErrorMessages.NO_EDITS);

	// Check if meal already exists for given person on date posted, if so then throw
	const mealOnProposedDate = await mealQueries.select_meal_byDatePerson({ person: meal.person, date: meal.date });
	if (!datesMatch && mealOnProposedDate) throw customError(HttpCode.BAD_REQUEST, ErrorMessages.MEAL_EXISTS);
	await mealQueries.update_meal_transaction({ mealId: meal.id, newMeal: meal, userId: user.registered_user_id });
	await rebuildCache();
	send({ res });
};

// Add a new meal to db
export const meal_post: RequestMethod = async (req, res) :Promise<void> => {
	const user = req.user as TPassportDeserializedUser;
	// Validate meal object
	const body = <TMealSchemaAddMeal>validate_input(req.body, schema_meal.addMeal);
	const meal = body.meal;

	// personTypeguard
	if (!isPerson(meal.person)) throw customTypeError(ErrorMessages.UNKNOWN_PERSON);
	// meal.person = meal.person as TPerson;

	// Check if individual_meal exists for date & person given
	const mealExists = await mealQueries.select_meal_byDatePerson({ date: meal.date, person: meal.person });
	// meal exists here instead
	if (mealExists) throw customError(HttpCode.BAD_REQUEST, ErrorMessages.MEAL_EXISTS);

	// const a = meal.person;
	const mealWithTPerson = {
		...meal,
		person: meal.person
	};

	await mealQueries.insert_meal_transaction(mealWithTPerson, user.registered_user_id);

	await rebuildCache();
	send({ res });
};
