import { customError } from './customError';
import { ErrorMessages } from '../types/enum_error';
import { extractIp } from '../lib/helpers';
import { HttpCode } from '../types/enum_httpCode';
import { RateLimiterRedis, IRateLimiterStoreOptions, RateLimiterRes } from 'rate-limiter-flexible';
import { MODE_ENV_TEST } from './env';
import { Redis } from '../config/db_redis';
import { RequestHandler } from 'express';
import { PV } from '../types';
import { redisQueries } from '../lib/redisQueries';

const redisOpts: IRateLimiterStoreOptions = {
	keyPrefix: 'limiter',
	storeClient: Redis,
	points: 120,
	duration: 60,
	blockDuration: 60,
	execEvenly: false,
};

export const limiter = new RateLimiterRedis(redisOpts);

export const rateLimiter: RequestHandler = async (req, _res, next): PV => {
	try {
		if (MODE_ENV_TEST && !process.env.limitTest) return next();
		const key = req.user ? req.user.email : extractIp(req);

		await redisQueries.limiter_add(key);
		const current_points = await limiter.get(key);
		const points = req.user ? req.user.admin ? 1 : 2 : 4;
		
		if (current_points && redisOpts.points && current_points.consumedPoints+points >= redisOpts.points * 2) {
			/**
			 * Block for 1/4/7 days depending on points total
			 * Also add a penalty, so numbers keep increasing
			 * maybe Destroy session?
			 */
			const oneDay = 60 * 60 * 24;
			if (current_points.consumedPoints >= redisOpts.points * 8 && current_points.consumedPoints <= redisOpts.points * 8 + points) {
				await limiter.block(key, oneDay * 7);
				await limiter.penalty(key, redisOpts.points * 8);
			}
			else if (current_points.consumedPoints >= redisOpts.points * 4 && current_points.consumedPoints <= redisOpts.points * 4 + points) {
				await limiter.block(key, oneDay * 3);
				await limiter.penalty(key, redisOpts.points * 4);
			}
			else if (current_points.consumedPoints >= redisOpts.points * 2 && current_points.consumedPoints <= redisOpts.points * 2 + points) {
				await limiter.block(key, oneDay);
				await limiter.penalty(key, redisOpts.points * 2);
			}
		}
		// Consumer points, aka add to redis user object
		await limiter.consume(key, points);
		next();
	} catch (e) {
		const message = e instanceof RateLimiterRes ? e.msBeforeNext : ErrorMessages.INTERNAL;
		const code = e instanceof RateLimiterRes && e.msBeforeNext? HttpCode.TOO_MANY_REQUESTS : HttpCode.INTERNAL_SERVER_ERROR;
		const error = customError(code, message);
		next(error);
	}
};
