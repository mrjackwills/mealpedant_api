import { customError } from '../config/customError';
import { customTypeError } from '../config/customError';
import { ErrorMessages } from '../types/enum_error';
import { HttpCode } from '../types/enum_httpCode';
import { parse } from 'secure-json-parse';
import { Redis } from '../config/db_redis';
import { RedisKey } from '../types/enum_redis';
import { TNewUser, UserId, TAllMealVue, lastId, TCategories } from '../types';
import { isId } from '../types/userGuard';
import ioredis from 'ioredis';

class Base {
	constructor (protected _db: ioredis.Redis) {

	}

	async exists (key: string): Promise<boolean> {
		const data = await this._db.exists(key);
		return data > 0;
	}

	async get (key: string): Promise<string|null> {
		const data = await this._db.get(key);
		return data;
	}
	
}

class Verify_Queries extends Base {

	async verifyEmail_exists (email: string): Promise<boolean> {
		const result = await this.exists(`${RedisKey.VERIFY_EMAIL}${email}`);
		return result;
	}

	async verifyUser_set (newUser: TNewUser, verifyString: string): Promise<void> {
		const expireTime = 60*60*6;
			
		await this._db.hset(`${RedisKey.VERIFY_STRING}${verifyString}`, `data`, JSON.stringify(newUser));
		await this._db.expire(`${RedisKey.VERIFY_STRING}${verifyString}`, expireTime);
		await this._db.set(`${RedisKey.VERIFY_EMAIL}${newUser.email}`, verifyString, 'ex', expireTime);
	}

	async verifyUser_get (verifyString: string): Promise<TNewUser> {
		const key = `${RedisKey.VERIFY_STRING}${verifyString}`;
		const newUser = await this.exists(key);

		if (!newUser) throw customError(HttpCode.BAD_REQUEST, ErrorMessages.VERIFICATION_INCORRECT);

		const newUserData = await this._db.hget(key, 'data');
		if (!newUserData) throw customError(HttpCode.BAD_REQUEST, ErrorMessages.VERIFICATION_INCORRECT);

		const user: TNewUser = parse(newUserData, undefined, { protoAction: 'remove', constructorAction: 'remove' });
		return user;
	}

	async verifyUser_delete (newUser: TNewUser, verifyString: string) :Promise<void> {

		await Promise.all([
			this._db.del(`${RedisKey.VERIFY_EMAIL}${newUser.email}`),
			this._db.del(`${RedisKey.VERIFY_STRING}${verifyString}`)
		]);
	}

}

class TwoFA_Queries extends Verify_Queries {

	async setup_delete (userId: UserId): Promise<void> {
		await this._db.del(`${RedisKey.TWO_FA_SETUP}${userId}`);
	}

	async setupToken_get (userId: UserId): Promise<string|null> {
		const tokenInRedis = await this.get(`${RedisKey.TWO_FA_SETUP}${userId}`);
		return tokenInRedis;
	}

	async setupToken_set (userId: UserId, secret: string): Promise<void> {
		await this._db.set(`${RedisKey.TWO_FA_SETUP}${userId}`, secret, 'ex', 90);
	}

	async secret_get (userId: UserId): Promise<string|null> {
		const secret = await this.get(`${RedisKey.TWO_FA_SETUP}${userId}`);
		return secret;
	}

	async secret_delete (userId: UserId): Promise<void> {
		await this._db.del(`${RedisKey.TWO_FA_SETUP}${userId}`);
	}
}

class User_Queries extends TwoFA_Queries {

	#userSetNameUserId (userId: UserId) :string {
		return `${RedisKey.SESSION_SET}${userId}`;
	}

	#userSetNameSessionId (sessionId: string) :string {
		return `${RedisKey.SESSION}${sessionId}`;
	}
	
	async userSet_get (userId: UserId): Promise<Array<string>> {
		const userSet = await this._db.smembers(this.#userSetNameUserId(userId));
		return userSet;
	}

	async session_get (sessionName: string): Promise<string|null> {
		const session = await this.get(sessionName);
		return session;
	}

	async sessionSet_remove (userId: UserId, sessionName: string): Promise<void> {
		await this._db.srem(this.#userSetNameUserId(userId), sessionName);
	}

	async session_remove (userId: UserId, sessionId: string): Promise<void> {
		await this.sessionSet_remove(userId, this.#userSetNameSessionId(sessionId));
	}
	
	async session_add (userId: UserId, sessionId: string) :Promise<void> {
		await this._db.sadd(`${RedisKey.SESSION_SET}${userId}`, this.#userSetNameSessionId(sessionId));
	}
}

class Limiter_Queries extends User_Queries {

	async limiter_add (key:string) :Promise<void> {
		await this._db.sadd(RedisKey.LIMITER_SET, `${RedisKey.LIMITER}${key}`);
	}

}

class Food_Queries extends Limiter_Queries {
	
	async delete_food_cache (): Promise<void> {
		await Promise.all([
			this._db.del(RedisKey.CACHE_ALL_CATEGORY),
			this._db.del(RedisKey.CACHE_ALL_MEAL),
			this._db.del(RedisKey.CACHE_LAST_ID),
		]);
	}

	async insert_allMeal_cache (allMealData: Array<TAllMealVue>): Promise<void> {
		if (!allMealData) throw customTypeError('insert_allMeal_cache(): !allMealData');
		await this._db.hset(RedisKey.CACHE_ALL_MEAL, 'data', JSON.stringify(allMealData));
	}
	
	async insert_allCategory_cache (allCategoryData: Array<TCategories>): Promise<void> {
		if (!allCategoryData) throw customTypeError('insert_allCategory_cache(): !allCategoryData');
		await this._db.hset(RedisKey.CACHE_ALL_CATEGORY, 'data', JSON.stringify(allCategoryData));
	}
	
	async insert_lastId_cache (lastId: lastId): Promise<void> {
		if (!lastId) throw customTypeError('insert_lastId_cache: !lastMealEditId');
		await this._db.set(RedisKey.CACHE_LAST_ID, lastId);
	}
	
	async select_allCategory_cache (): Promise<Array<TCategories>|undefined> {
		const data = await this._db.hget(RedisKey.CACHE_ALL_CATEGORY, 'data');
		return data ? parse(data) : undefined;
	}

	async select_allMeal_cache (): Promise<Array<TAllMealVue>|undefined> {
		const data = await this._db.hget(RedisKey.CACHE_ALL_MEAL, 'data');
		return data ? parse(data) : undefined;
	}

	async select_lastId_cache (): Promise<lastId|undefined> {
		const data = await this.get(RedisKey.CACHE_LAST_ID);
		return isId<lastId>(data) ? data : undefined;
	}

}

class Admin_Queries extends Food_Queries {

	async admin_get_session (sessionName: string) : Promise<string|null> {
		const session = await this.get(sessionName);
		return session;
	}
	async admin_remove_session (sessionName: string, userId: string): Promise<void> {
		
		await Promise.all([
			this._db.del(sessionName),
			this._db.srem(`${RedisKey.SESSION_SET}${userId}`, sessionName),
		]);
	}

	async admin_get_limited_clients (): Promise<Array<string>> {
		const limtedClients = await this._db.smembers(RedisKey.LIMITER_SET);
		return limtedClients;
	}

	async admin_remove_from_limiter_set (key: string): Promise<void> {
		await this._db.srem(RedisKey.LIMITER_SET, `${key}`);
	}

	async admin_get_setmembers (userSetName: string): Promise<Array<string>> {
		const data = await this._db.smembers(userSetName);
		return data;
	}

	async admin_remove_from_session_set (userSetName: string, sessionKey: string): Promise<void> {
		await this._db.srem(userSetName, sessionKey);
	}

	async admin_remove_active_user_sessions (userId: UserId): Promise<void> {
		const sessions = await this._db.smembers(`${RedisKey.SESSION_SET}${userId}`);
		if (sessions?.length>0) await this._db.del(sessions);
		await this._db.del(`${RedisKey.LIMITER_SET}${userId}`);
	}
}

export const redisQueries = new Admin_Queries(Redis);