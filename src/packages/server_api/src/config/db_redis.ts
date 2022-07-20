import { log } from './log';
import { REDIS_DB, REDIS_HOSTNAME, REDIS_PASS, REDIS_PORT } from '../config/env';
import ioredis, { RedisOptions } from 'ioredis';

const redisOptions: RedisOptions = {
	port: REDIS_PORT,
	password: REDIS_PASS,
	host: REDIS_HOSTNAME,
	family: 4,
	db: REDIS_DB,
};

const Redis = new ioredis(redisOptions);
Redis.on('connect', () => log.debug(`redis connected [${redisOptions.db}] @ redis://${redisOptions.host}:${redisOptions.port}`)) ;
Redis.on('error', (e) => log.error(e, { log: 'redis connection error' }));

export { Redis };