import { TestHelper } from '../testHelper';
import { promises as fs } from 'fs';
import { LOCATION_PHOTO_CONVERTED, LOCATION_PHOTO_ORIGINAL } from '../../config/env';
import FormData from 'form-data';

import { afterAll, beforeAll, beforeEach, describe, expect, it } from 'vitest';

const testHelper = new TestHelper();
const url_photo = `/admin/photo`;

const removePhotos = async (c: string, o: string): Promise<void> => {
	try {
		await Promise.all([
			fs.unlink(`${LOCATION_PHOTO_CONVERTED}/${c}`),
			fs.unlink(`${LOCATION_PHOTO_ORIGINAL}/${o}`)
		]);
	} catch (e) {
		// eslint-disable-next-line no-console
		console.log(e);
	}
};

describe('Incognito test runner', () => {

	beforeAll(async () => testHelper.beforeAll());

	beforeEach(async () => testHelper.beforeEach());

	afterAll(async () => testHelper.afterAll());

	describe(`ROUTE - ${url_photo}`, () => {

		it('GET - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.get(url_photo);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
		
		it('DELETE - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.delete(url_photo);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
		
		it('POST - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.post(url_photo);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});
		
		it('PATCH - should return unauthorized 403 when not signed in', async () => {
			expect.assertions(2);
			try {
				
				await testHelper.axios.patch(url_photo);
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
				await testHelper.axios.put(url_photo);
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
				await testHelper.axios.get(url_photo);
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
				await testHelper.axios.delete(url_photo);
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
				await testHelper.axios.post(url_photo);
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
				await testHelper.axios.patch(url_photo);
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
				await testHelper.axios.put(url_photo);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(403);
				expect(e.response?.data).toStrictEqual(testHelper.response_unauthorized);
			}
		});

		it('POST with no data responds 400 no photo uploaded', async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			try {
				await testHelper.axios.post(url_photo);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual({ response: 'No photo uploaded' });
			}
		});

		it('POST fails when file > 10mb', async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			const largeFile = Buffer.alloc(11111111);
			try {
				const data = new FormData();
				const fileName = `${testHelper.generateToday()}_${testHelper.randomPerson().substring(0, 1)}.jpg`;
				data.append('image', largeFile, fileName);
				await testHelper.uploadAxios.post(url_photo, data, {
					headers: {
						...data.getHeaders(),
					}
				});
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(413);
				expect(e.response?.data).toStrictEqual({ response: 'File size too large' });
			}
			
		});

		it('POST fails when filetype not jpg', async () => {
			expect.assertions(2);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			try {
				const largeFile = Buffer.alloc(1111111);
				const data = new FormData();
				const fileName = `${testHelper.generateToday()}_${testHelper.randomPerson().substring(0, 1)}.gif`;
				data.append('image', largeFile, fileName);
				await testHelper.uploadAxios.post(url_photo, data, {
					headers: {
						...data.getHeaders(),
					}
				});
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(400);
				expect(e.response?.data).toStrictEqual({ response: 'File uploaded is not an image' });
			}
			
		});

		it('POST correctly uploads photo, responds photoNames, exists on disk', async () => {
			expect.assertions(9);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			const file = Buffer.alloc(38352);
			const data = new FormData();
			const fileName = `${testHelper.generateToday()}_${testHelper.randomPerson().substring(0, 1)}.jpg`;
			data.append('image', file, fileName);
			const result = await testHelper.uploadAxios.post(url_photo, data, {
				headers: {
					...data.getHeaders(),
				}
			});
			const c = result.data.response.c;
			const o = result.data.response.o;
			const [ converted, original ] = await Promise.all([
				fs.stat(`${LOCATION_PHOTO_CONVERTED}/${c}`),
				fs.stat(`${LOCATION_PHOTO_ORIGINAL}/${o}`)
			]);
			expect(result.data.response).toBeDefined();
			expect(result.data.response).toHaveProperty('o');
			expect(result.data.response).toHaveProperty('c');
			expect(c).toMatch(testHelper.regex_photoConverted);
			expect(o).toMatch(testHelper.regex_photoOriginal);
			expect(converted).toBeTruthy();
			expect(converted.size).toEqual(16339);
			expect(original).toBeTruthy();
			expect(original.size).toEqual(38352);
			await removePhotos(c, o);
		});

		it('DELETE responds 200 empty, files removed from disk', async () => {
			expect.assertions(10);
			await testHelper.insertAdminUser();
			await testHelper.request_signin();
			const file = Buffer.alloc(38352);
			const data = new FormData();
			const fileName = `${testHelper.generateToday()}_${testHelper.randomPerson().substring(0, 1)}.jpg`;
			data.append('image', file, fileName);
			const uploadRequest = await testHelper.uploadAxios.post(url_photo, data, {
				headers: {
					...data.getHeaders(),
				}
			});
			const c = uploadRequest.data.response.c;
			const o = uploadRequest.data.response.o;
			const [ pre_converted, pre_original ] = await Promise.all([
				fs.stat(`${LOCATION_PHOTO_CONVERTED}/${c}`),
				fs.stat(`${LOCATION_PHOTO_ORIGINAL}/${o}`)
			]);
			const deleteRequest = await testHelper.axios.delete(url_photo, { data: { c, o } });

			try {
				await fs.stat(`${LOCATION_PHOTO_CONVERTED}/${c}`);
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			} catch (e: any) {
				expect(e.message.startsWith('ENOENT: no such file or directory')).toBeTruthy();
				expect(e).toBeTruthy();
			}

			try {
				await fs.stat(`${LOCATION_PHOTO_ORIGINAL}/${o}`);
				// eslint-disable-next-line @typescript-eslint/no-explicit-any
			} catch (e: any) {
				expect(e.message.startsWith('ENOENT: no such file or directory')).toBeTruthy();
				expect(e).toBeTruthy();
			}
			expect(c).toMatch(testHelper.regex_photoConverted);
			expect(o).toMatch(testHelper.regex_photoOriginal);
			expect(pre_converted).toBeTruthy();
			expect(pre_original).toBeTruthy();
			expect(deleteRequest.status).toEqual(200);
			expect(deleteRequest.data).toEqual(testHelper.response_empty);
		});

	});
});
