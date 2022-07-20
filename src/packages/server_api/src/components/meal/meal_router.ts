import { meal_delete, meal_get, meal_patch, meal_post, missingMeals_get } from './meal_controller';
import { Router } from 'express';
import { wrap } from '../../lib/wrap';
import { RoutesMeal } from '../../types/enum_routes';

const meal_router = Router({ mergeParams: true });

// Add, edit, delete meals
meal_router.route(RoutesMeal.BASE)
	// Get all meal data, ids in place of categories
	.delete(wrap(meal_delete))
	// edit meal
	.patch(wrap(meal_patch))
	// add new meal
	.post(wrap(meal_post));

// Add, edit, delete meals
meal_router.route(RoutesMeal.BASE_PARAM_DATE_PARAM_PERSON)
	// Get data on specific meal, for editing
	.get(wrap(meal_get));

// Search for any missing meals
meal_router.route(RoutesMeal.MISSING)
	// Get list of any missing meals
	.get(wrap(missingMeals_get));

export { meal_router };