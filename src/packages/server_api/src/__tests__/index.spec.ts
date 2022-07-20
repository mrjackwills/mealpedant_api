import { TestHelper } from './testHelper';
import format from 'pg-format';

import { afterAll, beforeAll, beforeEach, describe, expect, it } from 'vitest';

const testHelper = new TestHelper();

const random = Date.now();

describe('Test api init', () => {

	beforeAll(async () => testHelper.beforeAll());

	beforeEach(async () => testHelper.beforeEach());
	
	afterAll(async () => testHelper.afterAll());

	describe(`Valid postgres connection`, () => {
		it('should have name dev_mealpedant', async () => {
			expect.assertions(1);
			const query = format(`SELECT current_database()`);
			const { rows } = await testHelper.postgres.query(query);
			expect(rows[0]).toEqual({ current_database: 'dev_mealpedant' });
		});
		it('should return size of mealpedant db > 14mb', async () => {
			expect.assertions(1);
			const query = format(`SELECT pg_database_size(current_database()) as size;`);
			const { rows } = await testHelper.postgres.query(query);
			expect(Number(rows[0].size)).toBeGreaterThan(14000000);
		});
	});

	describe(`Valid redis connection`, () => {
		it('return PONG from a PING command', async () => {
			expect.assertions(1);
			const a = await testHelper.redis.ping();
			expect(a).toEqual('PONG');

		});
	});

	describe(`ROUTE - ${testHelper.VMajor}/${random}`, () => {
		it('GET should return Unkown endpoint json status 404', async () => {
			expect.assertions(2);
			try {
				await testHelper.axios.get(`/${random}}`);
			} catch (err) {
				const e = testHelper.returnAxiosError(err);
				expect(e.response?.status).toStrictEqual(404);
				expect(e.response?.data).toStrictEqual(testHelper.response_unknown);
			}
		});
	});
	
});