import { TestHelper } from '../testHelper';
import format from 'pg-format';
import { afterAll, beforeAll, beforeEach, describe, expect, it } from 'vitest';

const testHelper = new TestHelper();

const url_adminMealbase = `/admin/meal`;
const url_missing = `${url_adminMealbase}/missing`;
const url_foodBase = `/authenticated/food`;
const url_last = `${url_foodBase}/last`;
const url_category = `${url_foodBase}/category`;
const url_all = `${url_foodBase}/all`;

const initCache = async (): Promise<void> => {
	await Promise.all([
		testHelper.axios.get(url_last),
		testHelper.axios.get(url_category),
		testHelper.axios.get(url_all)
	]);
};

describe('Incognito test runner', () => {
	
	beforeAll(async () => testHelper.beforeAll());
	
	beforeEach(async () => testHelper.beforeEach());
	
	afterAll(async () => testHelper.afterAll());

	describe(`ROUTE - ${url_adminMealbase}`, () => {

		beforeEach(async () => testHelper.beforeEach());

		it('GET - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.get(url_adminMealbase);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
				
		it('DELETE - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.delete(url_adminMealbase);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
				
		it('POST - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.post(url_adminMealbase);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
				
		it('PATCH - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.patch(url_adminMealbase);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
		
		it('PUT - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.put(url_adminMealbase);
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
				await testHelper.axios.get(url_adminMealbase);
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
				await testHelper.axios.delete(url_adminMealbase);
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
				await testHelper.axios.post(url_adminMealbase);
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
				await testHelper.axios.patch(url_adminMealbase);
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
				await testHelper.axios.put(url_adminMealbase);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});

		it('DELETE responds with a 400 invalid user data when invalid body - id boolean', async () => {
			expect.assertions(2);
			try {
				await testHelper.insertAdminUser();
				await testHelper.request_signin();
				await testHelper.axios.delete(url_adminMealbase, { data: { id: testHelper.randomBoolean(), password: await testHelper.randomHex(12), token: null, twoFABackup: false } });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse('ID required'));
			}
		});

		it('DELETE responds with a 400 invalid user data when invalid body - id number', async () => {
			expect.assertions(2);
			try {
				await testHelper.insertAdminUser();
				await testHelper.request_signin();
				await testHelper.axios.delete(url_adminMealbase, { data: { id: testHelper.randomNumber(), password: await testHelper.randomHex(12), token: null, twoFABackup: false } });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse('ID required'));
			}
		});

		it('DELETE responds with a 401 when password invalid', async () => {
			expect.assertions(2);
			try {
				await testHelper.insertAdminUser();
				await testHelper.request_signin();
				await testHelper.axios.delete(url_adminMealbase, { data: { id: testHelper.knownResponse.meal.id, password: await testHelper.randomHex(12), token: null, twoFABackup: false } });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(401);
				expect(e.response?.data).toStrictEqual(testHelper.response_incorrectPasswordOrToken);
			}
		});

		it('DELETE responds with a 401 when token needed but invalid', async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			await testHelper.insert2FAAlwaysRequired();
			if (!testHelper.two_fa_secret) throw Error('!testHelper.two_fa_secret');
			const token = testHelper.generateTokenFromString(testHelper.two_fa_secret);
			const wrongToken = testHelper.generateIncorrectToken(token);
			try {
				await testHelper.axios.delete(url_adminMealbase, { data: { id: testHelper.knownResponse.meal.id, password: testHelper.password, token: wrongToken, twoFABackup: false } });

			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(401);
				expect(e.response?.data).toStrictEqual(testHelper.response_incorrectPasswordOrToken);
			}
		});

		it('DELETE responds with a 401 when password invalid', async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			await testHelper.insert2FAAlwaysRequired();
			if (!testHelper.two_fa_secret) throw Error('!testHelper.two_fa_secret');
			const token = testHelper.generateTokenFromString(testHelper.two_fa_secret);
			try {
				await testHelper.axios.delete(url_adminMealbase, { data: { id: testHelper.knownResponse.meal.id, password: await testHelper.randomHex(11), token, twoFABackup: false } });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(401);
				expect(e.response?.data).toStrictEqual(testHelper.response_incorrectPasswordOrToken);
			}
		});

		it('DELETE responds with a 401 when backuptoken invalid', async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			await testHelper.insert2FAAlwaysRequired();
			const token = await testHelper.randomHex(16);
			try {
				await testHelper.axios.delete(url_adminMealbase, { data: { id: testHelper.knownResponse.meal.id, password: testHelper.password, token, twoFABackup: false } });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(401);
				expect(e.response?.data).toStrictEqual(testHelper.response_incorrectPasswordOrToken);
			}
		});

		it('DELETE responds with a 200 empty response, postgres entries removed, redis cache updated accordingly', async () => {
			expect.assertions(11);
			try {
				await testHelper.pgDump();
				await testHelper.insertAdminUser();
				await testHelper.request_signin();
				await initCache();
				const [ pre_redis_cache_allCategory, pre_redis_cache_allMeal, pre_redis_cache_lastId, ] = await testHelper.getRedisCache();
				if (!pre_redis_cache_allMeal || ! pre_redis_cache_lastId || !pre_redis_cache_allCategory) throw Error('!redis cache');
				const result01 = await testHelper.axios.delete(url_adminMealbase, { data: { id: testHelper.knownResponse.meal.id, password: testHelper.password, token: null, twoFABackup: false } });
				const results = await Promise.all([
					testHelper.postgres.query(format(`SELECT * FROM meal_description md WHERE description = %1$L`, testHelper.knownResponse.meal.description)),
					testHelper.postgres.query(format(`SELECT * FROM individual_meal WHERE individual_meal_id = %1$L`, testHelper.knownResponse.meal.id)),
					testHelper.postgres.query(format(`SELECT * FROM meal_photo mp WHERE photo_original = %1$L`, testHelper.knownResponse.meal.photoNameOriginal)),
					testHelper.postgres.query(format(`SELECT * FROM meal_photo mp WHERE photo_converted = %1$L`, testHelper.knownResponse.meal.photoNameConverted))
				]);
				for (const i of results) expect(i.rows).toEqual([]);
				const result = await testHelper.axios.get(`${url_adminMealbase}/${testHelper.knownResponse.meal.date}/${testHelper.knownResponse.meal.person}`);
				const [ post_redis_cache_allCategory, post_redis_cache_allMeal, post_redis_cache_lastId ] = await testHelper.getRedisCache();
				if (!post_redis_cache_allMeal || !post_redis_cache_lastId || !post_redis_cache_allCategory) throw Error('!redis cache');
				expect(result01.data).toEqual(testHelper.response_empty);
				expect(result01.status).toEqual(200);
				expect(result.status).toEqual(200);
				expect(result.data).toEqual(testHelper.response_empty);
				expect(pre_redis_cache_lastId).not.toEqual(post_redis_cache_lastId);
				expect(pre_redis_cache_allMeal).not.toEqual(post_redis_cache_allMeal);
				expect(pre_redis_cache_allCategory).not.toEqual(post_redis_cache_allCategory);
			} catch (e) {
				// eslint-disable-next-line no-console
				console.log(e);
			} finally {
				await testHelper.pgRestore();
			}
		});

		it('PATCH responds with a 400 invalid user data - date & id mismatch', async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			try {
				await testHelper.axios.patch(url_adminMealbase, { originalDate: testHelper.generateToday(), meal: { id: testHelper.knownResponse.meal.id } });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse('date invalid'));
			}
		});

		it('PATCH responds with a 400 invalid user data - meal object not sent', async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			try {
				await testHelper.axios.patch(url_adminMealbase, { originalDate: testHelper.knownResponse.meal.date, meal: { id: testHelper.knownResponse.meal.id } });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse('date invalid'));
			}
		});

		it('PATCH responds with a 400 invalid user data - no edits made', async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			try {
				await testHelper.axios.patch(url_adminMealbase, { originalDate: testHelper.knownResponse.meal.date, meal: testHelper.knownResponse.meal });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual({ response: 'No edits made' });
			}
		});

		it(`PATCH error when id doesn't match original date`, async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			try {
				// eslint-disable-next-line max-len
				await testHelper.axios.patch(url_adminMealbase, { originalDate: testHelper.knownResponse.meal.date, meal: { ...testHelper.knownResponse.meal, category: await testHelper.randomHex(10), description: await testHelper.randomHex(10), id: String(Number(testHelper.knownResponse.meal.id) +1) } });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual({ response: 'Unknown meal' });
			}
		});

		it(`PATCH responds with a 400 invalid user data - no new date provided`, async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			try {
				await testHelper.axios.patch(url_adminMealbase, { originalDate: testHelper.knownResponse.meal.date, meal: { category: await testHelper.randomHex(10), description: await testHelper.randomHex(10), id: testHelper.knownResponse.meal.id } });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse('date invalid'));
			}
		});

		it(`PATCH responds with a 400 invalid user data - category null`, async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			try {
				await testHelper.axios.patch(url_adminMealbase, { originalDate: testHelper.knownResponse.meal.date, meal: { ...testHelper.knownResponse.meal, category: null } });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse('category'));
			}
		});

		it(`PATCH responds with a 400 invalid user data - description null`, async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			try {
				await testHelper.axios.patch(url_adminMealbase, { originalDate: testHelper.knownResponse.meal.date, meal: { ...testHelper.knownResponse.meal, description: null } });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse('description'));
			}
		});

		it('PATCH responds with a 200 on valid patch, redis and postgres updated', async () => {
			expect.assertions(12);
			try {
				await testHelper.pgDump();
				await testHelper.insertAdminUser();
				await testHelper.request_signin();
				await initCache();
				const [ pre_redis_cache_allCategory, pre_redis_cache_allMeal, pre_redis_cache_lastId, ] = await testHelper.getRedisCache();
				if (!pre_redis_cache_allMeal || ! pre_redis_cache_lastId || !pre_redis_cache_allCategory) throw Error('!redis cache');
				const randomText = await testHelper.randomHex(10);
				const patch_request = await testHelper.axios.patch(url_adminMealbase, { originalDate: testHelper.knownResponse.meal.date, meal: { ...testHelper.knownResponse.meal, category: randomText, description: randomText } });
				const postgres_results = await Promise.all([
					testHelper.postgres.query(format(`SELECT * FROM individual_meal im JOIN meal_description md ON im.meal_description_id = md.meal_description_id WHERE md.description = %1$L`, randomText)),
					testHelper.postgres.query(format(`SELECT * FROM meal_category mc WHERE category = %1$L`, randomText.toUpperCase())),
				]);
				const description_removed = await testHelper.postgres.query(format(format(`SELECT * FROM individual_meal im JOIN meal_description md ON im.meal_description_id = md.meal_description_id WHERE md.description = %1$L`, testHelper.knownResponse.meal.description)));
				const updated_response = await testHelper.axios.get(`${url_adminMealbase}/${testHelper.knownResponse.meal.date}/${testHelper.knownResponse.meal.person}`);
				const [ post_redis_cache_allCategory, post_redis_cache_allMeal, post_redis_cache_lastId, ] = await testHelper.getRedisCache();
				if (!post_redis_cache_allMeal || !post_redis_cache_lastId || !post_redis_cache_allCategory) throw Error('!redis cache');
				expect(patch_request.data).toEqual(testHelper.response_empty);
				expect(patch_request.status).toEqual(200);
				expect(description_removed.rows).toEqual([]);
				expect(updated_response.status).toEqual(200);
				expect(updated_response.data.response).toBeDefined();
				expect(updated_response.data.response.meal.description).toEqual(randomText);
				expect(updated_response.data.response.meal.category).toEqual(randomText.toUpperCase());
				for (const i of postgres_results) expect(i.rows[0]).toHaveProperty('timestamp');
				expect(pre_redis_cache_lastId).not.toEqual(post_redis_cache_lastId);
				expect(pre_redis_cache_allMeal).not.toEqual(post_redis_cache_allMeal);
				expect(pre_redis_cache_allCategory).not.toEqual(post_redis_cache_allCategory);
			} catch (e) {
				// eslint-disable-next-line no-console
				console.log(e);
			} finally {
				await testHelper.pgRestore();
			}
		});
		
		it('POST responds with a 400 invalid user data - meal on date already exists', async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			try {
				await testHelper.axios.post(url_adminMealbase, { originalDate: testHelper.knownResponse.meal.date, meal: { ...testHelper.knownResponse.meal } });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse('meal.id'));
			}
		});

		it(`POST 200 empty response, full post/patch/delete flow, postgres entries inserted then removed`, async () => {
			expect.assertions(37);
			try {
				await testHelper.pgDump();
				await testHelper.insertAdminUser();
				await testHelper.request_signin();
				await initCache();
				const person = testHelper.randomPerson();
				const [ pre_redis_cache_allCategory, pre_redis_cache_allMeal, pre_redis_cache_lastId, ] = await testHelper.getRedisCache();
				if (!pre_redis_cache_allMeal || ! pre_redis_cache_lastId || !pre_redis_cache_allCategory) throw Error('!redis cache');
				const tomorrow = testHelper.generateTomorrow();
				const randomText = await testHelper.randomHex(10);
				const result = await testHelper.axios.post(url_adminMealbase, { meal: {
					person,
					date: tomorrow,
					category: randomText,
					description: randomText,
					takeaway: false,
					restaurant: false,
					vegetarian: false
				} });
				const [ post_redis_cache_allCategory, post_redis_cache_allMeal, post_redis_cache_lastId ] = await testHelper.getRedisCache();
				if (!post_redis_cache_allMeal || !post_redis_cache_lastId || !post_redis_cache_allCategory) throw Error('!redis cache');
				const get_request = await testHelper.axios.get(`${url_adminMealbase}/${tomorrow}/${person}`);
				const post_postgres_results = await Promise.all([
					testHelper.postgres.query(format(`SELECT * FROM individual_meal im JOIN meal_description md ON im.meal_description_id = md.meal_description_id WHERE md.description = %1$L`, randomText)),
					testHelper.postgres.query(format(`SELECT * FROM meal_category mc WHERE category = %1$L`, randomText.toUpperCase())),
					testHelper.postgres.query(format(`SELECT * FROM meal_description md WHERE description = %1$L`, randomText))
				]);
				const patch_request = await testHelper.axios.patch(url_adminMealbase, { originalDate: tomorrow, meal: { ...testHelper.knownResponse.meal, person, date: tomorrow, category: `${randomText}a`, description: `${randomText}a`, id: get_request.data.response.meal.id } });
				const patch_send_request = await testHelper.axios.get(`${url_adminMealbase}/${tomorrow}/${person}`);
				const patch_postgres_results_empty = await Promise.all([
					testHelper.postgres.query(format(`SELECT * FROM individual_meal im JOIN meal_description md ON im.meal_description_id = md.meal_description_id WHERE md.description = %1$L`, randomText)),
					testHelper.postgres.query(format(`SELECT * FROM meal_category mc WHERE category = %1$L`, randomText.toUpperCase())),
					testHelper.postgres.query(format(`SELECT * FROM meal_description md WHERE description = %1$L`, randomText))
				]);
				const [ patch_redis_cache_allCategory, patch_redis_cache_allMeal, patch_redis_cache_lastId ] = await testHelper.getRedisCache();
				if (!post_redis_cache_allMeal || !post_redis_cache_lastId || !post_redis_cache_allCategory) throw Error('!redis cache');
				const patch_postgres_results = await Promise.all([
					testHelper.postgres.query(format(`SELECT * FROM individual_meal im JOIN meal_description md ON im.meal_description_id = md.meal_description_id WHERE md.description = %1$L`, `${randomText}a`)),
					testHelper.postgres.query(format(`SELECT * FROM meal_category mc WHERE category = %1$L`, `${randomText}a`.toUpperCase())),
					testHelper.postgres.query(format(`SELECT * FROM meal_description md WHERE description = %1$L`, `${randomText}a`))
				]);
				const delete_request = await testHelper.axios.delete(url_adminMealbase, { data: { id: get_request.data.response.meal.id, password: testHelper.password, token: null, twoFABackup: false } });
				const delete_postgres_results = await Promise.all([
					testHelper.postgres.query(format(`SELECT * FROM individual_meal im JOIN meal_description md ON im.meal_description_id = md.meal_description_id WHERE md.description = %1$L`, `${randomText}a`)),
					testHelper.postgres.query(format(`SELECT * FROM meal_category mc WHERE category = %1$L`, `${randomText}a`.toUpperCase())),
					testHelper.postgres.query(format(`SELECT * FROM meal_description md WHERE description = %1$L`, `${randomText}a`))
				]);
				const final_get_request = await testHelper.axios.get(`${url_adminMealbase}/${tomorrow}/${person}`);
				const [ delete_redis_cache_allCategory, delete_redis_cache_allMeal, delete_redis_cache_lastId ] = await testHelper.getRedisCache();
				if (!delete_redis_cache_allCategory || !delete_redis_cache_allMeal || !delete_redis_cache_lastId) throw Error('!redis cache');
				for (const i of [ pre_redis_cache_allCategory, pre_redis_cache_allMeal, pre_redis_cache_lastId, ]) expect(i).toBeTruthy();
				expect(result.status).toEqual(200);
				expect(result.data).toEqual(testHelper.response_empty);
				expect(pre_redis_cache_lastId).not.toEqual(post_redis_cache_lastId);
				expect(pre_redis_cache_allMeal).not.toEqual(post_redis_cache_allMeal);
				expect(pre_redis_cache_allCategory).not.toEqual(post_redis_cache_allCategory);
				expect(Number(pre_redis_cache_lastId)+1 === Number(post_redis_cache_lastId)).toBeTruthy();
				expect(get_request.status).toEqual(200);
				expect(get_request.data.response).toBeDefined();
				for (const i of post_postgres_results) expect(i.rows[0].timestamp).toBeDefined();
				expect(patch_request.status).toEqual(200);
				for (const i of patch_postgres_results_empty) expect(i.rows).toEqual([]);
				expect(patch_request.data).toEqual(testHelper.response_empty);
				expect(patch_send_request.data.response.meal.category).toEqual(`${randomText}a`.toUpperCase());
				expect(patch_send_request.data.response.meal.description).toEqual(`${randomText}a`);
				expect(post_redis_cache_lastId).not.toEqual(patch_redis_cache_lastId);
				expect(post_redis_cache_allMeal).not.toEqual(patch_redis_cache_allMeal);
				expect(post_redis_cache_allCategory).not.toEqual(patch_redis_cache_allCategory);
				for (const i of patch_postgres_results) expect(i.rows[0].timestamp).toBeDefined();
				expect(delete_request.status).toEqual(200);
				expect(delete_request.data).toEqual(testHelper.response_empty);
				for (const i of delete_postgres_results) expect(i.rows).toEqual([]);
				expect(final_get_request.status).toEqual(200);
				expect(final_get_request.data).toEqual(testHelper.response_empty);
				expect(patch_redis_cache_lastId).not.toEqual(delete_redis_cache_lastId);
				expect(patch_redis_cache_allMeal).not.toEqual(delete_redis_cache_allMeal);
				expect(patch_redis_cache_allCategory).not.toEqual(delete_redis_cache_allCategory);
			} catch (e) {
				// eslint-disable-next-line no-console
				console.log(e);
			} finally {
				await testHelper.pgRestore();
			}
		});

	});

	describe(`ROUTE - ${url_adminMealbase}/:date/:person`, () => {

		beforeEach(async () => testHelper.beforeEach());

		const url_today_randomPerson = () : string => `${url_adminMealbase}/${testHelper.generateToday()}/${testHelper.randomPerson()}`;

		it('GET - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.get(url_today_randomPerson());
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
				
		it('DELETE - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.delete(url_today_randomPerson());
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
				
		it('POST - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.post(url_today_randomPerson());
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
				
		it('PATCH - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.patch(url_today_randomPerson());
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
		
		it('PUT - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.put(url_today_randomPerson());
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
				await testHelper.axios.get(url_today_randomPerson());
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
				await testHelper.axios.delete(url_today_randomPerson());
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
				await testHelper.axios.post(url_today_randomPerson());
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
				await testHelper.axios.patch(url_today_randomPerson());
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
				await testHelper.axios.put(url_today_randomPerson());
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});

		it('GET responds with a 400 invalid user data params incorrect - date invalid', async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			try {
				await testHelper.axios.get(`${url_adminMealbase}/2020-13-01/${testHelper.randomPerson()}`);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse('date invalid'));
			}
		});

		it('GET responds with a 400 invalid user data params incorrect - person lowercase', async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			try {
				await testHelper.axios.get(`${url_adminMealbase}/${testHelper.generateToday()}/${testHelper.randomPerson().toLowerCase()}`);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse('person unrecognised'));
			}
		});

		it('GET responds with a 200 valid meal objects for random selection of days', async () => {
			expect.assertions(176);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			const urls = new Array(8);
			for (const [ index, _item ] of urls.entries()) urls[index] = `${testHelper.randomDate()}/${testHelper.randomPerson()}`;
			const promiseArray = [];
			for (const i of urls) promiseArray.push(testHelper.axios.get(`${url_adminMealbase}/${i}`));
			const results = await Promise.all(promiseArray);
			// 22 expects
			for (const [ index, i ] of results.entries()) {
				expect(i.status).toBe(200);
				expect(i.data.response.meal).toBeDefined();
				expect(i.data.response.meal).toHaveProperty('id');
				expect(isNaN(i.data.response.meal.id)).toBeFalsy();
				expect(i.data.response.meal).toHaveProperty('date');
				expect(i.data.response.meal.date).toEqual(urls[index].split('/')[0]);
				expect(i.data.response.meal).toHaveProperty('person');
				expect(i.data.response.meal.person).toEqual(urls[index].split('/')[1]);
				expect(i.data.response.meal).toHaveProperty('category');
				expect(typeof i.data.response.meal.category === 'string').toBeTruthy();
				expect(i.data.response.meal).toHaveProperty('restaurant');
				expect(i.data.response.meal.restaurant === null || typeof i.data.response.meal.restaurant === 'boolean').toBeTruthy();
				expect(i.data.response.meal).toHaveProperty('takeaway');
				expect(i.data.response.meal.takeaway === null || typeof i.data.response.meal.takeaway === 'boolean').toBeTruthy();
				expect(i.data.response.meal).toHaveProperty('vegetarian');
				expect(i.data.response.meal.takeaway === null || typeof i.data.response.meal.takeaway === 'boolean').toBeTruthy();
				expect(i.data.response.meal).toHaveProperty('meal_photo_id');
				expect(i.data.response.meal.meal_photo_id === null || !isNaN(i.data.response.meal.meal_photo_id)).toBeTruthy();
				expect(i.data.response.meal).toHaveProperty('photoNameOriginal');
				expect(i.data.response.meal.photoNameOriginal === null || testHelper.regex_photoOriginal.test(i.data.response.meal.photoNameOriginal)).toBeTruthy();
				expect(i.data.response.meal).toHaveProperty('photoNameConverted');
				expect(i.data.response.meal.photoNameConverted === null || testHelper.regex_photoConverted.test(i.data.response.meal.photoNameConverted)).toBeTruthy();
			}
		});
		
		it('GET responds with a 200 valid meal object matching known data for that specific day/person', async () => {
			expect.assertions(3);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			const result = await testHelper.axios.get(`${url_adminMealbase}/${testHelper.knownResponse.meal.date}/${testHelper.knownResponse.meal.person}`);
			expect(result.status).toBe(200);
			expect(result.data.response).toBeDefined();
			expect(result.data.response).toEqual(testHelper.knownResponse);
		});
		
		it('GET responds with a 200 empty response for a date without a meal', async () => {
			expect.assertions(2);
			const tomorrow = testHelper.generateTomorrow();
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			const result = await testHelper.axios.get(`${url_adminMealbase}/${tomorrow}/${testHelper.knownResponse.meal.person}`);
			expect(result.status).toBe(200);
			expect(result.data).toEqual(testHelper.response_empty);
		});

	});
	
	describe(`ROUTE - ${url_missing}`, () => {

		beforeEach(async () => testHelper.beforeEach());
		it('GET - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.get(url_missing);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
				
		it('DELETE - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.delete(url_missing);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
				
		it('POST - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.post(url_missing);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
				
		it('PATCH - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.patch(url_missing);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
		
		it('PUT - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.put(url_missing);
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
				await testHelper.axios.get(url_missing);
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
				await testHelper.axios.delete(url_missing);
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
				await testHelper.axios.post(url_missing);
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
				await testHelper.axios.patch(url_missing);
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
				await testHelper.axios.put(url_missing);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});

		it('GET responds 200, contains missingMeals key', async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			const result = await testHelper.axios.get(url_missing);
			expect(result.data.response).toBeDefined();
			expect(result.data.response).toHaveProperty('missingMeals');
		});

		it('GET responds 200, contains array of known missing meals, dates match', async () => {
			expect.assertions(9);
			try {
				await testHelper.pgDump();
				await testHelper.insertAdminUser();
				await testHelper.request_signin();
				const delete_request = await testHelper.axios.delete(url_adminMealbase, { data: { id: testHelper.knownResponse.meal.id, password: testHelper.password, token: null, twoFABackup: false } });
				const result = await testHelper.axios.get(url_missing);
				// eslint-disable-next-line @typescript-eslint/no-explicit-any
				const knownIndex = result.data.response.missingMeals.findIndex((i: any) => i.missing_date.substring(0, 10) === testHelper.knownResponse.meal.date);
				expect(result.data.response).toBeDefined();
				expect(result.data.response.missingMeals[0]).toHaveProperty('missing_date');
				expect(result.data.response.missingMeals[0]).toHaveProperty('person');
				expect(delete_request.status).toEqual(200);
				expect(delete_request.data).toEqual(testHelper.response_empty);
				expect(result.status).toEqual(200);
				expect(result.data.response.missingMeals.length).toBeGreaterThanOrEqual(1);
				expect(knownIndex).toBeGreaterThanOrEqual(0);
				expect(result.data.response.missingMeals[knownIndex].person).toEqual(testHelper.knownResponse.meal.person);
			} catch (e) {
				// eslint-disable-next-line no-console
				console.log(e);
			} finally {
				await testHelper.pgRestore();
				
			}
		});

	});

});
