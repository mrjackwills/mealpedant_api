import { validateInput } from '../../lib/validateInput';
import { schema } from '../../config/schema';
import { TestHelper } from '../testHelper';

const testHelper = new TestHelper();
import { afterAll, describe, expect, it } from 'vitest';

describe('schema tests', () => {

	afterAll(async () => {
		await testHelper.afterAll();
	});

	const photo = 'photo::convert';
	const ping = 'ping';

	describe(`convertPhoto schema`, () => {

		it('should resolve when message_name and data.password correct', async () => {
			expect.assertions(1);
			const message_name = photo;
			const data = {
				originalFileName: await testHelper.randomOriginalFileName()
			};
			let result = null;
			try {
				validateInput({ message_name, data }, schema.convertPhoto);
			} catch (e) {
				if (e instanceof Error) result = e.message;
			}
			expect(result).toBeFalsy();
		});

		it('should throw when message_name invalid', async () => {
			expect.assertions(1);
			const message_name = await testHelper.randomHex(8);
			const data = {
				originalFileName: await testHelper.randomOriginalFileName()
			};
			let result = null;
			try {
				validateInput({ message_name, data }, schema.convertPhoto);
			} catch (e) {
				if (e instanceof Error) result = e.message;
			}
			expect(result).toBeTruthy();
		});

		it('should throw when message_name missing', async () => {
			expect.assertions(1);
			const data = {
				originalFileName: await testHelper.randomOriginalFileName()
			};
			let result = null;
			try {
				validateInput({ data }, schema.convertPhoto);
			} catch (e) {
				if (e instanceof Error) result = e.message;
			}
			expect(result).toBeTruthy();
		});

		it('should throw when message_name key name invalid', async () => {
			expect.assertions(1);
			const messagename = photo;
			const data = {
				originalFileName: await testHelper.randomOriginalFileName()
			};
			try {
				validateInput({ messagename, data }, schema.convertPhoto);
			} catch (e) {
				expect(e).toBeTruthy();
			}
		});

		it('should throw when data missing', async () => {
			expect.assertions(1);
			const message_name = photo;
			let result = null;
			try {
				validateInput({ message_name }, schema.convertPhoto);
			} catch (e) {
				if (e instanceof Error) result = e.message;
			}
			expect(result).toBeTruthy();
		});

		it('should throw when data missing password key', async () => {
			expect.assertions(1);
			const message_name = photo;
			let result = null;
			const data = {};
			try {
				validateInput({ message_name, data }, schema.convertPhoto);
			} catch (e) {
				if (e instanceof Error) result = e.message;
			}
			expect(result).toBeTruthy();
		});
		
		it('should throw when data key invalid name', async () => {
			expect.assertions(1);
			const message_name = photo;
			let result = null;
			const Data = {
				originalFileName: await testHelper.randomOriginalFileName()
			};
			try {
				validateInput({ message_name, Data }, schema.convertPhoto);
			} catch (e) {
				if (e instanceof Error) result = e.message;
			}
			expect(result).toBeTruthy();
		});

		it('should throw when originalFileName empty string', async () => {
			expect.assertions(1);
			const message_name = photo;
			let result = null;
			const data = {
				originalFileName: ''
			};
			try {
				validateInput({ message_name, data }, schema.convertPhoto);
			} catch (e) {
				if (e instanceof Error) result = e.message;
			}
			expect(result).toBeTruthy();
		});

		it('should throw when originalFileName boolean', async () => {
			expect.assertions(1);
			const message_name = photo;
			let result = null;
			const data = {
				originalFileName: testHelper.randomBoolean
			};
			try {
				validateInput({ message_name, data }, schema.convertPhoto);
			} catch (e) {
				if (e instanceof Error) result = e.message;
			}
			expect(result).toBeTruthy();
		});
		
		it('should throw when originalFileName missing .jpeg suffix', async () => {
			expect.assertions(1);
			const message_name = photo;
			let result = null;
			const name = await testHelper.randomOriginalFileName();
			const data = {
				originalFileName: name.split('.jpeg')[0]
			};
			try {
				validateInput({ message_name, data }, schema.convertPhoto);
			} catch (e) {
				if (e instanceof Error) result = e.message;
			}
			expect(result).toBeTruthy();
		});

		it('should throw when originalFileName hex wrong length', async () => {
			expect.assertions(1);
			const message_name = photo;
			let result = null;

			const randomHex = await testHelper.randomHex(15);
			const randomPerson = testHelper.randomPersonInitial;
			const randomDate = testHelper.randomDate;
			const name = `${randomDate}_${randomPerson}_O_${randomHex}.jpeg`;

			const data = {
				originalFileName: name
			};
			try {
				validateInput({ message_name, data }, schema.convertPhoto);
			} catch (e) {
				if (e instanceof Error) result = e.message;
			}
			expect(result).toBeTruthy();
		});

		it('should throw when originalFileName missing date', async () => {
			expect.assertions(1);
			const message_name = photo;
			let result = null;

			const randomHex = await testHelper.randomHex(16);
			const randomPerson = testHelper.randomPersonInitial;
			const name = `${randomPerson}_O_${randomHex}.jpeg`;

			const data = {
				originalFileName: name
			};
			try {
				validateInput({ message_name, data }, schema.convertPhoto);
			} catch (e) {
				if (e instanceof Error) result = e.message;
			}
			expect(result).toBeTruthy();
		});

		it('should throw when originalFileName missing person', async () => {
			expect.assertions(1);
			const message_name = photo;
			let result = null;

			const randomHex = await testHelper.randomHex(16);
			const randomDate = testHelper.randomDate;
			const name = `${randomDate}_O_${randomHex}.jpeg`;

			const data = {
				originalFileName: name
			};
			try {
				validateInput({ message_name, data }, schema.convertPhoto);
			} catch (e) {
				if (e instanceof Error) result = e.message;
			}
			expect(result).toBeTruthy();
		});

		it('should throw when extra key in data', async () => {
			expect.assertions(1);
			const message_name = photo;
			const random = await testHelper.randomHex();
			let result = null;
			const data = {
				originalFileName: await testHelper.randomOriginalFileName(),
				[random]: random
			};
			try {
				validateInput({ message_name, data }, schema.convertPhoto);
			} catch (e) {
				if (e instanceof Error) result = e.message;
			}
			expect(result).toBeTruthy();
		});
	});

	describe(`PING schema`, () => {

		it('should resolve when message_name and data.attempt & data.password correct', async () => {
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

		it('should throw when message_name missing', async () => {
			expect.assertions(1);
			let result = null;
			try {
				validateInput({ }, schema.ping);

			} catch (e) {
				if (e instanceof Error) result = e.message;
			}
			expect(result).toBeTruthy();
		});

		it('should throw when message_name key name invalid', async () => {
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

		it('should resolve when message_name is either convertPhoto or PING', async () => {
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

		it('should resolve when message_name is empty string', async () => {
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

		it('should resolve when message_name is random boolean', async () => {
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

		it('should resolve when message_name is random number', async () => {
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