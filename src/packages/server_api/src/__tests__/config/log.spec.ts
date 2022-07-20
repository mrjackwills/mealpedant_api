import { log } from '../../config/log';
import { TestHelper } from '../testHelper';

import { afterAll, beforeEach, describe, expect, it } from 'vitest';

const testHelper = new TestHelper();

describe('log test runner', () => {

	beforeEach(async () => testHelper.cleanDB());
	afterAll(() => testHelper.afterAll());

	it('Expect verbose log to increase in db', async () => {
		expect.assertions(1);
		const pre = await testHelper.query_selectErrorCount('verbose');
		log.verbose(testHelper.logErrorMessage);
		await testHelper.sleep();
		const post = await testHelper.query_selectErrorCount('verbose');
		expect(post - pre).toEqual(1);
	});

	it('Expect warn log to increase in db', async () => {
		expect.assertions(1);
		const pre = await testHelper.query_selectErrorCount('warn');
		log.warn(testHelper.logErrorMessage);
		await testHelper.sleep();
		const post = await testHelper.query_selectErrorCount('warn');
		expect(post - pre).toEqual(1);
	});

	it('Expect error log to increase in db', async () => {
		expect.assertions(1);
		const pre = await testHelper.query_selectErrorCount('error');
		log.error(testHelper.logErrorMessage);
		await testHelper.sleep();
		const post = await testHelper.query_selectErrorCount('error');
		expect(post - pre).toEqual(1);
	});

	it('Expect error log to increase in db', async () => {
		expect.assertions(8);
		const randomError = `${Date.now()}`;
		const now = Date.now();
		try {
			throw Error(randomError);
		} catch (e) {
			log.error(e);
		}
		// Sleep due to the way logging is handled
		await testHelper.sleep(100);
		const selectResult = await testHelper.query_selectError(randomError);
		expect(selectResult).toBeDefined();
		expect(isNaN(Number(selectResult.error_log_id))).toBeFalsy();
		expect(Number(selectResult.error_log_id)).toBeGreaterThan(1);
		expect(selectResult.level).toEqual('error');
		expect(selectResult.message).toEqual(randomError);
		expect(selectResult.uuid).toBeNull();
		expect(selectResult.http_code).toBeNull();
		expect(now - selectResult.timestamp.getTime()).toBeLessThan(50);
	});
});