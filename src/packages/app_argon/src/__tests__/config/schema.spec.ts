import { validateInput } from '../../lib/validateInput';
import { schema } from '../../config/schema';
import { TestHelper } from '../testHelper';
import { describe, expect, it } from 'vitest';

const testHelper = new TestHelper();

describe('schema tests', () => {

	const message_name_create_hash = 'argon::create_hash';
	const message_name_validate_hash = 'argon::validate_hash';
	const message_name_ping = 'ping';

	describe(`createHash schema`, () => {

		it('should resolve when message_name and data.password correct', async () => {
			expect.assertions(1);
			const message_name = message_name_create_hash;
			const data = {
				password: await testHelper.randomHex(16),
			};
			let result = null;
			try {
				validateInput({ message_name, data }, schema.createHash);
			} catch (e) {
				if (e instanceof Error) result = e.message;	}
			expect(result).toBeFalsy();
		});

		it('should throw when message_name invalid', async () => {
			expect.assertions(1);
			const message_name = await testHelper.randomHex(8);
			const data = {
				password: await testHelper.randomHex(16),
			};
			let result = null;
			try {
				validateInput({ message_name, data }, schema.createHash);
			} catch (e) {
				if (e instanceof Error) result = e.message;	}
			expect(result).toBeTruthy();
		});

		it('should throw when message_name missing', async () => {
			expect.assertions(1);
			const data = {
				password: await testHelper.randomHex(16),
			};
			let result = null;
			try {
				validateInput({ data }, schema.createHash);
			} catch (e) {
				if (e instanceof Error) result = e.message;	}
			expect(result).toBeTruthy();
		});

		it('should throw when message_name key name invalid', async () => {
			expect.assertions(1);
			const messagename = message_name_create_hash;
			const data = {
				password: await testHelper.randomHex(16),
			};
			let result = null;
			try {
				validateInput({ messagename, data }, schema.createHash);
			} catch (e) {
				if (e instanceof Error) result = e.message;	}
			expect(result).toBeTruthy();
		});

		it('should throw when data missing', async () => {
			expect.assertions(1);
			const message_name = message_name_create_hash;
			let result = null;
			try {
				validateInput({ message_name }, schema.createHash);
			} catch (e) {
				if (e instanceof Error) result = e.message;	}
			expect(result).toBeTruthy();
		});

		it('should throw when data missing password key', async () => {
			expect.assertions(1);
			const message_name = message_name_create_hash;
			let result = null;
			const data = {};
			try {
				validateInput({ message_name, data }, schema.createHash);
			} catch (e) {
				if (e instanceof Error) result = e.message;	}
			expect(result).toBeTruthy();
		});
		
		it('should throw when data key invalid name', async () => {
			expect.assertions(1);
			const message_name = message_name_create_hash;
			let result = null;
			const Data = {
				password: await testHelper.randomHex(16),
			};
			try {
				validateInput({ message_name, Data }, schema.createHash);
			} catch (e) {
				if (e instanceof Error) result = e.message;	}
			expect(result).toBeTruthy();
		});

		it('should throw when password missing', async () => {
			expect.assertions(1);
			const message_name = message_name_create_hash;
			let result = null;
			const data = {
			} ;
			try {
				validateInput({ message_name, data }, schema.createHash);
			} catch (e) {
				if (e instanceof Error) result = e.message;	}
			expect(result).toBeTruthy();
		});

		it('should throw when password empty string', async () => {
			expect.assertions(1);
			const message_name = message_name_create_hash;
			let result = null;
			const data = {
				password: ''
			};
			try {
				validateInput({ message_name, data }, schema.createHash);
			} catch (e) {
				if (e instanceof Error) result = e.message;	}
			expect(result).toBeTruthy();
		});

		it('should throw when password boolean', async () => {
			expect.assertions(1);
			const message_name = message_name_create_hash;
			let result = null;
			const data = {
				password: testHelper.randomBoolean
			};
			try {
				validateInput({ message_name, data }, schema.createHash);
			} catch (e) {
				if (e instanceof Error) result = e.message;	}
			expect(result).toBeTruthy();
		});

		it('should throw when extra key in data', async () => {
			expect.assertions(1);
			const message_name = message_name_create_hash;
			const random = await testHelper.randomHex();
			let result = null;
			const data = {
				password: random,
				[random]: random
			};
			try {
				validateInput({ message_name, data }, schema.createHash);
			} catch (e) {
				if (e instanceof Error) result = e.message;	}
			expect(result).toBeTruthy();
		});
	});

	describe(`validateHash schema`, () => {

		it('should resolve when message_name and data.attempt & data.password correct', async () => {
			expect.assertions(1);
			const message_name = message_name_validate_hash;
			const data = {
				attempt: await testHelper.randomHex(16),
				known_password_hash: await testHelper.randomHex(16),
			};
			let result = null;
			try {
				validateInput({ message_name, data }, schema.validateHash);
			} catch (e) {
				if (e instanceof Error) result = e.message;	}
			expect(result).toBeFalsy();
		});

		it('should throw when message_name invalid', async () => {
			expect.assertions(1);
			const message_name = await testHelper.randomHex(8);
			const data = {
				attempt: await testHelper.randomHex(16),
				known_password_hash: await testHelper.randomHex(16),
			};
			let result = null;
			try {
				validateInput({ message_name, data }, schema.validateHash);

			} catch (e) {
				if (e instanceof Error) result = e.message;	}
			expect(result).toBeTruthy();
		});

		it('should throw when message_name missing', async () => {
			expect.assertions(1);
			const data = {
				attempt: await testHelper.randomHex(16),
				known_password_hash: await testHelper.randomHex(16),
			};
			let result = null;
			try {
				validateInput({ data }, schema.validateHash);

			} catch (e) {
				if (e instanceof Error) result = e.message;	}
			expect(result).toBeTruthy();
		});

		it('should throw when message_name key name invalid', async () => {
			expect.assertions(1);
			const messagename = message_name_validate_hash;
			const data = {
				attempt: await testHelper.randomHex(16),
				known_password_hash: await testHelper.randomHex(16),
			};
			let result = null;
			try {
				validateInput({ messagename, data }, schema.validateHash);

			} catch (e) {
				if (e instanceof Error) result = e.message;	}
			expect(result).toBeTruthy();
		});

		it('should throw when data missing', async () => {
			expect.assertions(1);
			const message_name = message_name_validate_hash;
			let result = null;
			try {
				validateInput({ message_name }, schema.validateHash);

			} catch (e) {
				if (e instanceof Error) result = e.message;	}
			expect(result).toBeTruthy();
		});

		it('should throw when data missing attempt key', async () => {
			expect.assertions(1);
			const message_name = message_name_validate_hash;
			const data = {
				known_password_hash: await testHelper.randomHex(16),
			};
			let result = null;
			try {
				validateInput({ message_name, data }, schema.validateHash);

			} catch (e) {
				if (e instanceof Error) result = e.message;	}
			expect(result).toBeTruthy();
		});

		it('should throw when data missing known_password_hash key', async () => {
			expect.assertions(1);
			const message_name = message_name_validate_hash;
			const data = {
				attempt: await testHelper.randomHex(16),
			};
			let result = null;
			try {
				validateInput({ message_name, data }, schema.validateHash);

			} catch (e) {
				if (e instanceof Error) result = e.message;	}
			expect(result).toBeTruthy();
		});
		
		it('should throw when data key invalid name', async () => {
			expect.assertions(1);
			const message_name = message_name_validate_hash;
			const Data = {
				attempt: await testHelper.randomHex(16),
				known_password_hash: await testHelper.randomHex(16),
			};
			let result = null;
			try {
				validateInput({ message_name, Data }, schema.validateHash);

			} catch (e) {
				if (e instanceof Error) result = e.message;	}
			expect(result).toBeTruthy();
		});

		it('should throw when attempt empty string', async () => {
			expect.assertions(1);
			const message_name = message_name_validate_hash;
			let result = null;
			const data = {
				attempt: '',
				known_password_hash: await testHelper.randomHex(16),
			};
			try {
				validateInput({ message_name, data }, schema.validateHash);
			} catch (e) {
				if (e instanceof Error) result = e.message;	}
			expect(result).toBeTruthy();
		});

		it('should throw when known_password_hash empty string', async () => {
			expect.assertions(1);
			const message_name = message_name_validate_hash;
			let result = null;
			const data = {
				known_password_hash: '',
				attempt: await testHelper.randomHex(16),
			};
			try {
				validateInput({ message_name, data }, schema.validateHash);
			} catch (e) {
				if (e instanceof Error) result = e.message;	}
			expect(result).toBeTruthy();
		});

		it('should throw when attempt is a boolean', async () => {
			expect.assertions(1);
			const message_name = message_name_validate_hash;
			let result = null;
			const data = {
				attempt: testHelper.randomBoolean,
				known_password_hash: await testHelper.randomHex(16),
			};
			try {
				validateInput({ message_name, data }, schema.validateHash);
			} catch (e) {
				if (e instanceof Error) result = e.message;	}
			expect(result).toBeTruthy();
		});
		it('should throw when known_password_hash is a boolean', async () => {
			expect.assertions(1);
			const message_name = message_name_validate_hash;
			let result = null;
			const data = {
				attempt: await testHelper.randomHex(16),
				known_password_hash: testHelper.randomBoolean,
			};
			try {
				validateInput({ message_name, data }, schema.validateHash);
			} catch (e) {
				if (e instanceof Error) result = e.message;	}
			expect(result).toBeTruthy();
		});
		
		it('should throw when extra key in data', async () => {
			expect.assertions(1);
			const message_name = message_name_validate_hash;
			const random = await testHelper.randomHex();
			let result = null;
			const data = {
				attempt: random,
				known_password_hash: random,
				[random]: random
			};
			try {
				validateInput({ message_name, data }, schema.validateHash);
			} catch (e) {
				if (e instanceof Error) result = e.message;	}
			expect(result).toBeTruthy();
		});
	});

	describe(`message_name schema`, () => {

		it('should resolve when message_name is either createHash or validateHash', async () => {
			expect.assertions(1);
			const message_name = testHelper.randomMessageName;
			let result = null;
			try {
				validateInput(message_name, schema.message_name);
			} catch (e) {
				if (e instanceof Error) result = e.message;	}
			expect(result).toBeFalsy();
		});

		it('should resolve when message_name is capitalized', async () => {
			expect.assertions(1);
			const message_name = testHelper.randomMessageName.toUpperCase();
			let result = null;
			try {
				validateInput(message_name, schema.message_name);
			} catch (e) {
				if (e instanceof Error) result = e.message;	}
			expect(result).toBeTruthy();
		});

		it('should resolve when message_name is empty string', async () => {
			expect.assertions(1);
			const message_name = '';
			let result = null;
			try {
				validateInput(message_name, schema.message_name);
			} catch (e) {
				if (e instanceof Error) result = e.message;	}
			expect(result).toBeTruthy();
		});

		it('should resolve when message_name is random string', async () => {
			expect.assertions(1);
			const message_name = await testHelper.randomHex(10);
			let result = null;
			try {
				validateInput(message_name, schema.message_name);
			} catch (e) {
				if (e instanceof Error) result = e.message;	}
			expect(result).toBeTruthy();
		});

		it('should resolve when message_name is random boolean', async () => {
			expect.assertions(1);
			const message_name = testHelper.randomBoolean;
			let result = null;
			try {
				validateInput(message_name, schema.message_name);
			} catch (e) {
				if (e instanceof Error) result = e.message;	}
			expect(result).toBeTruthy();
		});

		it('should resolve when message_name is random number', async () => {
			expect.assertions(1);
			const message_name = testHelper.randomBoolean;
			let result = null;
			try {
				validateInput(message_name, schema.message_name);
			} catch (e) {
				if (e instanceof Error) result = e.message;	}
			expect(result).toBeTruthy();
		});
	});

	describe(`PING schema`, () => {

		it('should resolve when message_name and data.attempt & data.password correct', async () => {
			expect.assertions(1);
			const message_name = message_name_ping;
			
			let result = null;
			try {
				validateInput({ message_name }, schema.ping);
			} catch (e) {
				if (e instanceof Error) result = e.message;	}
			expect(result).toBeFalsy();
		});

		it('should throw when message_name invalid', async () => {
			expect.assertions(1);
			const message_name = await testHelper.randomHex(8);
			let result = null;
			try {
				validateInput({ message_name }, schema.ping);

			} catch (e) {
				if (e instanceof Error) result = e.message;	}
			expect(result).toBeTruthy();
		});

		it('should throw when message_name missing', async () => {
			expect.assertions(1);
			let result = null;
			try {
				validateInput({ }, schema.ping);

			} catch (e) {
				if (e instanceof Error) result = e.message;	}
			expect(result).toBeTruthy();
		});

		it('should throw when message_name key name invalid', async () => {
			expect.assertions(1);
			const messagename = message_name_ping;
			let result = null;
			try {
				validateInput({ messagename }, schema.ping);

			} catch (e) {
				if (e instanceof Error) result = e.message;	}
			expect(result).toBeTruthy();
		});
		
		it('should throw when data provided', async () => {
			expect.assertions(1);
			const message_name = message_name_ping;
			const random = await testHelper.randomHex();
			let result = null;
			const data = {
				[random]: random
			};
			try {
				validateInput({ message_name, data }, schema.ping);
			} catch (e) {
				if (e instanceof Error) result = e.message;	}
			expect(result).toBeTruthy();
		});
	});

	describe(`message_name schema`, () => {

		it('should resolve when message_name is either createHash or validateHash', async () => {
			expect.assertions(1);
			const message_name = testHelper.randomMessageName;
			let result = null;
			try {
				validateInput(message_name, schema.message_name);
			} catch (e) {
				if (e instanceof Error) result = e.message;	}
			expect(result).toBeFalsy();
		});

		it('should resolve when message_name is empty string', async () => {
			expect.assertions(1);
			const message_name = '';
			let result = null;
			try {
				validateInput(message_name, schema.message_name);
			} catch (e) {
				if (e instanceof Error) result = e.message;	}
			expect(result).toBeTruthy();
		});

		it('should resolve when message_name is random string', async () => {
			expect.assertions(1);
			const message_name = await testHelper.randomHex(10);
			let result = null;
			try {
				validateInput(message_name, schema.message_name);
			} catch (e) {
				if (e instanceof Error) result = e.message;	}
			expect(result).toBeTruthy();
		});

		it('should resolve when message_name is random boolean', async () => {
			expect.assertions(1);
			const message_name = testHelper.randomBoolean;
			let result = null;
			try {
				validateInput(message_name, schema.message_name);
			} catch (e) {
				if (e instanceof Error) result = e.message;	}
			expect(result).toBeTruthy();
		});

		it('should resolve when message_name is random number', async () => {
			expect.assertions(1);
			const message_name = testHelper.randomBoolean;
			let result = null;
			try {
				validateInput(message_name, schema.message_name);
			} catch (e) {
				if (e instanceof Error) result = e.message;	}
			expect(result).toBeTruthy();
		});
	});
	
});