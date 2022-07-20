import { customError } from '../../config/customError';
import { foodQueries } from './food_queries';
import { HttpCode } from '../../types/enum_httpCode';
import { RequestMethod } from '../../types';
import { send } from '../../lib/send';
import { redisQueries } from '../../lib/redisQueries';

export const allMeal_get: RequestMethod = async (_req, res) => {
	const redisCache = await redisQueries.select_allMeal_cache();
	if (redisCache) send({ res, response: redisCache });
	else {
		const postgresData = await foodQueries.select_allMeal();
		await redisQueries.insert_allMeal_cache(postgresData);
		send({ res, response: postgresData });
	}
};

export const cache_delete: RequestMethod = async (_req, res) => {
	await redisQueries.delete_food_cache();
	send({ res });
};

export const category_get: RequestMethod = async (_req, res) => {
	const redisCache = await redisQueries.select_allCategory_cache();
	if (redisCache) send({ res, response: redisCache });
	else {
		const postgresData = await foodQueries.select_allCategory();
		await redisQueries.insert_allCategory_cache(postgresData);
		send({ res, response: postgresData });
	}
};

export const lastId_get: RequestMethod = async (_req, res) => {
	const redisCache = await redisQueries.select_lastId_cache();
	if (redisCache) return send({ res, response: { lastId: redisCache } });
	else {
		const lastId = await foodQueries.select_lastId();
		if (!lastId) throw customError(HttpCode.INTERNAL_SERVER_ERROR);
		await redisQueries.insert_lastId_cache(lastId);
		send({ res, response: { lastId } });
	}
};