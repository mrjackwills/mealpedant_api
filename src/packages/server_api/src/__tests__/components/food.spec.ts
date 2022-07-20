import { TestHelper } from '../testHelper';

const testHelper = new TestHelper();
const url_base = `/authenticated/food`;
const url_last = `${url_base}/last`;
const url_category = `${url_base}/category`;
const url_all = `${url_base}/all`;
const url_cache = `${url_base}/cache`;

import { afterAll, beforeAll, beforeEach, describe, expect, it } from 'vitest';

describe('Food test runner', () => {
	beforeAll(async () => testHelper.beforeAll());

	afterAll(async () => testHelper.afterAll());

	describe(`ROUTE - ${url_last}`, () => {

		beforeEach(async () => testHelper.beforeEach());

		it('GET - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.get(url_last);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
		
		it('DELETE - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.delete(url_last);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
		
		it('POST - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.post(url_last);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
		
		it('PATCH - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				
				await testHelper.axios.patch(url_last);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});

		it('PUT - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				
				await testHelper.axios.put(url_last);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});

		it('GET - should return id of last id of meal update/insert, and be greater than 4000', async () => {
			expect.assertions(4);
			await testHelper.insertUser();
			await testHelper.request_signin();
			const result = await testHelper.axios.get(url_last);
			expect(result.data.response).toBeDefined();
			expect(result.data.response.lastId).toBeDefined();
			expect(isNaN(result.data.response.lastId)).toBeFalsy();
			expect(Number(result.data.response.lastId)).toBeGreaterThan(4000);
		});

		it(`GET - should return last id after cache has been removed`, async () => {
			expect.assertions(3);
			await testHelper.insertUser();
			await testHelper.request_signin();
			const result01 = await testHelper.axios.get(url_last);
			await testHelper.redis.del(`cache:lastMealEditId`);
			const result02 = await testHelper.axios.get(url_last);
			expect(result01.data.response).toBeDefined();
			expect(result02.data.response).toBeDefined();
			expect(result01.data).toEqual(result02.data);
		});

	});

	describe(`ROUTE - ${url_category}`, () => {

		beforeEach(async () => testHelper.beforeEach());

		it('GET - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.get(url_category);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
			
		it('DELETE - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.delete(url_category);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
			
		it('POST - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.post(url_category);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
			
		it('PATCH - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
					
				await testHelper.axios.patch(url_category);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
	
		it('PUT - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
					
				await testHelper.axios.put(url_category);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});

		it('GET - should return list of array of objects of categories, each with an id, c, and n, key ', async () => {
			expect.assertions(6);
			await testHelper.insertUser();
			await testHelper.request_signin();
			const result = await testHelper.axios.get(url_category);
			const randomCategory = result.data.response[Math.floor(Math.random() * result.data.response.length)];
			expect(result.status).toEqual(200);
			expect(result.data.response).toBeDefined();
			expect(result.data.response.length > 1).toBeTruthy();
			expect(randomCategory).toHaveProperty('id');
			expect(randomCategory).toHaveProperty('c');
			expect(randomCategory).toHaveProperty('n');
		});

		it(`GET - should return allCategories after cache has been removed`, async () => {
			expect.assertions(3);
			await testHelper.insertUser();
			await testHelper.request_signin();
			const result01 = await testHelper.axios.get(url_category);
			await testHelper.redis.del(`cache:allCategory`);
			const result02 = await testHelper.axios.get(url_category);
			expect(result01.data.response).toBeDefined();
			expect(result02.data.response).toBeDefined();
			expect(result01.data).toEqual(result02.data);
		});
		
	});

	describe(`ROUTE - ${url_all}`, () => {

		beforeEach(async () => testHelper.beforeEach());
		
		it('GET - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.get(url_all);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
			
		it('DELETE - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.delete(url_all);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
			
		it('POST - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.post(url_all);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
			
		it('PATCH - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
					
				await testHelper.axios.patch(url_all);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
	
		it('PUT - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
					
				await testHelper.axios.put(url_all);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});

		it('GET - should return list of array of objects of meals, random one should have a d matching yyyymmdd, J, D, with J or D containting md and c ', async () => {
			expect.assertions(8);
			await testHelper.insertUser();
			await testHelper.request_signin();
			const result = await testHelper.axios.get(url_all);
			expect(result.status).toEqual(200);
			expect(result.data.response).toBeDefined();
			expect(result.data.response.length > 1).toBeTruthy();
			const randomDay = result.data.response[Math.floor(Math.random() * result.data.response.length)];
			const randomPerson = testHelper.randomPersonInitial();
			expect(randomDay).toHaveProperty('ds');
			expect(randomDay).toHaveProperty('J');
			expect(randomDay).toHaveProperty('D');
			expect(randomDay[randomPerson]).toHaveProperty('md');
			expect(randomDay[randomPerson]).toHaveProperty('c');
		});

		it(`GET - should return allMeals after cache has been removed`, async () => {
			expect.assertions(3);
			await testHelper.insertUser();
			await testHelper.request_signin();
			const result01 = await testHelper.axios.get(url_all);
			await testHelper.redis.del(`cache:allMeal`);
			const result02 = await testHelper.axios.get(url_all);
			expect(result01.data.response).toBeDefined();
			expect(result02.data.response).toBeDefined();
			expect(result01.data).toEqual(result02.data);
		});
	
	});
	
	describe(`ROUTE - ${url_cache}`, () => {

		beforeEach(async () => testHelper.beforeEach());
	
		it('GET - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.get(url_cache);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
				
		it('DELETE - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.delete(url_cache);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
				
		it('POST - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.post(url_cache);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
				
		it('PATCH - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
						
				await testHelper.axios.patch(url_cache);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
		
		it('PUT - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
						
				await testHelper.axios.put(url_cache);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});

		it('GET - should return unauthorized 403 when signed in but not admin', async () => {
			expect.assertions(2);
			try {
				await testHelper.insertUser();
				await testHelper.request_signin();
				await testHelper.axios.get(url_cache);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});

		it('DELETE - should return unauthorized 403 when signed in but not admin', async () => {
			expect.assertions(2);
			try {
				await testHelper.insertUser();
				await testHelper.request_signin();
				await testHelper.axios.delete(url_cache);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
		
		it('POST - should return unauthorized 403 when signed in but not admin', async () => {
			expect.assertions(2);
			try {
				await testHelper.insertUser();
				await testHelper.request_signin();
				await testHelper.axios.post(url_cache);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
		
		it('PATCH - should return unauthorized 403 when signed in but not admin', async () => {
			expect.assertions(2);
			try {
				await testHelper.insertUser();
				await testHelper.request_signin();
				await testHelper.axios.patch(url_cache);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
		
		it('PUT - should return unauthorized 403 when signed in but not admin', async () => {
			expect.assertions(2);
			try {
				await testHelper.insertUser();
				await testHelper.request_signin();
				await testHelper.axios.put(url_cache);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});

		it('DELETE - expect return empty body 200 status', async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			const result = await testHelper.axios.delete(url_cache);
			expect(result.status).toEqual(200);
			expect(result.data).toEqual(testHelper.response_empty);
		});

		it('DELETE - expect return 200 status', async () => {
			expect.assertions(10);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			await Promise.all([ testHelper.axios.get(url_category), testHelper.axios.get(url_last), testHelper.axios.get(url_all) ]);
			const [ pre_redis_cache_allCategory, pre_redis_cache_allMeal, pre_redis_cache_lastId ] = await testHelper.getRedisCache();
			const result = await testHelper.axios.delete(url_cache);
			const [ post_redis_cache_allCategory, post_redis_cache_allMeal, post_redis_cache_lastId ] = await testHelper.getRedisCache();
			await testHelper.sleep(2500);
		
			expect(result.status).toEqual(200);
			expect(result.data).toEqual(testHelper.response_empty);
			expect(pre_redis_cache_allCategory).toBeTruthy();
			expect(pre_redis_cache_allMeal).toBeTruthy();
			expect(pre_redis_cache_lastId).toBeTruthy();
			expect(result.status).toEqual(200);
			expect(result.data.response).toBeDefined();

			expect(pre_redis_cache_allCategory === post_redis_cache_allCategory).toBeFalsy();
			expect(pre_redis_cache_allMeal === post_redis_cache_allMeal).toBeFalsy();
			expect(pre_redis_cache_lastId === post_redis_cache_lastId).toBeFalsy();
		});
	
	});

});