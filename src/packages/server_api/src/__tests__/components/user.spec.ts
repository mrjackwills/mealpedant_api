import { TestHelper } from '../testHelper';
import { afterAll, beforeAll, beforeEach, describe, expect, it } from 'vitest';

const testHelper = new TestHelper();
const url_base = `/authenticated/user`;
const url_signout = `${url_base}/signout`;
const url_password = `${url_base}/password`;
const url_setupTwoFA = `${url_base}/setup/twofa`;
const url_twoFA = `${url_base}/twofa`;

describe('User components test runner', () => {

	beforeAll(async () => testHelper.beforeAll());

	beforeEach(async () => testHelper.beforeEach());

	afterAll(async () => testHelper.afterAll());
	
	describe(`ROUTE - ${url_base}`, () => {

		beforeEach(async () => testHelper.beforeEach());

		it('GET - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.get(url_base);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
		
		it('DELETE - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.delete(url_base);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
		
		it('POST - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.post(url_base);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
		
		it('PATCH - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				
				await testHelper.axios.patch(url_base);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
		
		it('PUT - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.put(url_base);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});

		it('GET - should return 200 user object', async () => {
			expect.assertions(2);
			await testHelper.insertUser();
			await testHelper.request_signin();
			const result = await testHelper.axios.get(url_base);
			expect(result.status).toEqual(200);
			expect(result.data).toEqual(testHelper.responseUser);
		});

		it('GET - should return 200 user object', async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			const result = await testHelper.axios.get(url_base);
			expect(result.status).toEqual(200);
			expect(result.data).toEqual(testHelper.responseAdmin);
		});
	
	});
	
	describe(`ROUTE - ${url_signout}`, () => {
		beforeEach(async () => testHelper.beforeEach());
		it('GET - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.get(url_signout);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
		
		it('DELETE - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.delete(url_signout);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
		
		it('POST - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.post(url_signout);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
		
		it('PATCH - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				
				await testHelper.axios.patch(url_signout);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
		
		it('PUT - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.put(url_signout);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
		
		it('POST - valid signout, should return unauthorized 403 on second request', async () => {
			expect.assertions(4);
			await testHelper.insertUser();
			await testHelper.request_signin();
			const result = await testHelper.axios.post(url_signout);
			try {
				await testHelper.axios.post(url_signout);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toEqual(403);
				expect(e.response?.data).toEqual(testHelper.response_unauthorized);

			}
			expect(result.status).toEqual(200);
			expect(result.data).toEqual(testHelper.response_empty);
		});
		
		it('POST - valid signout, should clear session data from redis', async () => {
			expect.assertions(2);
			await testHelper.insertUser();
			await testHelper.request_signin();
			const preSignoutRedisKeys = await testHelper.redis.keys('*');
			await testHelper.axios.post(url_signout);
			const postSignoutRedisKeys = await testHelper.redis.keys('*');
			expect(preSignoutRedisKeys.length).toEqual(2);
			expect(postSignoutRedisKeys.length).toEqual(0);
		});
	
	});
	
	describe(`ROUTE - ${url_password}`, () => {
		beforeEach(async () => testHelper.beforeEach());

		it('GET - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.get(url_password);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
		
		it('DELETE - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.delete(url_password);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
		
		it('POST - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.post(url_password);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
		
		it('PATCH - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.patch(url_password);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
		
		it('PUT - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.put(url_password);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});

		it('PATCH - should return unauthorized 400 invalid user data - no new password', async () => {
			expect.assertions(2);
			try {
				await testHelper.insertUser();
				await testHelper.request_signin();
				await testHelper.axios.patch(url_password, { password: testHelper.password });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse('passwords are required to be 10 characters minimum'));
			}
		});
		it('PATCH - should return unauthorized 400 invalid user data - new password invalid', async () => {
			expect.assertions(2);
			try {
				const newPasswordArray = [ '', null, 1111111111, testHelper.randomBoolean(), await testHelper.randomHex(8) ];
				const newPassword = newPasswordArray[Math.floor(Math.random() * newPasswordArray.length)];
				await testHelper.insertUser();
				await testHelper.request_signin();
				await testHelper.axios.patch(url_password, { password: testHelper.password, newPassword });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse('passwords are required to be 10 characters minimum'));
			}
		});
		
		it('PATCH - should return unauthorized 400 invalid user data - no new password', async () => {
			expect.assertions(2);
			try {
				await testHelper.insertUser();
				await testHelper.request_signin();
				await testHelper.axios.patch(url_password, { newPassword: await testHelper.randomHex(12) });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse('passwords are required to be 10 characters minimum'));
			}
		});
		it('PATCH - should return unauthorized 400 invalid user data - current password invalid', async () => {
			expect.assertions(2);
			try {
				const passwordArray = [ '', null, 1111111111, testHelper.randomBoolean(), await testHelper.randomHex(8) ];
				const password = passwordArray[Math.floor(Math.random() * passwordArray.length)];
				await testHelper.insertUser();
				await testHelper.request_signin();
				await testHelper.axios.patch(url_password, { password, newPassword: await testHelper.randomHex(12) });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse('passwords are required to be 10 characters minimum'));
			}
		});
		
		it('PATCH - invalid body password, should return 400 when new password is current email address', async () => {
			expect.assertions(2);
			await testHelper.insertUser();
			await testHelper.request_signin();
			try {
				await testHelper.axios.patch(url_password, { password: testHelper.password, newPassword: testHelper.email });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual({ response: `New password cannot contain email address` });
			}
		});

		it('PATCH - invalid body password, should return 403 incorrect password and/or token', async () => {
			expect.assertions(2);
			await testHelper.insertUser();
			await testHelper.request_signin();
			try {
				await testHelper.axios.patch(url_password, { password: await testHelper.randomHex(20), newPassword: await testHelper.randomHex(20) });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(401);
				expect(e.response?.data).toStrictEqual(testHelper.response_incorrectPasswordOrToken);
			}
		});

		it('PATCH - incorrect newPassword HIBP, should return 400 invalid body', async () => {
			expect.assertions(2);
			await testHelper.insertUser();
			await testHelper.request_signin();
			try {
				await testHelper.axios.patch(url_password, { password: testHelper.password, newPassword: 'iloveyou1234' });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual({ response: 'The password provided is in a database of compromised passwords and should never be used' });
			}
		});

		it(`PATCH - 2fa always required, invalid token provided, return 400 invalid`, async () => {
			expect.assertions(3);
			await testHelper.insertUser();
			await testHelper.insert2FAAlwaysRequired();
			if (!testHelper.two_fa_secret) throw Error('!two_fa_secret)');
			await testHelper.request_signin({ body: { email: testHelper.email, password: testHelper.password, token: testHelper.generateTokenFromString(testHelper.two_fa_secret) } });
			const prePassword = await testHelper.query_selectUser();
			try {
				const newPassword = await testHelper.randomHex(20);
				await testHelper.axios.patch(url_password, { password: testHelper.password, newPassword });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(401);
				expect(e.response?.data).toStrictEqual(testHelper.response_incorrectPasswordOrToken);
			}
			const postPassword = await testHelper.query_selectUser();
			expect(prePassword.password_hash).toStrictEqual(postPassword.password_hash);
		});
		
		it(`PATCH - 2fa always required, invalid token provided, return 400 invalid`, async () => {
			expect.assertions(3);
			await testHelper.insertUser();
			await testHelper.insert2FAAlwaysRequired();
			if (!testHelper.two_fa_secret) throw Error('!two_fa_secret)');
			await testHelper.request_signin({ body: { email: testHelper.email, password: testHelper.password, token: testHelper.generateTokenFromString(testHelper.two_fa_secret) } });
			const prePassword = await testHelper.query_selectUser();
			try {
				const newPassword = await testHelper.randomHex(20);
				const invalidToken = testHelper.generateIncorrectToken(testHelper.generateTokenFromString(testHelper.two_fa_secret));
				await testHelper.axios.patch(url_password, { password: testHelper.password, token: invalidToken, newPassword });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(401);
				expect(e.response?.data).toStrictEqual(testHelper.response_incorrectPasswordOrToken);
			}
			const postPassword = await testHelper.query_selectUser();
			expect(prePassword.password_hash).toStrictEqual(postPassword.password_hash);
		});

		it(`PATCH - valid password change, return 200 invalid body, old password no longer valid for logins, new password does`, async () => {
			expect.assertions(7);
			await testHelper.insertUser();
			await testHelper.request_signin();

			const newPassword = await testHelper.randomHex(20);
			const pre_change = await testHelper.query_selectUser();
			const result = await testHelper.axios.patch(url_password, { password: testHelper.password, newPassword });
			const post_change = await testHelper.query_selectUser();
			await testHelper.axios.post(url_signout);
			try {
				await testHelper.request_signin();
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toEqual(401);
				expect(e.response?.data).toEqual(testHelper.response_invalidLogin);
			}
			const signinWithNewPassword = await testHelper.request_signin({ body: { email: testHelper.email, password: newPassword } });
		
			expect(result.status).toEqual(200);
			expect(result.data).toEqual({ response: '' });
			expect(pre_change.password_hash === post_change.password_hash).toBeFalsy();
			expect(signinWithNewPassword.data).toEqual(testHelper.response_empty);
			expect(signinWithNewPassword.status).toEqual(200);
		});

		it(`PATCH - valid password change, expect email to have been send`, async () => {
			expect.assertions(2);
			await testHelper.insertUser();
			await testHelper.request_signin();
			const newPassword = await testHelper.randomHex(20);
			const preCount = testHelper.mockedRabbitSendEmail.mock.calls.length;
			await testHelper.axios.patch(url_password, { password: testHelper.password, newPassword });
			const postCount = testHelper.mockedRabbitSendEmail.mock.calls.length;
			expect(preCount).toStrictEqual(0);
			expect(postCount).toStrictEqual(1);
		});
		
		it(`PATCH - valid password change, expect rabbitMQ to be called with correct data`, async () => {
			expect.assertions(7);
			await testHelper.insertUser();
			await testHelper.request_signin();
			const newPassword = await testHelper.randomHex(20);
			await testHelper.axios.patch(url_password, { password: testHelper.password, newPassword });
			const mqMessage = testHelper.mockedRabbitSendEmail.mock.calls[0];
			if (!mqMessage) return;
			const m = mqMessage[0];
			expect(m).toBeTruthy();
			expect(m.message_name).toEqual('email::change_password');
			expect(m.data).toHaveProperty('ipId', testHelper.ip_id);
			expect(m.data).toHaveProperty('userAgentId', testHelper.user_agent_id);
			expect(m.data).toHaveProperty('firstName', testHelper.firstName);
			expect(m.data).toHaveProperty('userId', testHelper.registered_user_id);
			expect(m.data).toHaveProperty('email', testHelper.email);
		});
		
		it(`PATCH - valid password change when 2FA always required is set, return 200 invalid body, old password no longer valid for logins, new password does`, async () => {
			expect.assertions(7);
			await testHelper.insertUser();
			await testHelper.request_signin();
			await testHelper.insert2FAAlwaysRequired();
			if (!testHelper.two_fa_secret) throw Error('!two_fa_secret)');
			const newPassword = await testHelper.randomHex(20);
			const pre_change = await testHelper.query_selectUser();
			const result = await testHelper.axios.patch(url_password, { password: testHelper.password, token: testHelper.generateTokenFromString(testHelper.two_fa_secret), newPassword });
			const post_change = await testHelper.query_selectUser();
			await testHelper.axios.post(url_signout);
			try {
				await testHelper.request_signin({ body: { email: testHelper.email, password: testHelper.password, token: testHelper.generateTokenFromString(testHelper.two_fa_secret) } });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(401);
				expect(e.response?.data).toStrictEqual(testHelper.response_invalidLogin);
			}
			const signinWithNewPassword = await testHelper.request_signin({ body: { email: testHelper.email, password: newPassword, token: testHelper.generateTokenFromString(testHelper.two_fa_secret) } });
			expect(result.status).toStrictEqual(200);
			expect(result.data).toStrictEqual({ response: '' });
			expect(pre_change.password_hash === post_change.password_hash).toBeFalsy();
			expect(signinWithNewPassword.data).toStrictEqual(testHelper.response_empty);
			expect(signinWithNewPassword.status).toStrictEqual(200);
		});
	
	});

	describe(`ROUTE - ${url_setupTwoFA}`, () => {

		beforeEach(async () => testHelper.beforeEach());

		it('GET - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.get(url_setupTwoFA);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
		
		it('DELETE - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.delete(url_setupTwoFA);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
		
		it('POST - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.post(url_setupTwoFA);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
		
		it('PATCH - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				
				await testHelper.axios.patch(url_setupTwoFA);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
		
		it('PUT - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.put(url_setupTwoFA);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});

		it('GET - should return 200 body containing valid secret token, secret in redis', async () => {
			expect.assertions(6);
			await testHelper.insertUser();
			await testHelper.request_signin();
			const result = await testHelper.axios.get(url_setupTwoFA);
			const keyInRedis = await testHelper.redis.get(`2fa:setup:${testHelper.registered_user_id}`);
			const redisTtl = await testHelper.redis.ttl(`2fa:setup:${testHelper.registered_user_id}`);
			const secret = result.data.response.secret;
			expect(redisTtl).toEqual(90);
			expect(result.status).toEqual(200);
			expect(result.data.response).toHaveProperty('secret');
			expect(testHelper.regex_otpSecretRegex.test(secret)).toBeTruthy();
			expect(keyInRedis).toBeTruthy();
			expect(keyInRedis === secret).toBeTruthy();
		});

		it('POST - should return errors on invalid token', async () => {
			expect.assertions(2);
			await testHelper.insertUser();
			await testHelper.request_signin();
			await testHelper.axios.get(url_setupTwoFA);
			const tokenArray = [ '', null, '00000', '1111111', testHelper.randomBoolean(), await testHelper.randomHex(8) ];
			const token = tokenArray[Math.floor(Math.random() * tokenArray.length)];
			try {
				await testHelper.axios.post(url_setupTwoFA, { token });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse('token format incorrect'));
			}
		});

		it('POST - should return error on invalid generate token from secret', async () => {
			expect.assertions(2);
			await testHelper.insertUser();
			await testHelper.request_signin();
			const secretRequest = await testHelper.axios.get(url_setupTwoFA);
			const token = testHelper.generateToken(secretRequest.data.response.secret);
			try {
				await testHelper.axios.post(url_setupTwoFA, { token: testHelper.generateIncorrectToken(token) });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual({ response: 'The app-generated code provided is incorrect' });
			}
		});
		
		it('POST - valid 2fa enable and login with token, auth check 2fa status = true', async () => {
			expect.assertions(2);
			await testHelper.insertUser();
			await testHelper.request_signin();
			const secretRequest = await testHelper.axios.get(url_setupTwoFA);
			const token = testHelper.generateToken(secretRequest.data.response.secret);
			await testHelper.axios.post(url_setupTwoFA, { token });
			await testHelper.axios.post(url_signout);
			await testHelper.request_signin({ body: { email: testHelper.email, password: testHelper.password, token: token } });
			const result = await testHelper.axios.get(url_base);
			const tfaActive = {
				...testHelper.responseUser.response,
				two_fa_active: true
			};
			expect(result.status).toEqual(200);
			expect(result.data.response).toEqual(tfaActive);
		});

		it('GET - 409 2FA setup in progress', async () => {
			expect.assertions(2);
			await testHelper.insertUser();
			await testHelper.request_signin();
			await testHelper.axios.get(url_setupTwoFA);
			try {
				await testHelper.axios.get(url_setupTwoFA);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(409);
				expect(e.response?.data).toStrictEqual({ response: '2FA setup already in progress' });
			}
		});

		it('GET - 409 2FA already enabled', async () => {
			expect.assertions(2);
			expect.assertions(2);
			await testHelper.insertUser();
			await testHelper.request_signin();
			const secretRequest = await testHelper.axios.get(url_setupTwoFA);
			const token = testHelper.generateToken(secretRequest.data.response.secret);
			await testHelper.axios.post(url_setupTwoFA, { token });
			try {
				await testHelper.axios.get(url_setupTwoFA);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(409);
				expect(e.response?.data).toStrictEqual({ response: '2FA is already enabled' });
			}

		});
		
		it('PATCH - 409 2FA is not enabled', async () => {
			expect.assertions(2);
			await testHelper.insertUser();
			await testHelper.request_signin();
			try {
				await testHelper.axios.patch(url_setupTwoFA);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(409);
				expect(e.response?.data).toStrictEqual({ response: '2FA is not enabled' });
			}
		});

		it('PATCH - 400 invalid body - alwaysRequired', async () => {
			expect.assertions(2);
			await testHelper.insertUser();
			await testHelper.request_signin();
			const secretRequest = await testHelper.axios.get(url_setupTwoFA);
			const token = testHelper.generateToken(secretRequest.data.response.secret);
			await testHelper.axios.post(url_setupTwoFA, { token });
			const alwaysRequiredArray = [ '', null, testHelper.randomNumberAsString(), await testHelper.randomHex(2) ];
			const alwaysRequired = alwaysRequiredArray[Math.floor(Math.random() * alwaysRequiredArray.length)];
			try {
				await testHelper.axios.patch(url_setupTwoFA, { alwaysRequired });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse('alwaysRequired'));
			}
		});

		it('PATCH - 200 empty alwaysRequired enabled', async () => {
			expect.assertions(2);
			await testHelper.insertUser();
			await testHelper.request_signin();
			const secretRequest = await testHelper.axios.get(url_setupTwoFA);
			const token = testHelper.generateToken(secretRequest.data.response.secret);
			await testHelper.axios.post(url_setupTwoFA, { token });
			const result = await testHelper.axios.patch(url_setupTwoFA, { alwaysRequired: true });
			expect(result.status).toEqual(200);
			expect(result.data).toEqual(testHelper.response_empty);
		});

		it('DELETE - remove setup from redis, 200 empty response', async () => {
			expect.assertions(4);
			await testHelper.insertUser();
			await testHelper.request_signin();
			await testHelper.axios.get(url_setupTwoFA);
			const pre_delete = await testHelper.redis.exists(`2fa:setup:${testHelper.registered_user_id}`);
			const result = await testHelper.axios.delete(url_setupTwoFA);
			const post_delete = await testHelper.redis.exists(`2fa:setup:${testHelper.registered_user_id}`);
			expect(pre_delete).toEqual(1);
			expect(result.status).toEqual(200);
			expect(result.data).toEqual(testHelper.response_empty);
			expect(post_delete).toEqual(0);
		});
	
	});

	describe(`ROUTE - ${url_twoFA}`, () => {
		
		beforeEach(async () => testHelper.beforeEach());

		it('GET - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.get(url_twoFA);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
		
		it('DELETE - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.delete(url_twoFA);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
		
		it('POST - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.post(url_twoFA);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
		
		it('PATCH - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				
				await testHelper.axios.patch(url_twoFA);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
		
		it('PUT - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.put(url_twoFA);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});

		it('GET - should return 2fa objects', async () => {
			expect.assertions(3);
			await testHelper.insertUser();
			await testHelper.request_signin();
			const result01 = await testHelper.axios.get(url_twoFA);
			await testHelper.insert2FA();
			const result02 = await testHelper.axios.get(url_twoFA);
			await testHelper.axios.patch(url_setupTwoFA, { alwaysRequired: true });
			const result03 = await testHelper.axios.get(url_twoFA);
			const twoFAObject = {
				two_fa_backup: false,
				two_fa_count: 0,
				two_fa_always_required: false,
				two_fa_active: false,
			};
			const twoFAActive = { ...twoFAObject };
			twoFAActive.two_fa_active = true;
			const twoFAAlwaysActive = { ...twoFAActive };
			twoFAAlwaysActive.two_fa_always_required = true;
			expect(result01.data).toEqual({ response: twoFAObject });
			expect(result02.data).toEqual({ response: twoFAActive });
			expect(result03.data).toEqual({ response: twoFAAlwaysActive });
		});

		it('POST - should 200 return array random tokens, 2FA GET should return true, true, false, 10', async () => {
			expect.assertions(6);
			await testHelper.insertUser();
			await testHelper.request_signin();
			await testHelper.insert2FA();
			const result01 = await testHelper.axios.post(url_twoFA);
			const backups = await testHelper.query_selectBackupCodes();
			const result02 = await testHelper.axios.get(url_twoFA);
			const twoFABackupsEnabled = {
				response: {
					two_fa_active: true,
					two_fa_backup: true,
					two_fa_count: 10,
					two_fa_always_required: false
				}
			};
			expect(backups.length === 10).toBeTruthy();
			expect(result01.data.response).toBeDefined();
			expect(result01.data.response.backups).toBeDefined();
			expect(result01.data.response.backups.length === 10).toBeTruthy();
			expect(result01.data.response.backups[Math.floor(Math.random() * result01.data.response.backups.length)]).toMatch(testHelper.regex_backupToken);
			expect(result02.data).toEqual(twoFABackupsEnabled);
		});

		it('POST - 400 already enabled', async () => {
			expect.assertions(2);
			await testHelper.insertUser();
			await testHelper.request_signin();
			await testHelper.insert2FA();
			await testHelper.axios.post(url_twoFA);
			try {
				await testHelper.axios.post(url_twoFA);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toEqual({ response: 'Backup codes already enabled' });
			}
		});

		it('PATCH - should return 400 password not provided enabled', async () => {
			expect.assertions(2);
			await testHelper.insertUser();
			await testHelper.request_signin();
			await testHelper.insert2FA();
			try {
				await testHelper.axios.patch(url_twoFA);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(401);
				expect(e.response?.data).toEqual({ response: 'Invalid password and/or Two Factor Authentication token' });
			}
		});

		it('PATCH - should return 401 invalid password', async () => {
			expect.assertions(2);
			await testHelper.insertUser();
			await testHelper.request_signin();
			await testHelper.insert2FA();
			try {
				await testHelper.axios.patch(url_twoFA, { password: await testHelper.randomHex(16) });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(401);
				expect(e.response?.data).toEqual(testHelper.response_incorrectPasswordOrToken);
			}
		});

		it('PATCH - Refresh 2fa tokens, new random selection from old token array should not be in new array', async () => {
			expect.assertions(5);
			await testHelper.insertUser();
			await testHelper.request_signin();
			await testHelper.insert2FA();
			await testHelper.axios.post(url_twoFA);
			const pre_patch = await testHelper.query_selectBackupCodes();
			const result01 = await testHelper.axios.patch(url_twoFA, { password: testHelper.password });
			const post_patch = await testHelper.query_selectBackupCodes();
			const randomToken = result01.data.response.backups[Math.floor(Math.random() * result01.data.response.backups.length)];
			const result02 = await testHelper.axios.get(url_twoFA);
			const twoFABackupsEnabled = {
				two_fa_backup: true,
				two_fa_count: 10,
				two_fa_always_required: false,
				two_fa_active: true,
			};
			const randomBackup = Math.floor(Math.random() * post_patch.length);
			expect(pre_patch[randomBackup].two_fa_backup_code === post_patch[randomBackup]).toBeFalsy();
			expect(result01.data.response.backups.length === 10).toBeTruthy();
			expect(randomToken).toMatch(testHelper.regex_backupToken);
			expect(pre_patch).not.toEqual(post_patch);
			expect(result02.data).toEqual({ response: twoFABackupsEnabled });
		});
	});

});