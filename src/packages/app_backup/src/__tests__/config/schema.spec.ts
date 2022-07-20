import { validateInput } from '../../lib/validateInput';
import { schema } from '../../config/schema';
import { TestHelper } from '../testHelper';

import { afterAll, beforeAll, describe, expect, it } from 'vitest';

const testHelper = new TestHelper();

describe('schema tests', () => {

	const ping = 'ping';
	const full ='backup::full_backup';
	const sql = 'backup::sql_backup';

	afterAll(async () => testHelper.afterAll());

	beforeAll(async () => testHelper.beforeEach());

	describe(`FULL schema`, () => {

		it.concurrent('should resolve when message_name is only key present', async () => {
			expect.assertions(1);
			const message_name = full;
			let result = null;
			try {
				validateInput({ message_name }, schema.full);
			} catch (e) {
				if (e instanceof Error) result = e.message;
			}
			expect(result).toBeFalsy();
		});

		it('should throw when message_name invalid', async () => {
			expect.assertions(1);
			const message_name = await testHelper.randomHex(8);
			let result = null;
			try {
				validateInput({ message_name }, schema.full);
			} catch (e) {
				if (e instanceof Error) result = e.message;
			}
			expect(result).toBeTruthy();
		});

		it.concurrent('should throw when message_name missing', async () => {
			expect.assertions(1);
			let result = null;
			try {
				validateInput({ }, schema.full);
			} catch (e) {
				if (e instanceof Error) result = e.message;
			}
			expect(result).toBeTruthy();
		});

		it.concurrent('should throw when message_name key name invalid', async () => {
			expect.assertions(1);
			const messagename = full;
			let result = null;
			try {
				validateInput({ messagename }, schema.full);
			} catch (e) {
				if (e instanceof Error) result = e.message;
			}
			expect(result).toBeTruthy();
		});

		it('should throw when extra key in data', async () => {
			expect.assertions(1);
			const message_name = full;
			const random = await testHelper.randomHex();
			let result = null;
			try {
				validateInput({ message_name, random }, schema.full);
			} catch (e) {
				if (e instanceof Error) result = e.message;
			}
			expect(result).toBeTruthy();
		});
	
	});

	describe(`SQL_ONLY schema`, () => {

		it.concurrent('should resolve when message_name is only key present', async () => {
			expect.assertions(1);
			const message_name = sql;
			let result = null;
			try {
				validateInput({ message_name }, schema.sqlOnly);
			} catch (e) {
				if (e instanceof Error) result = e.message;
			}
			expect(result).toBeFalsy();
		});

		it('should throw when message_name invalid', async () => {
			expect.assertions(1);
			const message_name = await testHelper.randomHex(8);
			let result = null;
			try {
				validateInput({ message_name }, schema.sqlOnly);
			} catch (e) {
				if (e instanceof Error) result = e.message;
			}
			expect(result).toBeTruthy();
		});

		it.concurrent('should throw when message_name missing', async () => {
			expect.assertions(1);
			let result = null;
			try {
				validateInput({ }, schema.sqlOnly);
			} catch (e) {
				if (e instanceof Error) result = e.message;
			}
			expect(result).toBeTruthy();
		});

		it.concurrent('should throw when message_name key name invalid', async () => {
			expect.assertions(1);
			const messagename = sql;
			let result = null;
			try {
				validateInput({ messagename }, schema.sqlOnly);
			} catch (e) {
				if (e instanceof Error) result = e.message;
			}
			expect(result).toBeTruthy();
		});

		it('should throw when extra key in data', async () => {
			expect.assertions(1);
			const message_name = sql;
			const random = await testHelper.randomHex();
			let result = null;
			try {
				validateInput({ message_name, random }, schema.sqlOnly);
			} catch (e) {
				if (e instanceof Error) result = e.message;
			}
			expect(result).toBeTruthy();
		});
	
	});

	describe(`PING schema`, () => {

		it.concurrent('should resolve when message_name and data.attempt & data.password correct', async () => {
			expect.assertions(1);
			const message_name = ping;
			
			let result = null;
			try {
				validateInput({ message_name }, schema.ping);
			} catch (e) {
				if (e instanceof Error) result = e.message;
			}
			expect(result).toBeFalsy();
		});

		it('should throw when message_name invalid', async () => {
			expect.assertions(1);
			const message_name = await testHelper.randomHex(8);
			let result = null;
			try {
				validateInput({ message_name }, schema.ping);

			} catch (e) {
				if (e instanceof Error) result = e.message;
			}
			expect(result).toBeTruthy();
		});

		it.concurrent('should throw when message_name missing', async () => {
			expect.assertions(1);
			let result = null;
			try {
				validateInput({ }, schema.ping);

			} catch (e) {
				if (e instanceof Error) result = e.message;
			}
			expect(result).toBeTruthy();
		});

		it.concurrent('should throw when message_name key name invalid', async () => {
			expect.assertions(1);
			const messagename = ping;
			let result = null;
			try {
				validateInput({ messagename }, schema.ping);

			} catch (e) {
				if (e instanceof Error) result = e.message;
			}
			expect(result).toBeTruthy();
		});
		
		it('should throw when data provided', async () => {
			expect.assertions(1);
			const message_name = ping;
			const random = await testHelper.randomHex();
			let result = null;
			const data = {
				[random]: random
			};
			try {
				validateInput({ message_name, data }, schema.ping);
			} catch (e) {
				if (e instanceof Error) result = e.message;
			}
			expect(result).toBeTruthy();
		});
	});

	describe(`message_name schema`, () => {

		it.concurrent('should resolve when message_name is either createHash or validateHash', async () => {
			expect.assertions(1);
			const message_name = testHelper.randomMessageName;
			let result = null;
			try {
				validateInput(message_name, schema.message_name);
			} catch (e) {
				if (e instanceof Error) result = e.message;
			}
			expect(result).toBeFalsy();
		});

		it.concurrent('should resolve when message_name is empty string', async () => {
			expect.assertions(1);
			const message_name = '';
			let result = null;
			try {
				validateInput(message_name, schema.message_name);
			} catch (e) {
				if (e instanceof Error) result = e.message;
			}
			expect(result).toBeTruthy();
		});

		it('should resolve when message_name is random string', async () => {
			expect.assertions(1);
			const message_name = await testHelper.randomHex(10);
			let result = null;
			try {
				validateInput(message_name, schema.message_name);
			} catch (e) {
				if (e instanceof Error) result = e.message;
			}
			expect(result).toBeTruthy();
		});

		it.concurrent('should resolve when message_name is random boolean', async () => {
			expect.assertions(1);
			const message_name = testHelper.randomBoolean;
			let result = null;
			try {
				validateInput(message_name, schema.message_name);
			} catch (e) {
				if (e instanceof Error) result = e.message;
			}
			expect(result).toBeTruthy();
		});

		it.concurrent('should resolve when message_name is random number', async () => {
			expect.assertions(1);
			const message_name = testHelper.randomBoolean;
			let result = null;
			try {
				validateInput(message_name, schema.message_name);
			} catch (e) {
				if (e instanceof Error) result = e.message;
			}
			expect(result).toBeTruthy();
		});
	});
	
});