import format from 'pg-format';
import { TestHelper } from './testHelper';
import { afterAll, describe, expect, it } from 'vitest';

const testHelper = new TestHelper();

describe('Test api init', () => {

	afterAll(async () => testHelper.afterAll());

	describe(`Valid postgres connection`, () => {

		it('should have name dev_mealpedant', async () => {
			expect.assertions(1);
			const query = format(`SELECT current_database()`);
			const { rows } = await testHelper.postgres.query(query);
			expect(rows[0]).toEqual({ current_database: 'dev_mealpedant' });
		});

	});

});