import { allMeal_get, cache_delete, category_get, lastId_get, } from './food_controller';
import { RoutesFood } from '../../types/enum_routes';
import { isAdmin } from '../../lib/checkAuthentication';
import { Router } from 'express';
import { wrap } from '../../lib/wrap';

const food_router = Router({ mergeParams: true });

food_router.route(RoutesFood.ALL)
	// Get all meal data, ids in place of categories
	.get(wrap(allMeal_get));

food_router.route(RoutesFood.CATEGORY)
	// Get list of all categories, with id and count
	.get(wrap(category_get));

food_router.route(RoutesFood.LAST)
	// Get id of last meal edit, for syncing client and server data
	// .post(wrap(lastId_get))
	.get(wrap(lastId_get));

// MAYBE move to admin routes
// Actually probable ok here, can have a food/admin route
food_router.route(RoutesFood.CACHE)
	.all(isAdmin)
	// Apply ADMIN ONLY privilege to all routes
	// flush meal data from redis cache
	.delete(wrap(cache_delete));

export { food_router };