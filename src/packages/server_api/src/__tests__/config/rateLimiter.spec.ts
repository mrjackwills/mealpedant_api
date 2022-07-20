/* eslint-disable no-await-in-loop, @typescript-eslint/ban-types, @typescript-eslint/no-explicit-any */
import { TestHelper } from '../testHelper';
import { afterAll, beforeAll, beforeEach, describe, expect, it } from 'vitest';

const testHelper = new TestHelper();
const foodBaseRoute = `/authenticated/food/last`;

describe('RateLimit using apiHelper', () => {
	
	const injectEnv = (): void => {
		process.env.limitTest = 'true';
	};
	
	const multipleRequest = async (numberOfRequests: number): Promise<void> => {
		for (const _i of new Array(numberOfRequests)) {
			await testHelper.sleep(2);
			try {
				await testHelper.axios.get(foodBaseRoute);
			}
			catch (e) {
				// nothing
			}
		}
	};

	beforeAll(async () => testHelper.beforeAll());

	beforeEach(async () : Promise<void> => {
		process.env.limitTest = undefined;
		await testHelper.beforeEach();
	});
	
	afterAll(async () => testHelper.afterAll());

	it('Not authed, 429 response, 60 second block, after 30 requests', async () => {
		expect.assertions(3);
		injectEnv();
		await multipleRequest(30);
		try {
			await testHelper.axios.get(foodBaseRoute);
		} catch (err) {
			const e = testHelper.returnAxiosError(err);
			expect(e.response?.status).toStrictEqual(429);
			expect(isNaN(e.response?.data.response)).toBeFalsy();
			expect(Number(e.response?.data.response) === 60000).toBeTruthy();
		}
	});

	it('Not authed, 429 response, 24 hour block, after 60 requests', async () => {
		expect.assertions(3);
		injectEnv();
		await multipleRequest(60);
		try {
			await testHelper.axios.get(foodBaseRoute);
		} catch (err) {
			const e = testHelper.returnAxiosError(err);
			expect(e.response?.status).toStrictEqual(429);
			expect(isNaN(e.response?.data.response)).toBeFalsy();
			expect(Number(e.response?.data.response) >= 86399990 && Number(e.response?.data.response) <= 86401000).toBeTruthy();
		}

	});

	it('Not authed, 429 response, 3 day block, after 120 requests', async () => {
		expect.assertions(3);
		injectEnv();
		await multipleRequest(120);
		try {
			await testHelper.axios.get(foodBaseRoute);
		} catch (err) {
			const e = testHelper.returnAxiosError(err);
			expect(e.response?.status).toStrictEqual(429);
			expect(isNaN(e.response?.data.response)).toBeFalsy();
			expect(Number(e.response?.data.response) >= 259190000 && Number(e.response?.data.response) <= 259200000).toBeTruthy();
		}

	});

	it('Not authed, 429 response, 7 day block, after 240 requests', async () => {
		expect.assertions(3);
		injectEnv();
		await multipleRequest(240);
		try {
			await testHelper.axios.get(foodBaseRoute);
		} catch (err) {
			const e = testHelper.returnAxiosError(err);
			expect(e.response?.status).toStrictEqual(429);
			expect(isNaN(e.response?.data.response)).toBeFalsy();
			expect(Number(e.response?.data.response) >= 604795000 && Number(e.response?.data.response) <= 604800000).toBeTruthy();
		}
	});

});