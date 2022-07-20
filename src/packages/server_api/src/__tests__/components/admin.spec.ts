/* eslint-disable no-console */
/* eslint-disable @typescript-eslint/no-explicit-any, max-len */
import { TestHelper } from '../testHelper';
import { promises as fs } from 'fs';
import { LOCATION_BACKUP } from '../../config/env';

import { afterAll, afterEach, beforeAll, beforeEach, describe, expect, it } from 'vitest';

const testHelper = new TestHelper();
const url_base = `/admin`;
const url_backup = `${url_base}/backup`;
const url_email = `${url_base}/email`;
const url_error = `${url_base}/error`;
const url_memory = `${url_base}/memory`;
const url_restart = `${url_base}/restart`;
const url_limit = `${url_base}/limit`;
const url_user = `${url_base}/user`;
const url_session = `${url_base}/session`;

const url_signin = `incognito/signin`;

async function deleteAll (): Promise<void> {
	const files = await fs.readdir(`${LOCATION_BACKUP}`);
	const promiseArray = [];
	for (const file of files) if (file.includes('.gpg')) promiseArray.push(fs.unlink(`${LOCATION_BACKUP}/${file}`));
	await Promise.all(promiseArray);
}

describe('Admin test runner', () => {
	
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
				await testHelper.insertUser();
				await testHelper.axios.put(url_base);
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
				await testHelper.axios.get(url_base);
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
				await testHelper.axios.delete(url_base);
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
				await testHelper.axios.post(url_base);
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
				await testHelper.axios.patch(url_base);
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
				await testHelper.axios.put(url_base);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});

		it('GET responds with a 200 empty', async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			const result = await testHelper.axios.get(url_base);
			expect(result.status).toEqual(200);
			expect(result.data).toEqual(testHelper.response_empty);
		});

	});

	describe(`ROUTE - ${url_backup}`, () => {
		beforeEach(async () => {
			await testHelper.beforeEach();
			await deleteAll();
		});

		afterAll(async () => deleteAll());

		it('DELETE - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.delete(url_backup);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
		
		it('POST - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.post(url_backup);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
		
		it('PATCH - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				
				await testHelper.axios.patch(url_backup);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
		
		it('PUT - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.insertUser();
				await testHelper.request_signin();
				await testHelper.axios.put(url_backup);
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
				await testHelper.axios.delete(url_base);
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
				await testHelper.axios.post(url_backup);
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
				await testHelper.axios.patch(url_backup);
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
				await testHelper.axios.put(url_backup);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});

		it('POST responds with a 400 invalid user data - no body', async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			try {
				await testHelper.axios.post(url_backup, {});
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`backup option`));
			}
		});

		it('POST responds with a 400 invalid user data - withPhoto: randomText', async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			try {
				await testHelper.axios.post(url_backup, { withPhoto: await testHelper.randomHex(10) });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`backup option`));
			}
		});

		it('POST responds with a 400 invalid user data - withPhoto: null', async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			try {
				await testHelper.axios.post(url_backup, { withPhoto: null });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`backup option`));
			}
		});

		it('POST responds with a 400 invalid user data - withPhoto: randomNumber', async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			try {
				await testHelper.axios.post(url_backup, { withPhoto: testHelper.randomNumber() });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`backup option`));
			}
		});

		it('POST responds with a 400 invalid user data - withPhoto: randomBoolean AS string', async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			try {
				await testHelper.axios.post(url_backup, { withPhoto: `${testHelper.randomBoolean()}` });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`backup option`));
			}
		});

		it('POST responds with a 200 empty response, backup folder has 1 file, matching sqlOnly regex, filesize > 3mb', async () => {
			expect.assertions(6);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			const pre_files = await fs.readdir(`${LOCATION_BACKUP}`);
			const request = await testHelper.axios.post(url_backup, { withPhoto: false });
			const post_files = await fs.readdir(`${LOCATION_BACKUP}`);
			if (post_files[0]) expect(post_files[0].match(testHelper.regex_sql_only)).toBeTruthy();
			const filesize = await fs.stat(`${LOCATION_BACKUP}/${post_files[0]}`);
			expect(pre_files).toEqual([]);
			expect(request.status).toEqual(200);
			expect(request.data).toEqual(testHelper.response_empty);
			expect(post_files.length === 1).toBeTruthy();
			expect(filesize.size).toBeGreaterThan(300000);
		});

		it('POST responds with a 200 empty response, backup folder has 1 file, matching full regex, filesize > 300mb', async () => {
			expect.assertions(6);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			const pre_files = await fs.readdir(`${LOCATION_BACKUP}`);
			const request = await testHelper.axios.post(url_backup, { withPhoto: true });
			const post_files = await fs.readdir(`${LOCATION_BACKUP}`);
			if (post_files[0]) expect(post_files[0].match(testHelper.regex_sql_full)).toBeTruthy();
			const filesize = await fs.stat(`${LOCATION_BACKUP}/${post_files[0]}`);
			expect(pre_files).toEqual([]);
			expect(request.status).toEqual(200);
			expect(request.data).toEqual(testHelper.response_empty);
			expect(post_files.length === 1).toBeTruthy();
			expect(filesize.size).toBeGreaterThan(300000000);
		});

		it('GET responds with a 200 empty array when no backups in place', async () => {
			expect.assertions(3);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			const result = await testHelper.axios.get(url_backup);
			expect(result.status).toEqual(200);
			expect(result.data.response).toBeDefined();
			expect(result.data.response).toEqual([]);
		});

		it('GET responds with a 200 list of backups, matching known file names', async () => {
			expect.assertions(8);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			await Promise.all([
				testHelper.axios.post(url_backup, { withPhoto: false }),
				testHelper.axios.post(url_backup, { withPhoto: true }),
			]);
			const post_files = await fs.readdir(`${LOCATION_BACKUP}`);
			const result = await testHelper.axios.get(url_backup);
			expect(result.status).toEqual(200);
			expect(result.data.response).toBeDefined();
			expect(result.data.response.length === 2).toBeTruthy();
			for (const i of result.data.response) {
				expect(i).toHaveProperty('filename');
				expect(i).toHaveProperty('filesize');
			}
			const extractedNames = [ result.data.response[0].filename, result.data.response[1].filename ].sort();
			expect(extractedNames).toEqual(post_files.sort());
		});

		it('DELETE responds with a 400 invalid user data - no body', async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			try {
				await testHelper.axios.delete(url_backup);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`incorrect filename`));
			}
		});

		it('DELETE responds with a 400 invalid user data - empty body', async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			try {
				await testHelper.axios.delete(url_backup, { data: { } });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`incorrect filename`));
			}
		});
		
		it('DELETE responds with a 400 invalid user data - fileName: random string', async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			try {
				await testHelper.axios.delete(url_backup, { data: { fileName: await testHelper.randomHex(30) } });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`incorrect filename`));
			}
		});
			
		it('DELETE responds with a 400 invalid user data - fileName: random number', async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			try {
				await testHelper.axios.delete(url_backup, { data: { fileName: testHelper.randomNumber() } });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`incorrect filename`));
			}
		});

		it('DELETE responds with a 400 invalid user data - fileName: random boolean', async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			try {
				await testHelper.axios.delete(url_backup, { data: { fileName: testHelper.randomBoolean() } });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`incorrect filename`));
			}
		});

		it('DELETE responds with a 400 invalid user data - fileName: short fileName', async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			const tomorrrow = testHelper.generateTomorrow();
			const full = testHelper.randomBoolean();
			const name = `_LOGS_${full? 'PHOTOS_':''}REDIS_SQL_`;
			const file = `mealpedant_${tomorrrow}_15.32.35${name}${await testHelper.randomHex(8)}.tar.${full? '':'gz.'}gpg`;
			await testHelper.request_signin();
			try {
				await testHelper.axios.delete(url_backup, { data: { fileName: file.substring(1,) } });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`incorrect filename`));
			}
		});

		it('DELETE responds with a 400 invalid user data - fileName: none base64 filename', async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			const tomorrrow = testHelper.generateTomorrow();
			const full = testHelper.randomBoolean();
			const name = `_LOGS_${full? 'PHOTOS_':''}REDIS_SQL_`;
			const fileName = `mealpedant_${tomorrrow}_15.32.35${name}${await testHelper.randomHex(7)}.tar.${full? '':'gz.'}gpg`;
			await testHelper.request_signin();
			try {
				await testHelper.axios.delete(url_backup, { data: { fileName } });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`incorrect filename`));
			}
		});

		it('DELETE responds with a 400 file not found', async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			const tomorrrow = testHelper.generateTomorrow();
			const full = testHelper.randomBoolean();
			const name = `_LOGS_${full? 'PHOTOS_':''}REDIS_SQL_`;
			const fileName = `mealpedant_${tomorrrow}_15.32.35${name}${await testHelper.randomHex(8)}.tar.${full? '':'gz.'}gpg`;
			await testHelper.request_signin();
			try {
				await testHelper.axios.delete(url_backup, { data: { fileName } });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual({ response: `File not found` });
			}
		});

		it('DELETE responds with a 200 empty, known file is removed', async () => {
			expect.assertions(4);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			await testHelper.axios.post(url_backup, { withPhoto: false });
			const pre_delete = await fs.readdir(`${LOCATION_BACKUP}`);
			if (!pre_delete[0]) throw Error('No files');
			const result = await testHelper.axios.delete(url_backup, { data: { fileName: pre_delete[0] } });
			const post_delete = await fs.readdir(`${LOCATION_BACKUP}`);
			expect(result.data).toEqual(testHelper.response_empty);
			expect(result.status).toEqual(200);
			expect(post_delete).not.toEqual(pre_delete);
			expect(post_delete).toEqual([]);
		});
	
	});

	describe(`ROUTE - ${url_backup}/:filename`, () => {
		
		beforeEach(async () => {
			await testHelper.beforeEach();
			await deleteAll();
		});

		afterAll(async () => deleteAll());

		const timeNow = (): string => {
			const now = new Date();
			return `${testHelper.zeroPad(now.getHours())}.${testHelper.zeroPad(now.getMinutes())}.${testHelper.zeroPad(now.getSeconds())}`;
		};

		const randomFileName = async () :Promise<string> => {
			const full = testHelper.randomBoolean();
			const name = `_LOGS_${full? 'PHOTOS_':''}REDIS_SQL_`;
			return `mealpedant_${testHelper.generateTomorrow()}_${timeNow()}_${name}_${await testHelper.randomHex(8)}.tar.${full?'':'gz.'}gpg`;
		};

		it('GET - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.get(`${url_backup}/${await randomFileName()}`);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});

		it('DELETE - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.delete(`${url_backup}/${await randomFileName()}`);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
		
		it('POST - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.delete(`${url_backup}/${await randomFileName()}`);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
		
		it('PATCH - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.delete(`${url_backup}/${await randomFileName()}`);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
		
		it('PUT - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.delete(`${url_backup}/${await randomFileName()}`);
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
				await testHelper.axios.get(`${url_backup}/${await randomFileName()}`);
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
				await testHelper.axios.delete(`${url_backup}/${await randomFileName()}`);
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
				await testHelper.axios.delete(`${url_backup}/${await randomFileName()}`);
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
				await testHelper.axios.delete(`${url_backup}/${await randomFileName()}`);
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
				await testHelper.axios.delete(`${url_backup}/${await randomFileName()}`);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});

		it('GET download backup, is buffer, size match known file', async () => {
			expect.assertions(3);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			await testHelper.axios.post(url_backup, { withPhoto: false });
			const file = await fs.readdir(`${LOCATION_BACKUP}`);
			const fileSize = await fs.stat(`${LOCATION_BACKUP}/${file[0]}`);
			const result = await testHelper.axios.get(`${url_backup}/${file[0]}`, { responseType: 'arraybuffer' });
			const downloadedBuffer = Buffer.from(result.data);
			expect(result.status).toBe(200);
			expect(Buffer.isBuffer(result.data)).toBeTruthy();
			expect(downloadedBuffer.length).toEqual(fileSize.size);
		});

	});

	describe(`ROUTE - ${url_email}`, () => {
		
		beforeEach(async () => testHelper.beforeEach());

		it('GET - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.get(`${url_email}`);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});

		it('DELETE - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.get(`${url_email}`);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
		
		it('POST - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.get(`${url_email}`);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
		
		it('PATCH - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.get(`${url_email}`);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
		
		it('PUT - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.get(`${url_email}`);
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
				await testHelper.axios.get(`${url_email}`);
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
				await testHelper.axios.get(`${url_email}`);
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
				await testHelper.axios.get(`${url_email}`);
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
				await testHelper.axios.get(`${url_email}`);
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
				await testHelper.axios.get(`${url_email}`);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});

		it('GET responds array, including jest user email', async () => {
			expect.assertions(3);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			const result = await testHelper.axios.get(url_email);
			expect(result.status).toEqual(200);
			expect(result.data.response.emails).toBeDefined();
			expect(result.data.response.emails.includes(testHelper.email)).toBeTruthy();
		});

		it('POST responds with a 400 invalid, when data incorrect - no body', async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			try {
				await testHelper.axios.post(url_email);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse('email address'));
			}
		});
	
		it('POST responds with a 400 invalid, when data incorrect - empty body', async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			try {
				await testHelper.axios.post(url_email, {});
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse('email address'));
			}
		});

		it('POST responds with a 400 invalid, when data incorrect - email address not array', async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			try {
				await testHelper.axios.post(url_email, { userAddress: testHelper.email, emailTitle: 'title', lineOne: 'lineOne' });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse('email address'));
			}
		});
		
		it('POST responds with a 400 invalid, when data incorrect - email address empty', async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			try {
				await testHelper.axios.post(url_email, { userAddress: [], emailTitle: 'title', lineOne: 'lineOne' });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse('email address'));
			}
		});

		it('POST responds with a 400 invalid, when data incorrect - invalid user', async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			try {
				await testHelper.axios.post(url_email, { userAddress: [ `${await testHelper.randomHex(5)}@${await testHelper.randomHex(5)}.com` ], emailTitle: 'title', lineOne: 'lineOne' });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual({ response: 'User not found' });
			}
		});

		it('POST responds with a 400 invalid, when data incorrect - no title', async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			try {
				await testHelper.axios.post(url_email, { userAddress: [ testHelper.email ], emailTitle: false, lineOne: 'lineOne' });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse('title required'));
			}
		});

		it('POST responds with a 400 invalid, when data incorrect - line one not string', async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			try {
				await testHelper.axios.post(url_email, { userAddress: [ testHelper.email ], emailTitle: 'emailTitle', lineOne: testHelper.randomBoolean() });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse('line one required'));
			}
		});

		it('POST responds with a 400 invalid, when data incorrect - line two not string', async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			try {
				await testHelper.axios.post(url_email, { userAddress: [ testHelper.email ], emailTitle: 'emailTitle', lineOne: 'lineOne', lineTwo: testHelper.randomBoolean() });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse('line two error'));
			}
		});
		
		it('POST responds with a 400 invalid, when data incorrect - invalid link address', async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			try {
				await testHelper.axios.post(url_email, { userAddress: [ testHelper.email ], emailTitle: 'emailTitle', lineOne: 'lineOne', lineTwo: 'lineTwo', link: 'link' });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse('link error'));
			}
		});

		it('POST responds with a 400 invalid, when data incorrect - button text invalid', async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			try {
				await testHelper.axios.post(url_email, { userAddress: [ testHelper.email ], emailTitle: 'emailTitle', lineOne: 'lineOne', lineTwo: 'lineTwo', link: 'https://www.google.com', button: testHelper.randomBoolean() });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse('button text error'));
			}
		});

		it('POST responds with a 200, email sent, emailCount increased', async () => {
			expect.assertions(3);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			const pre_emailCount = testHelper.mockedRabbitSendEmail.mock.calls.length;
			const result = await testHelper.axios.post(url_email, { userAddress: [ testHelper.email ], emailTitle: await testHelper.randomHex(10), lineOne: await testHelper.randomHex(10), lineTwo: await testHelper.randomHex(10), link: 'https://www.google.com', button: await testHelper.randomHex(10) });
			const post_emailCount = testHelper.mockedRabbitSendEmail.mock.calls.length;
			expect(result.status).toEqual(200);
			expect(result.data).toEqual(testHelper.response_empty);
			expect(pre_emailCount).not.toEqual(post_emailCount);
		});
	
	});

	describe(`ROUTE - ${url_error}`, () => {

		beforeEach(async () => testHelper.beforeEach());

		it('GET - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.get(url_error);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
	
		it('DELETE - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.get(url_error);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
			
		it('POST - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.get(url_error);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
			
		it('PATCH - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.get(url_error);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
			
		it('PUT - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.get(url_error);
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
				await testHelper.axios.get(url_error);
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
				await testHelper.axios.get(url_error);
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
				await testHelper.axios.get(url_error);
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
				await testHelper.axios.get(url_error);
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
				await testHelper.axios.get(url_error);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});

		it('GET responds array or errors', async () => {
			expect.assertions(1);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			const result = await testHelper.axios.get(url_error);
			expect(result.data.response).toBeDefined();
		});

	});

	describe(`ROUTE - ${url_memory}`, () => {

		beforeEach(async () => testHelper.beforeEach());

		it('GET - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.get(url_memory);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
	
		it('DELETE - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.get(url_memory);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
			
		it('POST - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.get(url_memory);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
			
		it('PATCH - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.get(url_memory);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
			
		it('PUT - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.get(url_memory);
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
				await testHelper.axios.get(url_memory);
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
				await testHelper.axios.get(url_memory);
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
				await testHelper.axios.get(url_memory);
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
				await testHelper.axios.get(url_memory);
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
				await testHelper.axios.get(url_memory);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});

		it('GET responds array of server details, matching number regex', async () => {
			expect.assertions(14);
			// Sleep for issues with serverUptime stats
			// await testHelper.sleep(1000);
			const pointRegex = /^\d+\.\d{2}$/;
			const numRegex = /^\d+$/;
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			const result = await testHelper.axios.get(url_memory);
			expect(result.status).toEqual(200);
			expect(result.data.response).toBeDefined();
			expect(result.data.response).toHaveProperty('rss');
			expect(result.data.response.rss.match(pointRegex)).toBeTruthy();
			expect(result.data.response).toHaveProperty('heapUsed');
			expect(result.data.response.heapUsed.match(pointRegex)).toBeTruthy();
			expect(result.data.response).toHaveProperty('heapTotal');
			expect(result.data.response.heapTotal.match(pointRegex)).toBeTruthy();
			expect(result.data.response).toHaveProperty('external');
			expect(result.data.response.external.match(pointRegex)).toBeTruthy();
			expect(result.data.response).toHaveProperty('nodeUptime');
			expect(String(result.data.response.nodeUptime).match(numRegex)).toBeTruthy();
			expect(result.data.response).toHaveProperty('serverUptime');
			expect(String(result.data.response.serverUptime).match(pointRegex)).toBeTruthy();
		});
	
	});

	describe(`ROUTE - ${url_restart}`, () => {

		beforeEach(async () => testHelper.beforeEach());

		it('GET - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.get(url_restart);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
	
		it('DELETE - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.get(url_restart);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
			
		it('POST - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.get(url_restart);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
			
		it('PATCH - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.get(url_restart);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
			
		it('PUT - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.get(url_restart);
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
				await testHelper.axios.get(url_restart);
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
				await testHelper.axios.get(url_restart);
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
				await testHelper.axios.get(url_restart);
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
				await testHelper.axios.get(url_restart);
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
				await testHelper.axios.get(url_restart);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});

		it('PUT - response 400 invalid user data - no body', async () => {
			expect.assertions(2);
			try {
				await testHelper.insertAdminUser();
				await testHelper.request_signin();
				await testHelper.axios.put(url_restart);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`passwords are required to be 10 characters minimum`));
			}
		});
		
		it('PUT - response 400 invalid user data - empty body', async () => {
			expect.assertions(2);
			try {
				await testHelper.insertAdminUser();
				await testHelper.request_signin();
				await testHelper.axios.put(url_restart, {});
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`passwords are required to be 10 characters minimum`));
			}
		});
			
		it('PUT - response 400 invalid user data - password too short', async () => {
			expect.assertions(2);
			try {
				await testHelper.insertAdminUser();
				await testHelper.request_signin();
				await testHelper.axios.put(url_restart, { password: await testHelper.randomHex(1), token: '000000' });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`passwords are required to be 10 characters minimum`));
			}
		});

		it('PUT - response 400 invalid user data - token format invalid', async () => {
			expect.assertions(2);
			try {
				await testHelper.insertAdminUser();
				await testHelper.request_signin();
				await testHelper.insert2FA();
				if (!testHelper.two_fa_secret) throw Error('!two_fa_secret');
				const token = testHelper.generateTokenFromString(testHelper.two_fa_secret);
				await testHelper.axios.put(url_restart, { password: await testHelper.password, token: token.substring(1,) });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`token format incorrect`));
			}
		});

		it('PUT - response 400 invalid user data - backup token format invalid', async () => {
			expect.assertions(2);
			try {
				await testHelper.insertAdminUser();
				await testHelper.request_signin();
				await testHelper.insert2FA();
				if (!testHelper.two_fa_secret) throw Error('!two_fa_secret');
				if (!testHelper.two_fa_secret) throw Error('!two_fa_secret');
				await testHelper.axios.put(url_restart, { password: testHelper.password, token: '000000', backupToken: `${testHelper.randomBoolean()}` });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`backupToken`));
			}
		});

		it('PUT - response 401 invalid password/token: wrong password', async () => {
			expect.assertions(2);
			try {
				await testHelper.insertAdminUser();
				await testHelper.request_signin();
				await testHelper.axios.put(url_restart, { password: await testHelper.randomHex(12), token: '000000' });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(401);
				expect(e.response?.data).toStrictEqual(testHelper.response_incorrectPasswordOrToken);
			}
		});

		it('PUT - response 401 invalid password/token: no token', async () => {
			expect.assertions(2);
			try {
				await testHelper.insertAdminUser();
				await testHelper.request_signin();
				await testHelper.axios.put(url_restart, { password: testHelper.password, });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`token format incorrect`));
			}
		});
		
		it('PUT - response 401 invalid password/token: token format invalid', async () => {
			expect.assertions(2);
			try {
				await testHelper.insertAdminUser();
				await testHelper.request_signin();
				await testHelper.axios.put(url_restart, { password: testHelper.password, token: await testHelper.randomHex(9) });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`token format incorrect`));
			}
		});

		it('PUT 200 empty response', async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			await testHelper.insert2FA();
			if (!testHelper.two_fa_secret) throw Error('!two_fa_secret');
			const result = await testHelper.axios.put(url_restart, { password: testHelper.password, token: testHelper.generateTokenFromString(testHelper.two_fa_secret) });
			expect(result.data).toEqual(testHelper.response_empty);
			expect(result.status).toEqual(200);
		});

	});

	describe(`ROUTE - ${url_limit}`, () => {

		const injectEnv = (): void => {
			process.env.limitTest = 'true';
		};
		
		beforeEach(async () => testHelper.beforeEach());

		afterEach(async () => {
			process.env.limitTest = null;
		});
		
		it('GET - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.get(url_limit);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
	
		it('DELETE - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.get(url_limit);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
			
		it('POST - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.get(url_limit);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
			
		it('PATCH - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.get(url_limit);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
			
		it('PUT - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.get(url_limit);
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
				await testHelper.axios.get(url_limit);
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
				await testHelper.axios.get(url_limit);
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
				await testHelper.axios.get(url_limit);
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
				await testHelper.axios.get(url_limit);
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
				await testHelper.axios.get(url_limit);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});

		it('GET responds with a 200, array of single user with rateLimit status', async () => {
			injectEnv();
			expect.assertions(2);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			const promiseArray = [];
			const randomRequestNumber = Math.floor(Math.random() * (15 - 3)) + 2;
			for (const _i of new Array(randomRequestNumber)) promiseArray.push(testHelper.axios.get(url_limit));
			await Promise.all(promiseArray);
			const result = await testHelper.axios.get(url_limit);
			expect(result.status).toEqual(200);
			expect(result.data.response.limits).toEqual(expect.arrayContaining([ { p: randomRequestNumber+1, u: testHelper.email, b: false }, { b: false, p: 4, u: testHelper.axios_ip } ]));
		});

		it('PATCH invalid data responds with a 400 invalid body - no body', async () => {
			expect.assertions(2);
			try {
				await testHelper.insertAdminUser();
				await testHelper.request_signin();
				await testHelper.axios.patch(url_limit);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`client`));
			}
		});

		it('PATCH invalid data responds with a 400 invalid body - empty body', async () => {
			expect.assertions(2);
			try {
				await testHelper.insertAdminUser();
				await testHelper.request_signin();
				await testHelper.axios.patch(url_limit, {});
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`client`));
			}
		});

		it('PATCH invalid data responds with a 400 invalid body - random boolean', async () => {
			expect.assertions(2);
			try {
				await testHelper.insertAdminUser();
				await testHelper.request_signin();
				await testHelper.axios.patch(url_limit, { client: testHelper.randomBoolean() });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`client`));
			}
		});

		it('PATCH invalid data responds with a 400 invalid body - random number', async () => {
			expect.assertions(2);
			try {
				await testHelper.insertAdminUser();
				await testHelper.request_signin();
				await testHelper.axios.patch(url_limit, { client: testHelper.randomNumber() });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`client`));
			}
		});

		it('PATCH invalid data responds with a 400 invalid body - random string', async () => {
			expect.assertions(2);
			try {
				await testHelper.insertAdminUser();
				await testHelper.request_signin();
				await testHelper.axios.patch(url_limit, { client: await testHelper.randomHex(12) });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`client`));
			}
		});

		it('PATCH invalid data responds with a 400 invalid body - invalid short ip address', async () => {
			expect.assertions(2);
			try {
				await testHelper.insertAdminUser();
				await testHelper.request_signin();
				await testHelper.axios.patch(url_limit, { client: '1.1.1' });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`client`));
			}
		});

		it('PATCH invalid data responds with a 400 invalid body - invalid ip address', async () => {
			expect.assertions(2);
			try {
				await testHelper.insertAdminUser();
				await testHelper.request_signin();
				await testHelper.axios.patch(url_limit, { client: '256.1.1.1' });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`client`));
			}
		});

		it('PATCH invalid data responds with a 400 invalid body - invalid ip address', async () => {
			expect.assertions(2);
			try {
				await testHelper.insertAdminUser();
				await testHelper.request_signin();
				await testHelper.axios.patch(url_limit, { client: '1.256.1.1' });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`client`));
			}
		});

		it('PATCH invalid data responds with a 400 invalid body - invalid ip address', async () => {
			expect.assertions(2);
			try {
				await testHelper.insertAdminUser();
				await testHelper.request_signin();
				await testHelper.axios.patch(url_limit, { client: '1.1.256.1' });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`client`));
			}
		});

		it('PATCH invalid data responds with a 400 invalid body - invalid ip address', async () => {
			expect.assertions(2);
			try {
				await testHelper.insertAdminUser();
				await testHelper.request_signin();
				await testHelper.axios.patch(url_limit, { client: '1.1.1.256' });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`client`));
			}
		});

		it('PATCH invalid data responds with a 400 invalid body - email missing end', async () => {
			expect.assertions(2);
			try {
				await testHelper.insertAdminUser();
				await testHelper.request_signin();
				const client = `${await testHelper.randomHex(5)}@${await testHelper.randomHex(5)}`;
				await testHelper.axios.patch(url_limit, { client });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`client`));
			}
		});

		it('PATCH invalid data responds with a 400 invalid body - email missing start', async () => {
			expect.assertions(2);
			try {
				await testHelper.insertAdminUser();
				await testHelper.request_signin();
				const client = `@${await testHelper.randomHex(5)}`;
				await testHelper.axios.patch(url_limit, { client });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`client`));
			}
		});

		it('PATCH invalid data responds with a 400 invalid body - email missing start', async () => {
			expect.assertions(2);
			try {
				await testHelper.insertAdminUser();
				await testHelper.request_signin();
				const client = `@${await testHelper.randomHex(5)}.com`;
				await testHelper.axios.patch(url_limit, { client });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`client`));
			}
		});

		it('PATCH removes all points, responds with a 200, array of single user with single point', async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			await testHelper.sleep(1000);
			
			injectEnv();
			const promiseArray = [];
			const randomRequestNumber = Math.floor(Math.random() * (15 - 9)) + 8;
			for (const _i of new Array(randomRequestNumber)) promiseArray.push(testHelper.axios.get(url_limit));
			await Promise.all(promiseArray);
			const result01 = await testHelper.axios.get(url_limit);
			await testHelper.axios.patch(url_limit, { client: testHelper.email });
			await testHelper.sleep(500);
			const result02 = await testHelper.axios.get(url_limit);
			const emailIndex01 = result01.data.response.limits.findIndex((i) => i.u === testHelper.email);
			const emailIndex02 = result02.data.response.limits.findIndex((i) => i.u === testHelper.email);
			if (emailIndex01 < 0) throw Error('!emailIndex');
			if (emailIndex02 < 0) throw Error('!emailIndex');

			expect(result01.data.response.limits[emailIndex01]).toEqual({ p: randomRequestNumber+1, u: testHelper.email, b: false });
			expect(result02.data.response.limits[emailIndex02]).toEqual({ p: 1, u: testHelper.email, b: false });
		});

	});
	
	describe(`ROUTE - ${url_user}`, () => {
		
		const userResponseKeys = [
			'firstName',
			'lastName',
			'email',
			'active',
			'timestamp',
			'user_creation_ip',
			'login_attempt_number',
			'password_reset_id',
			'reset_string',
			'passwordResetDate',
			'passwordResetCreationIp',
			'passwordResetConsumed',
			'login_ip',
			'loginSuccess',
			'login_date',
			'user_agent_string',
			'admin',
			'tfaSecret',
		];
		beforeEach(async () => testHelper.beforeEach());

		it('GET - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.get(url_user);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});

		it('DELETE - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.get(url_user);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
		
		it('POST - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.get(url_user);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
		
		it('PATCH - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.get(url_user);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
		
		it('PUT - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.get(url_user);
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
				await testHelper.axios.get(url_user);
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
				await testHelper.axios.get(url_user);
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
				await testHelper.axios.get(url_user);
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
				await testHelper.axios.get(url_user);
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
				await testHelper.axios.get(url_user);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});

		it('GET responds with a 200, array of users, jestUser match known variables, resetPassword and 2fa enabled', async () => {
			expect.assertions(40);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			await testHelper.insert2FA();
			await testHelper.axios.post(`/incognito/reset-password`, { email: testHelper.email });
			const result = await testHelper.axios.get(url_user);
			const jestIndex = result.data.response.findIndex((i: any) => i.email === testHelper.email);
			const jestUser = result.data.response[jestIndex];
			for (const i of userResponseKeys) expect(jestUser).toHaveProperty(i);
			expect(result.status).toEqual(200);
			const today = new Date(testHelper.generateToday());
			expect(result.data.response).toBeDefined();
			expect(result.data.response.length > 1).toBeTruthy();
			expect(jestUser.firstName).toEqual(testHelper.firstName);
			expect(jestUser.firstName).toEqual(testHelper.firstName);
			expect(jestUser.lastName).toEqual(testHelper.lastName);
			expect(jestUser.email).toEqual(testHelper.email);
			expect(jestUser.active).toEqual(true);
			expect(new Date(jestUser.timestamp) > today).toBeTruthy();
			expect(jestUser.user_creation_ip).toEqual(testHelper.axios_ip);
			expect(jestUser.login_attempt_number).toEqual(null);
			expect(isNaN(jestUser.password_reset_id)).toBeFalsy();
			expect(new Date(jestUser.passwordResetDate) > today).toBeTruthy();
			expect(jestUser.passwordResetCreationIp).toEqual(testHelper.axios_ip);
			expect(jestUser.reset_string).toMatch(testHelper.regex_passwordResetString);
			expect(jestUser.passwordResetConsumed).toEqual(false);
			expect(jestUser.login_ip).toEqual(testHelper.axios_ip);
			expect(jestUser.loginSuccess).toEqual(true);
			expect(new Date(jestUser.login_date) > today).toBeTruthy();
			expect(jestUser.user_agent_string).toEqual(testHelper.userAgent);
			expect(jestUser.admin).toEqual(true);
			expect(jestUser.tfaSecret).toEqual(true);
		});

		it('PATCH {active} on oneself responds 400', async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			try {
				await testHelper.axios.patch(url_user, { email: testHelper.email, patch: { active: false } });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.response_onself);
			}
		});

		it('PATCH {active} toggle user active status user, test toggle true and false', async () => {
			const anon_helper = new TestHelper();
			expect.assertions(12);
			await testHelper.insertAdminUser();
			await testHelper.insertAnonUser();
			await testHelper.request_signin();
			const signin01 = await anon_helper.axios.post(url_signin, { email: testHelper.email_anon, password: testHelper.password });
			const patch_disable = await testHelper.axios.patch(url_user, { email: testHelper.email_anon, patch: { active: false } });
			const disabled = await testHelper.axios.get(url_user);
			try {
				await anon_helper.axios.post(url_signin, { email: testHelper.email_anon, password: testHelper.password });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(401);
				expect(e.response?.data).toStrictEqual(testHelper.response_invalidLogin);
			}
			const patch_enable = await testHelper.axios.patch(url_user, { email: testHelper.email_anon, patch: { active: true } });
			const enabled = await testHelper.axios.get(url_user);
			const signin03 = await anon_helper.axios.post(url_signin, { email: testHelper.email_anon, password: testHelper.password });
			const jestIndex_disabled = disabled.data.response.findIndex((i: any) => i.email === testHelper.email_anon);
			const jestIndex_enabled = enabled.data.response.findIndex((i: any) => i.email === testHelper.email_anon);
			expect(signin01.status).toEqual(200);
			expect(signin01.data).toEqual(testHelper.response_empty);
			expect(patch_disable.status).toEqual(200);
			expect(patch_disable.data).toEqual(testHelper.response_empty);
			expect(patch_disable.status).toEqual(200);
			expect(patch_enable.data).toEqual(testHelper.response_empty);
			expect(signin03.status).toEqual(200);
			expect(signin03.data).toEqual(testHelper.response_empty);
			expect(disabled.data.response[jestIndex_disabled].active).toBeFalsy();
			expect(enabled.data.response[jestIndex_enabled].active).toBeTruthy();
		});

		it('PATCH {passwordResetId} toggle password invalid body responds 400: invalid email', async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			await testHelper.insertAnonUser();
			await testHelper.request_signin();
			const anon_helper = new TestHelper();
			await anon_helper.axios.post(`/incognito/reset-password`, { email: testHelper.email_anon });
			try {
				await testHelper.axios.patch(url_user, { email: `${await testHelper.randomHex(12)}@${await testHelper.randomHex(12)}`, patch: { passwordResetId: '1' } });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`email address`));
			}
		});

		// HERE
		it('PATCH {passwordResetId} toggle password invalid body responds 400: invalid resetId - letter', async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			await testHelper.insertAnonUser();
			await testHelper.request_signin();
			const anon_helper = new TestHelper();
			await anon_helper.axios.post(`/incognito/reset-password`, { email: testHelper.email_anon });
			try {
				await testHelper.axios.patch(url_user, { email: `${await testHelper.randomHex(12)}@${await testHelper.randomHex(12)}`, patch: { passwordResetId: 'a' } });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`passwordResetId`));
			}
		});

		it('PATCH {passwordResetId} toggle password invalid body responds 400: invalid resetId - null', async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			await testHelper.insertAnonUser();
			await testHelper.request_signin();
			const anon_helper = new TestHelper();
			await anon_helper.axios.post(`/incognito/reset-password`, { email: testHelper.email_anon });
			try {
				await testHelper.axios.patch(url_user, { email: testHelper.email_anon, patch: { passwordResetId: null } });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual({ response: 'Password reset invalid' });
			}
		});
		
		it('PATCH {passwordResetId} toggle password invalid body responds 400: invalid resetId - boolean', async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			await testHelper.insertAnonUser();
			await testHelper.request_signin();
			const anon_helper = new TestHelper();
			await anon_helper.axios.post(`/incognito/reset-password`, { email: testHelper.email_anon });
			try {
				await testHelper.axios.patch(url_user, { email: testHelper.email_anon, patch: { passwordResetId: testHelper.randomBoolean() } });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`passwordResetId`));
			}
		});

		it('PATCH {passwordResetId} toggle password invalid body responds 400: invalid resetId - number', async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			await testHelper.insertAnonUser();
			await testHelper.request_signin();
			const anon_helper = new TestHelper();
			await anon_helper.axios.post(`/incognito/reset-password`, { email: testHelper.email_anon });
			try {
				await testHelper.axios.patch(url_user, { email: testHelper.email_anon, patch: { passwordResetId: testHelper.randomNumber() } });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse(`passwordResetId`));
			}
		});

		it('PATCH {reset} toggle password reset on, without email sent', async () => {
			expect.assertions(11);
			await testHelper.insertAdminUser();
			await testHelper.insertAnonUser();
			await testHelper.request_signin();
			const now = new Date();
			const pre_count = testHelper.mockedRabbitSendEmail.mock.calls.length;
			const pre_get = await testHelper.axios.get(url_user);
			const pre_index = pre_get.data.response.findIndex((i: any) => i.email === testHelper.email_anon);
			const generate_password_reset_request = await testHelper.axios.patch(url_user, { email: testHelper.email_anon, patch: { reset: { withEmail: false } } });
			const post_get = await testHelper.axios.get(url_user);
			const post_index = post_get.data.response.findIndex((i: any) => i.email === testHelper.email_anon);
			const post_count = testHelper.mockedRabbitSendEmail.mock.calls.length;
			expect(pre_get.data.response[pre_index].reset_string).toBeFalsy();
			expect(pre_get.data.response[pre_index].passwordResetDate).toBeFalsy();
			expect(pre_get.data.response[pre_index].passwordResetCreationIp).toBeFalsy();
			expect(pre_get.data.response[pre_index].passwordResetId).toBeFalsy();
			expect(generate_password_reset_request.status).toEqual(200);
			expect(generate_password_reset_request.data).toEqual(testHelper.response_empty);
			expect(post_get.data.response[post_index].reset_string).toMatch(testHelper.regex_passwordResetString);
			expect(post_get.data.response[post_index].passwordResetDate).toBeTruthy();
			expect(new Date(post_get.data.response[post_index].passwordResetDate)> now).toBeTruthy();
			expect(post_get.data.response[post_index].passwordResetCreationIp).toEqual(testHelper.axios_ip);
			expect(pre_count).toEqual(post_count);
		});

		it('PATCH {reset} on oneself responds 400', async () => {
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			try {
				await testHelper.axios.patch(url_user, { email: testHelper.email, patch: { reset: { withEmail: true } } });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.response_onself);
			}
		});

		it('PATCH {reset} toggle password reset on, with email sent', async () => {
			expect.assertions(10);
			await testHelper.insertAdminUser();
			await testHelper.insertAnonUser();
			await testHelper.request_signin();
			const pre_count = testHelper.mockedRabbitSendEmail.mock.calls.length;
			const now = new Date();
			const pre_get = await testHelper.axios.get(url_user);
			const pre_index = pre_get.data.response.findIndex((i: any) => i.email === testHelper.email_anon);
			const generate_password_reset_request = await testHelper.axios.patch(url_user, { email: testHelper.email_anon, patch: { reset: { withEmail: true } } });
			const post_get = await testHelper.axios.get(url_user);
			const post_index = post_get.data.response.findIndex((i: any) => i.email === testHelper.email_anon);
			const post_count = testHelper.mockedRabbitSendEmail.mock.calls.length;
			expect(pre_get.data.response[pre_index].reset_string).toBeFalsy();
			expect(pre_get.data.response[pre_index].passwordResetDate).toBeFalsy();
			expect(pre_get.data.response[pre_index].passwordResetCreationIp).toBeFalsy();
			expect(generate_password_reset_request.status).toEqual(200);
			expect(generate_password_reset_request.data).toEqual(testHelper.response_empty);
			expect(post_get.data.response[post_index].reset_string).toMatch(testHelper.regex_passwordResetString);
			expect(post_get.data.response[post_index].passwordResetDate).toBeTruthy();
			expect(new Date(post_get.data.response[post_index].passwordResetDate)> now).toBeTruthy();
			expect(post_get.data.response[post_index].passwordResetCreationIp).toEqual(testHelper.axios_ip);
			expect(pre_count + 1 === post_count).toBeTruthy();
		});

		it('PATCH {reset} toggle password reset off', async () => {
			expect.assertions(10);
			await testHelper.insertAdminUser();
			await testHelper.insertAnonUser();
			await testHelper.request_signin();
			const now = new Date();
			const anon_helper = new TestHelper();
			await anon_helper.axios.post(`/incognito/reset-password`, { email: testHelper.email_anon });
			const pre_get = await testHelper.axios.get(url_user);
			const pre_index = pre_get.data.response.findIndex((i: any) => i.email === testHelper.email_anon);
			const toggle_off_request = await testHelper.axios.patch(url_user, { email: testHelper.email_anon, patch: { passwordResetId: pre_get.data.response[pre_index].password_reset_id } });
			const post_get = await testHelper.axios.get(url_user);
			const post_index = post_get.data.response.findIndex((i: any) => i.email === testHelper.email_anon);
			expect(toggle_off_request.status).toEqual(200);
			expect(toggle_off_request.data).toEqual(testHelper.response_empty);
			expect(pre_get.data.response[pre_index].reset_string).toMatch(testHelper.regex_passwordResetString);
			expect(pre_get.data.response[pre_index].passwordResetDate).toBeTruthy();
			expect(new Date(pre_get.data.response[pre_index].passwordResetDate)> now).toBeTruthy();
			expect(pre_get.data.response[pre_index].passwordResetCreationIp).toEqual(testHelper.axios_ip);
			expect(post_get.data.response[post_index].reset_string).toEqual(null);
			expect(post_get.data.response[post_index].passwordResetDate).toEqual(null);
			expect(post_get.data.response[post_index].passwordResetCreationIp).toEqual(null);
			expect(post_get.data.response[post_index].passwordResetConsumed).toEqual(null);
		});

		it('PATCH {attempt} reset login attempt number', async () => {
			expect.assertions(24);
			await testHelper.insertAdminUser();
			await testHelper.insertAnonUser();
			await testHelper.request_signin();
			for (const _i of new Array(8)) {
				try {
					// eslint-disable-next-line no-await-in-loop
					await testHelper.request_signin({ body: { email: testHelper.email_anon, password: 'testHelper.password' } });
				} catch (err) {
					const e = testHelper.returnAxiosError(err);
					expect(e.response?.data).toEqual(testHelper.response_incorrectPasswordOrToken);
					expect(e.response?.status).toEqual(401);
				}
			}
			const pre_get = await testHelper.axios.get(url_user);
			const pre_index = pre_get.data.response.findIndex((i: any) => i.email === testHelper.email_anon);
			const toggle_off_request = await testHelper.axios.patch(url_user, { email: testHelper.email_anon, patch: { attempt: true } });
			const post_get = await testHelper.axios.get(url_user);
			const post_index = post_get.data.response.findIndex((i: any) => i.email === testHelper.email_anon);
			expect(pre_get.data.response[pre_index].login_ip).toEqual(testHelper.axios_ip);
			expect(pre_get.data.response[pre_index].login_attempt_number).toEqual('8');
			expect(pre_get.data.response[pre_index].loginSuccess).toEqual(false);
			expect(toggle_off_request.data).toEqual(testHelper.response_empty);
			expect(toggle_off_request.status).toEqual(200);
			expect(post_get.data.response[post_index].login_ip).toEqual(testHelper.axios_ip);
			expect(post_get.data.response[post_index].login_attempt_number).toEqual('0');
			expect(post_get.data.response[post_index].loginSuccess).toEqual(false);
		});

		it('PATCH {2fa} on oneself responds 400', async () => {
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			try {
				await testHelper.axios.patch(url_user, { email: testHelper.email, patch: { tfaSecret: false } });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.data).toEqual(testHelper.response_onself);
				expect(e.response?.status).toEqual(400);
			}
		});

		it('PATCH {2fa} disabled 2fa', async () => {
			expect.assertions(8);
			await testHelper.insertAdminUser();
			await testHelper.insertAnonUser();
			await testHelper.insertAnon2FA();
			await testHelper.request_signin();
			const pre_get = await testHelper.axios.get(url_user);
			const pre_index = pre_get.data.response.findIndex((i: any) => i.email === testHelper.email_anon);
			const signin01 = await testHelper.request_signin({ body: { email: testHelper.email_anon, password: testHelper.password } });
			const toggle_off_request = await testHelper.axios.patch(url_user, { email: testHelper.email_anon, patch: { tfaSecret: false } });
			const post_get = await testHelper.axios.get(url_user);
			const post_index = post_get.data.response.findIndex((i: any) => i.email === testHelper.email_anon);
			const signin02 = await testHelper.request_signin({ body: { email: testHelper.email_anon, password: testHelper.password } });
			expect(pre_get.data.response[pre_index].tfaSecret).toBeTruthy();
			expect(signin01.status).toEqual(202);
			expect(signin01.data).toEqual(testHelper.response_empty);
			expect(toggle_off_request.data).toEqual(testHelper.response_empty);
			expect(toggle_off_request.status).toEqual(200);
			expect(signin02.status).toEqual(200);
			expect(signin02.data).toEqual(testHelper.response_empty);
			expect(post_get.data.response[post_index].tfaSecret).toBeFalsy();
		});
	
	});

	describe(`ROUTE - ${url_session}`, () => {

		const sessionKeyNames = [
			'originalMaxAge',
			'expires',
			'secure',
			'httpOnly',
			'domain',
			'path',
			'sameSite',
			'sessionKey',
			'userAgent',
			'ip'
		];
	
		beforeEach(async () => testHelper.beforeEach());

		it('GET - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.get(url_session);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
	
		it('DELETE - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.delete(url_session);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
	
		it('POST - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.post(url_session);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
	
		it('PATCH - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
			
				await testHelper.axios.patch(url_session);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
	
		it('PUT - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.insertUser();
				await testHelper.request_signin();
				await testHelper.axios.put(url_session);
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
				await testHelper.axios.get(url_session);
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
				await testHelper.axios.delete(url_session);
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
				await testHelper.axios.post(url_session);
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
				await testHelper.axios.patch(url_session);
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
				await testHelper.axios.put(url_session);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});

		it(`GET /:email responds 400 invalid userdata when email is invalid - missing start`, async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			try {
				await testHelper.axios.get(`${url_session}/${await testHelper.randomHex(10)}.com`);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse('email address'));
			}
		});

		it(`GET /:email responds 400 invalid userdata when email is invalid - missing end`, async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			try {
				await testHelper.axios.get(`${url_session}/${await testHelper.randomHex(10)}@${await testHelper.randomHex(10)}`);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse('email address'));
			}
		});
		
		it(`GET /:email responds 400 invalid userdata when email is invalid - missing end`, async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			try {
				await testHelper.axios.get(`${url_session}/@${await testHelper.randomHex(10)}`);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse('email address'));
			}
		});
		
		it(`GET /:email responds 400 invalid userdata when email is invalid - missing end`, async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			try {
				await testHelper.axios.get(`${url_session}/@${await testHelper.randomHex(10)}.com`);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse('email address'));
			}
		});
		
		it(`GET /:email responds 400 invalid userdata when email is invalid - missing end`, async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			try {
				await testHelper.axios.get(`${url_session}/${await testHelper.randomHex(10)}`);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.generateBadResponse('email address'));
			}
		});

		it(`GET /:email responds 400 invalid userdata when email is unkown user`, async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			try {
				await testHelper.axios.get(`${url_session}/${testHelper.email_anon}`);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual({ response: 'User not found' });
			}
		});

		it(`GET /:email responds 200 array of session for anon and admin user`, async () => {
			expect.assertions(25);
			await testHelper.insertAdminUser();
			await testHelper.insertAnonUser();
			await testHelper.request_signin();
			const anon_helper = new TestHelper();
			await anon_helper.axios.post(url_signin, { email: testHelper.email_anon, password: testHelper.password });
			const anonSession = await testHelper.axios.get(`${url_session}/${testHelper.email_anon}`);
			const adminSession = await testHelper.axios.get(`${url_session}/${testHelper.email}`);
			expect(anonSession.status).toEqual(200);
			expect(anonSession.data.response).toBeDefined();
			expect(adminSession.status).toEqual(200);
			expect(adminSession.data.response).toBeDefined();
			for (const i of sessionKeyNames) expect(anonSession.data.response[0]).toHaveProperty(i);
			for (const i of sessionKeyNames) expect(adminSession.data.response[0]).toHaveProperty(i);
			expect(adminSession.data.response[0].currentSession).toEqual(true);
		});

		it('DELETE responds 400 oneself when trying to delete current session ', async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			await testHelper.insertAnonUser();
			await testHelper.request_signin();
			const keys = await testHelper.redis.keys(`session:*`);
			if (!keys) throw Error('!keys');
			const sessionId = keys[keys.findIndex((i:string) => testHelper.regex_session.test(i) === true)];
			try {
				await testHelper.axios.delete(url_session, { data: { session: sessionId } });
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual(testHelper.response_onself);
			}
		});

		it('DELETE responds 200, correctly removes anon user session ', async () => {
			expect.assertions(6);
			await testHelper.insertAdminUser();
			await testHelper.insertAnonUser();
			await testHelper.request_signin();
			const anon_helper = new TestHelper();
			await anon_helper.axios.post(url_signin, { email: testHelper.email_anon, password: testHelper.password });
			const anonSession = await testHelper.axios.get(`${url_session}/${testHelper.email_anon}`);
			const pre_session = await testHelper.redis.get(anonSession.data.response[0].sessionKey);
			const result = await testHelper.axios.delete(url_session, { data: { session: anonSession.data.response[0].sessionKey } });
			const post_session = await testHelper.redis.get(anonSession.data.response[0].sessionKey);
			expect(anonSession.status).toEqual(200);
			expect(anonSession.data.response[0]).toBeDefined();
			expect(pre_session).toBeTruthy();
			expect(result.status).toEqual(200);
			expect(result.data).toEqual(testHelper.response_empty);
			expect(post_session).toBeFalsy();
		});

	});

});