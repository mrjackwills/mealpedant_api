import { api_version } from '../../config/api_version';
import { TestHelper } from '../testHelper';
import { RedisKey } from '../../types/enum_redis';

import { afterAll, beforeAll, beforeEach, describe, expect, it } from 'vitest';

const testHelper = new TestHelper();

const url_base = `/incognito`;
const url_online = `${url_base}/online`;
const url_signin = `${url_base}/signin`;
const url_signout = `authenticated/user/signout`;
const url_register = `${url_base}/register`;
const url_resetPassword = `${url_base}/reset-password`;
const url_verify = `${url_base}/verify`;

describe('Incognito test runner', () => {

	const insertUserPreVerify = async ():Promise<void> => {
		await testHelper.axios.post(url_register,
			{ firstName: testHelper.firstName, lastName: testHelper.lastName, password: testHelper.password, email: testHelper.email, invite: testHelper.invite },
			{ headers: { 'User-Agent': testHelper.userAgent } }
		);
	};

	beforeAll(async () => testHelper.beforeAll());

	beforeEach(async () => testHelper.beforeEach());

	afterAll(async () => testHelper.afterAll());

	describe(`ROUTE - ${url_online}`, () => {

		it('DELETE - should return unknown 404', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.delete(url_online);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(404);
				expect(e.response?.data).toStrictEqual(testHelper.response_unknown);
			}
		});
		
		it('POST - should return unknown 404', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.post(url_online);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(404);
				expect(e.response?.data).toStrictEqual(testHelper.response_unknown);
			}
		});
		
		it('PATCH - should return unknown 404', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.patch(url_online);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(404);
				expect(e.response?.data).toStrictEqual(testHelper.response_unknown);
			}
		});
		
		it('PUT - should return unknown 404', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.put(url_online);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(404);
				expect(e.response?.data).toStrictEqual(testHelper.response_unknown);
			}
		});

		it('GET returns 200 api_version & uptime', async () => {
			expect.assertions(2);
			const result = await testHelper.axios.get(url_online);
			expect(result.status).toEqual(200);
			expect(result.data.response.api_version).toStrictEqual(api_version);
		});

	});

	describe(`ROUTE - ${url_register}`, () => {
		beforeEach(async () => testHelper.beforeEach());

		it('DELETE - should return unknown 404', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.delete(url_register);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(404);
				expect(e.response?.data).toStrictEqual(testHelper.response_unknown);
			}
		});
		
		it('GET - should return unknown 404', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.get(url_register);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(404);
				expect(e.response?.data).toStrictEqual(testHelper.response_unknown);
			}
		});
		
		it('PATCH - should return unknown 404', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.patch(url_register);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(404);
				expect(e.response?.data).toStrictEqual(testHelper.response_unknown);
			}
		});
		
		it('PUT - should return unknown 404', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.put(url_register);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(404);
				expect(e.response?.data).toStrictEqual(testHelper.response_unknown);
			}
		});

		it('POST invalid body, statuscode 400, body: invalid user data: firstname', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.post(url_register, { lastName: testHelper.lastName, email: testHelper.email, password: testHelper.password, invite: testHelper.invite });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`A first name is required`));
			}
		});
		
		it('POST invalid body, statuscode 400, body: invalid user data: last name', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.post(url_register, { firstName: testHelper.firstName, email: testHelper.email, password: testHelper.password, invite: testHelper.invite });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`A last name is required`));
			}
		});

		it('POST invalid body, no email, statuscode 400, body: invalid user data: email', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.post(url_register, { firstName: testHelper.firstName, lastName: testHelper.lastName, password: testHelper.password, invite: testHelper.invite });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`email address`));
			}
		});

		it('POST invalid body email joi, statuscode 400, body: invalid user data: email', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.post(url_register, { firstName: testHelper.firstName, lastName: testHelper.lastName, email: testHelper.firstName, password: testHelper.password, invite: testHelper.invite });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`email address`));
			}
		});

		it('POST invalid body email in banned list, statuscode 400, body: invalid user data: email', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.post(url_register, { firstName: testHelper.firstName, lastName: testHelper.lastName, email: `${await testHelper.randomHex(10)}@10minutetempemail.com`, password: testHelper.password, invite: testHelper.invite });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual({ response: 'Invalid email address domain' });
			}
		});
		
		it('POST invalid body, statuscode 400, body: invalid user data: password missing', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.post(url_register, { firstName: testHelper.firstName, lastName: testHelper.lastName, email: testHelper.email, invite: testHelper.invite });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`passwords are required to be 10 characters minimum`));
			}
		});

		it('POST invalid body, statuscode 400, body: invalid user data: password too short', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.post(url_register, { firstName: testHelper.firstName, lastName: testHelper.lastName, password: await testHelper.randomHex(8), email: testHelper.email, invite: testHelper.invite });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`passwords are required to be 10 characters minimum`));
			}
		});

		it('POST invalid body, statuscode 400, body: invalid user data: password hibp', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.post(url_register, { firstName: testHelper.firstName, lastName: testHelper.lastName, password: 'iloveyou1234', email: testHelper.email, invite: testHelper.invite });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual({ response: `The password provided is in a database of compromised passwords and should never be used` });
			}
		});

		it('POST invalid body, statuscode 400, body: invalid user data: invite', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.post(url_register, { firstName: testHelper.firstName, lastName: testHelper.lastName, password: testHelper.password, email: testHelper.email, invite: 'invalid invite' });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`the invite code provided is incorrect`));
			}
		});

		it('POST invalid register body -- already registered user, statuscode 200, body check email, no redis key', async () => {
			expect.assertions(3);
			await testHelper.insertUser();
			const result = await testHelper.axios.post(url_register, { firstName: testHelper.firstName, lastName: testHelper.lastName, password: testHelper.password, email: testHelper.email, invite: testHelper.invite });
			const verifyString = await testHelper.redis.get(`verify:email:${testHelper.email}`);
			expect(result.status).toStrictEqual(200);
			expect(result.data).toStrictEqual({ response: 'Instructions have been sent to the email address provided' });
			expect(verifyString).toBeFalsy();
		});

		it('POST valid register body, response 200 "check email", verifyString in redis, ip and userAgent in db', async () => {
			expect.assertions(16);
			const preCount = testHelper.mockedRabbitSendEmail.mock.calls.length;
			await insertUserPreVerify();
			const verifyString = await testHelper.redis.keys(`${RedisKey.VERIFY_STRING}*`);
			const verifyEmail_exists = await testHelper.redis.exists(`${RedisKey.VERIFY_EMAIL}${testHelper.email}`);
			if (!verifyString || !verifyString[0]) throw Error('!verifyString || verifyString.length !== 1');
			const verifyEmail_ttl = await testHelper.redis.ttl(`${RedisKey.VERIFY_EMAIL}${testHelper.email}`);
			const verifyString_ttl = await testHelper.redis.ttl(verifyString[0]);
			const u = await testHelper.redis.hget(verifyString[0], 'data');
			if (!u) throw new Error('no redis');
			const userAgentExists = await testHelper.query_selectUserAgent();
			const ipExists = await testHelper.query_selectIp();
			const redisUser = JSON.parse(u);
			const postCount = testHelper.mockedRabbitSendEmail.mock.calls.length;
			expect(verifyEmail_ttl).toBeGreaterThan(21590);
			expect(verifyEmail_ttl).toBeLessThan(21650);
			expect(verifyString_ttl).toBeGreaterThan(21590);
			expect(verifyString_ttl).toBeLessThan(21650);
			expect(preCount).toBe(0);
			expect(postCount).toBe(1);
			expect(userAgentExists).toBe(testHelper.userAgent);
			expect(ipExists).toBe(testHelper.axios_ip);
			expect(verifyEmail_exists).toBeTruthy();
			expect(verifyString[0]).toMatch(testHelper.regex_verifyString);
			expect(redisUser.email).toStrictEqual(testHelper.email);
			expect(redisUser.first_name).toStrictEqual(testHelper.firstName);
			expect(redisUser.last_name).toStrictEqual(testHelper.lastName);
			expect(redisUser.password_hash).toBeTruthy();
			expect(redisUser.ipId).toBeTruthy();
			expect(redisUser.userAgentId).toBeTruthy();
		});

		it('POST valid register body, rabbitMq called with correct data', async () => {
			expect.assertions(7);
			await testHelper.axios.post(url_register,
				{ firstName: testHelper.firstName, lastName: testHelper.lastName, password: testHelper.password, email: testHelper.email, invite: testHelper.invite },
				{ headers: { 'User-Agent': testHelper.userAgent } }
			);
			const verifyString = await testHelper.redis.keys(`${RedisKey.VERIFY_STRING}*`);
			const mqMessage = testHelper.mockedRabbitSendEmail.mock.calls[0];
			if (!mqMessage) return;
			const m = mqMessage[0];
			expect(m).toBeTruthy();
			const vString = verifyString[0]?.split('verify:string:')[1];
			expect(m.message_name).toEqual('email::verify');
			expect(m.data).toHaveProperty('verifyString', vString);
			expect(m.data).toHaveProperty('ipId');
			expect(m.data).toHaveProperty('userAgentId');
			expect(m.data).toHaveProperty('firstName', testHelper.firstName);
			expect(m.data).toHaveProperty('email', testHelper.email);
		});

	});

	describe(`ROUTE - ${url_verify}:id`, () => {

		beforeEach(async () => testHelper.beforeEach());
		
		it('DELETE - should return unknown 404', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.delete(url_verify);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(404);
				expect(e.response?.data).toStrictEqual(testHelper.response_unknown);
			}
		});
	
		it('POST - should return unknown 404', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.post(url_verify);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(404);
				expect(e.response?.data).toStrictEqual(testHelper.response_unknown);
			}
		});
	
		it('PATCH - should return unknown 404', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.patch(url_verify);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(404);
				expect(e.response?.data).toStrictEqual(testHelper.response_unknown);
			}
		});
	
		it('PUT - should return unknown 404', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.put(url_verify);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(404);
				expect(e.response?.data).toStrictEqual(testHelper.response_unknown);
			}
		});

		it('GET invalid verification string (random 128) responds with a 400', async () => {
			expect.assertions(2);
			await insertUserPreVerify();
			const hex128 = await testHelper.randomHex(128);
			try {
				await testHelper.axios.get(`${url_verify}/${hex128}`);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`incorrect verification data`));
			}
		});

		it('GET invalid verification string hex string (random 16) responds with a 400', async () => {
			expect.assertions(2);
			await insertUserPreVerify();
			const hex16 = await testHelper.randomHex(16);
			try {
				await testHelper.axios.get(`${url_verify}/${hex16}`);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`incorrect verification data`));
			}
		});
		
		it('GET valid verify string, user in postgres & response 200 "Account verified..."', async () => {
			expect.assertions(4);
			await insertUserPreVerify();
			const verifyString = await testHelper.redis.keys(`${RedisKey.VERIFY_STRING}*`);
			if (!verifyString || !verifyString[0]) throw Error('!verifyString');
			const verifyStringHex = verifyString[0].split(RedisKey.VERIFY_STRING)[1];
			if (!verifyStringHex) throw Error('!verifyStringHex');
			const result = await testHelper.axios.get(`${url_verify}/${verifyStringHex}`);
			const userQuery = await testHelper.query_selectUser();
			expect(result.data).toStrictEqual({ response: 'Account verified, please sign in to continue' });
			expect(result.status).toStrictEqual(200);
			expect(userQuery).toBeTruthy();
			expect(userQuery.active).toBeTruthy();
		});

	});

	describe(`ROUTE - ${url_resetPassword}`, () => {

		beforeEach(async () => testHelper.beforeEach());

		it('DELETE - should return unknown 404', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.delete(url_resetPassword);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(404);
				expect(e.response?.data).toStrictEqual(testHelper.response_unknown);
			}
		});
	
		it('GET - should return unknown 404', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.get(url_resetPassword);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(404);
				expect(e.response?.data).toStrictEqual(testHelper.response_unknown);
			}
		});
	
		it('PATCH - should return unknown 404', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.patch(url_resetPassword);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(404);
				expect(e.response?.data).toStrictEqual(testHelper.response_unknown);
			}
		});
	
		it('PUT - should return unknown 404', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.put(url_resetPassword);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(404);
				expect(e.response?.data).toStrictEqual(testHelper.response_unknown);
			}
		});

		it('POST invalid email, response 400, body: invalid user data - no body', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.post(url_resetPassword);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`email address`));
			}
		});

		it('POST invalid email, response 400, body: invalid user data - empty object', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.post(url_resetPassword, {});
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`email address`));
			}
		});

		it('POST invalid email, response 400, body: invalid user data - empty string', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.post(url_resetPassword, { email: '' });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`email address`));
			}
		});
		
		it('POST invalid email, response 400, body: invalid user data - invalid email address', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.post(url_resetPassword, { email: `${await testHelper.randomHex(2)}@${await testHelper.randomHex(2)}` });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`email address`));
			}
		});

		it('POST unknown email, response 200, body: check inbox', async () => {
			expect.assertions(2);
			const preAt = await testHelper.randomHex(8);
			const postAt = await testHelper.randomHex(8);
			const result = await testHelper.axios.post(url_resetPassword, { email: `${preAt}@${postAt}.com` });
			expect(result.data).toStrictEqual({ response: 'Instructions have been sent to the email address provided' });
			expect(result.status).toStrictEqual(200);
		});

		it('POST known email, password_reset in postgres, response 200, body: check inbox', async () => {
			expect.assertions(2);
			await testHelper.insertUser();
			await testHelper.axios.post(url_resetPassword, { email: testHelper.email });
			const rows = await testHelper.query_selectPasswordReset();
			expect(rows.password_reset_id).toBeTruthy();
			expect(rows.timestamp).toBeTruthy();
		});

		it('POST known email twice, data in db is unchanged', async () => {
			expect.assertions(3);
			await testHelper.insertUser();
			await testHelper.axios.post(url_resetPassword, { email: testHelper.email });
			const firstRows = await testHelper.query_selectPasswordReset();
			await testHelper.axios.post(url_resetPassword, { email: testHelper.email });
			const secondRows = await testHelper.query_selectPasswordReset();
			expect(firstRows.password_reset_id).toBeTruthy();
			expect(firstRows.timestamp).toBeTruthy();
			expect(secondRows).toStrictEqual(firstRows);
		});

		it('POST known email address, expect rabbitSendEmail to have been called', async () => {
			expect.assertions(2);
			await testHelper.insertUser();
			const preCount = testHelper.mockedRabbitSendEmail.mock.calls.length;
			await testHelper.axios.post(url_resetPassword, { email: testHelper.email });
			const postCount = testHelper.mockedRabbitSendEmail.mock.calls.length;
			expect(preCount).toStrictEqual(0);
			expect(postCount).toStrictEqual(1);
		});
		
		it('POST known email address, expect rabbitSendEmail to have been called with data', async () => {
			expect.assertions(8);
			await testHelper.insertUser();
			await testHelper.axios.post(url_resetPassword, { email: testHelper.email }, { headers: { 'User-Agent': testHelper.userAgent } });
			const mqMessage = testHelper.mockedRabbitSendEmail.mock.calls[0];
			if (!mqMessage) return;
			const m = mqMessage[0];
			const rows = await testHelper.query_selectPasswordReset();
			expect(m).toBeTruthy();
			expect(m.message_name).toEqual('email::reset');
			expect(m.data).toHaveProperty('resetString', rows.reset_string);
			expect(m.data).toHaveProperty('ipId', testHelper.ip_id);
			expect(m.data).toHaveProperty('userAgentId', testHelper.user_agent_id);
			expect(m.data).toHaveProperty('firstName', testHelper.firstName);
			expect(m.data).toHaveProperty('userId', testHelper.registered_user_id);
			expect(m.data).toHaveProperty('email', testHelper.email);
		});

	});

	describe(`ROUTE - ${url_resetPassword}/:id`, () => {

		beforeEach(async () => testHelper.beforeEach());

		const createRandomAddress = async (): Promise<string> => `${url_resetPassword}/${await testHelper.randomHex(128)}`;

		const insertPasswordReset = async (): Promise<string> => {
			await testHelper.insertUser();
			await testHelper.axios.post(url_resetPassword, { email: testHelper.email });
			const rows = await testHelper.query_selectPasswordReset();
			return rows.reset_string;
		};

		it('DELETE - should return unknown 404', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.delete(await createRandomAddress());
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(404);
				expect(e.response?.data).toStrictEqual(testHelper.response_unknown);
			}
		});
	
		it('PUT - should return unknown 404', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.put(await createRandomAddress());
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(404);
				expect(e.response?.data).toStrictEqual(testHelper.response_unknown);
			}
		});

		it('GET invalid params, response 400, body: invalid user data', async () => {
			expect.assertions(2);
			const hex127 = await testHelper.randomHex(127);
			try {
				await testHelper.axios.get(`${url_resetPassword}/${hex127}`);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`incorrect verification data`));
			}
		});

		it('GET unknown resetString, response 400, body: Incorrect verification data', async () => {
			expect.assertions(2);
			const hex256 = await testHelper.randomHex(256);
			try {
				await testHelper.axios.get(`${url_resetPassword}/${hex256}`);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual({ response: `Incorrect verification data` });
			}
		});

		it('GET valid resetString, response 200, body empty', async () => {
			expect.assertions(2);
			await testHelper.insertUser();
			await testHelper.axios.post(url_resetPassword, { email: testHelper.email });
			const reset = await testHelper.query_selectPasswordReset();
			const result = await testHelper.axios.get(`${url_resetPassword}/${reset.reset_string}`);
			expect(result.data).toStrictEqual({ response: {
				two_fa_active: false,
				two_fa_backup: false
			} });
			expect(result.status).toStrictEqual(200);
		});

		it('PATCH invalid new password, response 400, body: invalid user data - no body', async () => {
			expect.assertions(2);
			const resetString = await insertPasswordReset();
			try {
				await testHelper.axios.patch(`${url_resetPassword}/${resetString}`);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`passwords are required to be 10 characters minimum`));
			}
		});

		it('PATCH invalid new password, response 400, body: invalid user data - empty object', async () => {
			expect.assertions(2);
			const resetString = await insertPasswordReset();
			try {
				await testHelper.axios.patch(`${url_resetPassword}/${resetString}`, {});
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`passwords are required to be 10 characters minimum`));
			}
		});

		it('PATCH invalid new password, response 400, body: invalid user data - empty string', async () => {
			expect.assertions(2);
			const resetString = await insertPasswordReset();
			try {
				await testHelper.axios.patch(`${url_resetPassword}/${resetString}`, { newPassword: '' });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`passwords are required to be 10 characters minimum`));
			}
		});

		it('PATCH invalid new password, response 400, body: invalid user data - number', async () => {
			expect.assertions(2);
			const resetString = await insertPasswordReset();
			try {
				await testHelper.axios.patch(`${url_resetPassword}/${resetString}`, { newPassword: testHelper.randomNumber() });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`passwords are required to be 10 characters minimum`));
			}
		});

		it('PATCH invalid new password, response 400, body: invalid user data - password too short', async () => {
			expect.assertions(2);
			const resetString = await insertPasswordReset();
			try {
				await testHelper.axios.patch(`${url_resetPassword}/${resetString}`, { newPassword: await testHelper.randomHex(9) });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`passwords are required to be 10 characters minimum`));
			}
		});

		it('PATCH invalid new password, response 400, body: invalid user data - password in hibp', async () => {
			expect.assertions(2);
			const resetString = await insertPasswordReset();
			try {
				await testHelper.axios.patch(`${url_resetPassword}/${resetString}`, { newPassword: 'iloveyou1234' });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual({ response: `The password provided is in a database of compromised passwords and should never be used` });
			}
		});

		it('PATCH invalid new password, response 400, body: invalid user data - is own email address', async () => {
			expect.assertions(2);
			const resetString = await insertPasswordReset();
			const email = testHelper.randomBoolean() ? testHelper.email.toUpperCase() : testHelper.email;
			try {
				await testHelper.axios.patch(`${url_resetPassword}/${resetString}`, { newPassword: email });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual({ response: `New password cannot contain email address` });
			}
		});

		it('PATCH invalid new password, response 400, body: invalid user data - contains own email address', async () => {
			expect.assertions(2);
			const resetString = await insertPasswordReset();
			const email = testHelper.randomBoolean() ? testHelper.email.toUpperCase() : testHelper.email;
			try {
				await testHelper.axios.patch(`${url_resetPassword}/${resetString}`, { newPassword: `${await testHelper.randomHex(5)}${email}${await testHelper.randomHex(5)}` });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual({ response: `New password cannot contain email address` });
			}
		});

		it('PATCH valid resetString & valid password, response 200, body: please sign in, hash changed', async () => {
			expect.assertions(3);
			const resetString = await insertPasswordReset();
			const pre = await testHelper.query_selectUser();
			const result = await testHelper.axios.patch(`${url_resetPassword}/${resetString}`, { newPassword: await testHelper.randomHex(20) });
			const post = await testHelper.query_selectUser();
			expect(pre.password_hash === post.password_hash).toBeFalsy();
			expect(result.data).toStrictEqual({ response: 'Password reset complete - please sign in' });
			expect(result.status).toStrictEqual(200);
		});

		it('PATCH valid resetString & valid password, expect rabbit to have been called', async () => {
			expect.assertions(1);
			const resetString = await insertPasswordReset();
			const preCount = testHelper.mockedRabbitSendEmail.mock.calls.length;
			await testHelper.axios.patch(`${url_resetPassword}/${resetString}`, { newPassword: await testHelper.randomHex(20) });
			const postCount = testHelper.mockedRabbitSendEmail.mock.calls.length;
			expect(postCount-preCount).toStrictEqual(1);
		});

		it('PATCH valid resetString & valid password, expect rabbitMq with correct data', async () => {
			expect.assertions(6);
			const resetString = await insertPasswordReset();
			await testHelper.axios.patch(`${url_resetPassword}/${resetString}`, { newPassword: await testHelper.randomHex(20) });
			const mqMessage = testHelper.mockedRabbitSendEmail.mock.calls[1];
			if (!mqMessage) return;
			const m = mqMessage[0];
			expect(m).toBeTruthy();
			expect(m.message_name).toEqual('email::change_password');
			expect(m.data).toHaveProperty('ipId');
			expect(m.data).toHaveProperty('userAgentId');
			expect(m.data).toHaveProperty('firstName', testHelper.firstName);
			expect(m.data).toHaveProperty('email', testHelper.email);
		});
		
		it('PATCH no token, response 401, body: token invalid', async () => {
			expect.assertions(2);
			const resetString = await insertPasswordReset();
			await testHelper.insert2FA();
			try {
				await testHelper.axios.patch(`${url_resetPassword}/${resetString}`, { newPassword: await testHelper.randomHex(20) });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual({ response: `Token invalid` });
			}
		});

		it('PATCH bad hex token, response 400, body: invalid user data: token invalid', async () => {
			expect.assertions(2);
			const resetString = await insertPasswordReset();
			await testHelper.insert2FA();
			try {
				await testHelper.axios.patch(`${url_resetPassword}/${resetString}`, { newPassword: await testHelper.randomHex(20), token: await testHelper.randomHex(6) });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual({ response: `Invalid user data: token format incorrect` });
			}
		});
		
		it('PATCH incorrect token, response 400, body: token invalid', async () => {
			expect.assertions(2);
			const resetString = await insertPasswordReset();
			await testHelper.insert2FA();
			const badToken = testHelper.generateIncorrectToken(testHelper.generateToken(testHelper.two_fa_secret));
			try {
				await testHelper.axios.patch(`${url_resetPassword}/${resetString}`, { newPassword: await testHelper.randomHex(20), token: badToken });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual({ response: `Token invalid` });
			}
		});

		it('PATCH correct token, response 200, body: token invalid', async () => {
			expect.assertions(2);
			const resetString = await insertPasswordReset();
			await testHelper.insert2FA();
			const token = testHelper.generateToken(testHelper.two_fa_secret);
			const result = await testHelper.axios.patch(`${url_resetPassword}/${resetString}`, { newPassword: await testHelper.randomHex(20), token });
			expect(result.status).toStrictEqual(200);
			expect(result.data).toStrictEqual({ response: 'Password reset complete - please sign in' });
		});

		it('PATCH valid resetString & valid password, expect rabbitMq with correct data', async () => {
			expect.assertions(6);
			const resetString = await insertPasswordReset();
			await testHelper.insert2FA();
			const token = testHelper.generateToken(testHelper.two_fa_secret);
			await testHelper.axios.patch(`${url_resetPassword}/${resetString}`, { newPassword: await testHelper.randomHex(20), token });
			const mqMessage = testHelper.mockedRabbitSendEmail.mock.calls[1];
			if (!mqMessage) return;
			const m = mqMessage[0];
			expect(m).toBeTruthy();
			expect(m.message_name).toEqual('email::change_password');
			expect(m.data).toHaveProperty('ipId');
			expect(m.data).toHaveProperty('userAgentId');
			expect(m.data).toHaveProperty('firstName', testHelper.firstName);
			expect(m.data).toHaveProperty('email', testHelper.email);
		});

		it('PATCH able to sign in with new password', async () => {
			expect.assertions(2);
			const randomPassword = await testHelper.randomHex(20);
			const resetString = await insertPasswordReset();
			await testHelper.axios.patch(`${url_resetPassword}/${resetString}`, { newPassword: randomPassword });
			const result = await testHelper.axios.post(url_signin, { email: testHelper.email, password: randomPassword });
			expect(result.status).toStrictEqual(200);
			expect(result.data).toStrictEqual(testHelper.response_empty);
		});
	
	});

	describe(`ROUTE - ${url_signin}`, () => {
		beforeEach(async () => testHelper.beforeEach());

		const multipleLogins = async (num = 5) :Promise<void> => {
			const randomPassword = await testHelper.randomHex(20);
			for (const _i of new Array(num)) {
				try {
					// eslint-disable-next-line no-await-in-loop
					await testHelper.axios.post(url_signin, { email: testHelper.email, password: randomPassword });
				} catch (e) {
					// void, don't throw, just accept and move on
				}
			}
		};

		it('GET - should return unknown 404', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.delete(url_signin);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(404);
				expect(e.response?.data).toStrictEqual(testHelper.response_unknown);
			}
		});
	
		it('PATCH - should return unknown 404', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.put(url_signin);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(404);
				expect(e.response?.data).toStrictEqual(testHelper.response_unknown);
			}
		});

		it('DELETE - should return unknown 404', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.delete(url_signin);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(404);
				expect(e.response?.data).toStrictEqual(testHelper.response_unknown);
			}
		});
	
		it('PUT - should return unknown 404', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.put(url_signin);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(404);
				expect(e.response?.data).toStrictEqual(testHelper.response_unknown);
			}
		});

		it('POST invalid body, response 400, body: invalid user data: no body', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.post(url_signin);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`email address`));
			}
		});

		it('POST invalid body, response 400, body: invalid user data: no email', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.post(url_signin, { password: await testHelper.randomHex(14) });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`email address`));
			}
		});

		it('POST invalid body, response 400, body: invalid user data: no password', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.post(url_signin, { email: testHelper.email });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`passwords are required to be 10 characters minimum`));
			}
		});

		it('POST invalid body, response 400, body: invalid email + password', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.post(url_signin, { email: testHelper.email, password: await testHelper.randomHex(15) });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(401);
				expect(e.response?.data).toStrictEqual(testHelper.response_invalidLogin);
			}
		});

		it('POST invalid body, response 400, body: invalid email + password', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.post(url_signin, { email: testHelper.email, password: await testHelper.randomHex(15) });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(401);
				expect(e.response?.data).toStrictEqual(testHelper.response_invalidLogin);
			}
		});

		it('POST invalid body, response 400, body: invalid password', async () => {
			expect.assertions(2);
			await testHelper.insertUser();
			try {
				await testHelper.axios.post(url_signin, { email: testHelper.email, password: await testHelper.randomHex(15) });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(401);
				expect(e.response?.data).toStrictEqual(testHelper.response_invalidLogin);
			}
		});

		it('POST correct email/password, response 200, body: empty, loginAttempt increased', async () => {
			expect.assertions(4);
			await testHelper.insertUser();
			const pre_count = await testHelper.query_selectLoginCount();
			const result = await testHelper.axios.post(url_signin, { email: testHelper.email, password: testHelper.password, token: null, twoFABackup: null, remember: null });
			const post_count = await testHelper.query_selectLoginCount();
			expect(pre_count).toStrictEqual('0');
			expect(result.data).toStrictEqual({ response: '' });
			expect(result.status).toStrictEqual(200);
			expect(post_count).toStrictEqual('1');
		});

		it('POST correct email/password, without token, expect 202 response', async () => {
			expect.assertions(4);
			const pre_count = await testHelper.query_selectLoginCount();
			await testHelper.insertUser();
			await testHelper.insert2FA();
			const result = await testHelper.axios.post(url_signin, { email: testHelper.email, password: testHelper.password, token: null, twoFABackup: null, remember: null });
			const post_count = await testHelper.query_selectLoginCount();
			expect(pre_count).toStrictEqual('0');
			expect(result.data).toStrictEqual({ response: '' });
			expect(result.status).toStrictEqual(202);
			expect(post_count).toStrictEqual('0');
		});

		it('POST correct email/password, with token, response 200, body: empty, loginAttempt increased', async () => {
			expect.assertions(4);
			const pre_count = await testHelper.query_selectLoginCount();
			await testHelper.insertUser();
			await testHelper.insert2FA();
			if (!testHelper.two_fa_secret) throw Error('!two_fa_secret');
			const token = testHelper.generateTokenFromString(testHelper.two_fa_secret);
			if (!token) throw Error('!token');
			const result = await testHelper.axios.post(url_signin, { email: testHelper.email, password: testHelper.password, token, twoFABackup: false });
			const post_count = await testHelper.query_selectLoginCount();
			expect(pre_count).toStrictEqual('0');
			expect(result.data).toStrictEqual({ response: '' });
			expect(result.status).toStrictEqual(200);
			expect(post_count).toStrictEqual('1');
		});
		
		it('POST - should return 200 when logging in with backup token, backup count reduced by one', async () => {
			expect.assertions(4);
			await testHelper.insertUser();
			await testHelper.request_signin();
			await testHelper.insert2FA();
			await testHelper.insert2FABackup();
			const pre_count = await testHelper.query_selectBackupCodes();
			const result02 = await testHelper.request_signin({ body: { email: testHelper.email, password: testHelper.password, token: testHelper.two_fa_backups[0], twoFABackup: true } });
			const post_count = await testHelper.query_selectBackupCodes();
			expect(result02.status).toEqual(200);
			expect(result02.data).toEqual(testHelper.response_empty);
			expect(pre_count.length === post_count.length).toBeFalsy();
			expect(pre_count.length - post_count.length === 1).toBeTruthy();
		});

		it('POST - valid signin, session in redis and redisSet', async () => {
			expect.assertions(4);
			await testHelper.insertUser();
			await testHelper.request_signin();
			const redisKeys = await testHelper.redis.keys('*');
			if (redisKeys.length !== 2) throw Error('!redisKeys');
			const sessionkeyIndex = redisKeys.findIndex((i) => i.startsWith('session:'));
			const userSessionIndex = redisKeys.findIndex((i) => i.startsWith('set:session:'));
			if (sessionkeyIndex <0 || userSessionIndex <0) throw Error('!indexes');
			const sessionKey = redisKeys[sessionkeyIndex];
			const sessionUserKey = redisKeys[userSessionIndex];
			if (!sessionKey||!sessionUserKey) throw new Error('!sessionKey|!sessionUserKey');
			const setMember = await testHelper.redis.smembers(sessionUserKey);
			if (!setMember[0]) throw Error('!setMember[0]');
			expect(redisKeys.length).toStrictEqual(2);
			expect(sessionKey).toMatch(testHelper.regex_session);
			expect(sessionUserKey).toMatch(testHelper.regex_sessionSet);
			expect(setMember[0]).toMatch(sessionKey);
		});

		it('POST correct email/password, with real token, but setting backupToken to true, expect 401 invalid', async () => {
			expect.assertions(4);
			const pre_count = await testHelper.query_selectLoginCount();
			await testHelper.insertUser();
			await testHelper.insert2FA();
			if (!testHelper.two_fa_secret) throw Error('!two_fa_secret');
			const token = testHelper.generateTokenFromString(testHelper.two_fa_secret);
			if (!token) throw Error('!token');
			try {
				await testHelper.axios.post(url_signin, { email: testHelper.email, password: testHelper.password, token, twoFABackup: true });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.data).toStrictEqual(testHelper.response_invalidLogin);
				expect(e.response?.status).toStrictEqual(401);
			}
			const post_count = await testHelper.query_selectLoginCount();
			expect(pre_count).toStrictEqual('0');
			expect(post_count).toStrictEqual('0');
		});

		it('POST invalid email/password x 5, email sent to user', async () => {
			expect.assertions(2);
			await testHelper.insertUser();
			const preCount = testHelper.mockedRabbitSendEmail.mock.calls.length;
			await multipleLogins(6);
			const postCount = testHelper.mockedRabbitSendEmail.mock.calls.length;
			expect(preCount).toEqual(0);
			expect(postCount).toEqual(1);
		});

		it('POST invalid email/password x 5, expect rabbiMq to be called with correct data', async () => {
			expect.assertions(6);
			await testHelper.insertUser();
			await multipleLogins(6);
			const mqMessage = testHelper.mockedRabbitSendEmail.mock.calls[0];
			if (!mqMessage) return;
			const m = mqMessage[0];
			expect(m).toBeTruthy();
			expect(m.message_name).toEqual('email::login_attempt');
			expect(m.data).toHaveProperty('ipId', testHelper.ip_id);
			expect(m.data).toHaveProperty('userAgentId', testHelper.user_agent_id);
			expect(m.data).toHaveProperty('firstName', testHelper.firstName);
			expect(m.data).toHaveProperty('email', testHelper.email);

		});

		it('POST invalid email/password x 20, email sent to user', async () => {
			expect.assertions(2);
			await testHelper.insertUser();
			const preCount = testHelper.mockedRabbitSendEmail.mock.calls.length;
			await multipleLogins(20);
			const postCount = testHelper.mockedRabbitSendEmail.mock.calls.length;
			expect(preCount).toEqual(0);
			expect(postCount).toEqual(1);
		});

		it('POST invalid email/password x 20, 401 user blocked from login', async () => {
			expect.assertions(2);
			await testHelper.insertUser();
			await multipleLogins(20);
			try {
				await testHelper.request_signin({ body: { email: testHelper.email, password: testHelper.password } });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(401);
				expect(e.response?.data).toStrictEqual({ response: 'You have been locked out - please contact support to unblock your account' });
			}
		});
	
		it('POST invalid email/password x 7, email sent to user, login with correct details responds 200 empty', async () => {
			expect.assertions(2);
			await testHelper.insertUser();
			await multipleLogins(7);
			const result = await testHelper.request_signin({ body: { email: testHelper.email, password: testHelper.password } });
			expect(result.status).toStrictEqual(200);
			expect(result.data).toStrictEqual(testHelper.response_empty);
		});

		it('POST invalid backup token, 401 response', async () => {
			expect.assertions(2);
			await testHelper.insertUser();
			await testHelper.insert2FA();
			await testHelper.insert2FABackup();
			try {
				await testHelper.request_signin({ body: { email: testHelper.email, password: testHelper.password, twoFABackup: true, token: await testHelper.randomHex(16) } });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.data).toStrictEqual(testHelper.response_invalidLogin);
				expect(e.response?.status).toStrictEqual(401);
			}
		});
		
		it('POST valid login, backupCode removed', async () => {
			expect.assertions(3);
			await testHelper.insertUser();
			await testHelper.insert2FA();
			await testHelper.insert2FABackup();
			const preCount = await testHelper.query_select2FABackupCount();
			const result = await testHelper.request_signin({ body: { email: testHelper.email, password: testHelper.password, twoFABackup: true, token: testHelper.two_fa_backups[0] } });
			const postCount = await testHelper.query_select2FABackupCount();
			expect(preCount-postCount).toStrictEqual(1);
			expect(result.status).toStrictEqual(200);
			expect(result.data).toStrictEqual(testHelper.response_empty);
		});

		it('POST invalid login, when using twoFABackup token twice', async () => {
			expect.assertions(2);
			await testHelper.insertUser();
			await testHelper.insert2FA();
			await testHelper.insert2FABackup();
			await testHelper.request_signin({ body: { email: testHelper.email, password: testHelper.password, twoFABackup: true, token: testHelper.two_fa_backups[0] } });
			await testHelper.axios.post(url_signout);
			try {
				await testHelper.request_signin({ body: { email: testHelper.email, password: testHelper.password, twoFABackup: true, token: testHelper.two_fa_backups[0] } });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.data).toStrictEqual(testHelper.response_invalidLogin);
				expect(e.response?.status).toStrictEqual(401);
			}
		});

	});

});