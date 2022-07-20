import { validate_input } from '../../lib/validateinput';
import { schema_user } from '../../components/user/user_schema';
import { TestHelper } from '../testHelper';

import { afterAll, describe, expect, it } from 'vitest';

const testHelper = new TestHelper();

describe('user_schema joi tests', () => {

	afterAll(() => testHelper.afterAll());

	describe(`schema_userChangePassword function`, () => {

		it('should resolve when password & newPassword', async () => {
			expect.assertions(1);
			let result = false;
			try {
				validate_input({ password: await testHelper.randomHex(12), newPassword: await testHelper.randomHex(12) }, schema_user.changePassword);
			} catch (e) {
				result = true;
			}
			expect(result).toBeFalsy();
		});

		it('should resolve when password & newPassword & token', async () => {
			expect.assertions(1);
			let result = false;
			try {
				validate_input({ password: await testHelper.randomHex(12), newPassword: await testHelper.randomHex(12), token: '000000' }, schema_user.changePassword);
			} catch (e) {
				result = true;
			}
			expect(result).toBeFalsy();
		});

		it('should throw error when password is < 10 ', async () => {
			expect.assertions(2);
			try {
				validate_input({ password: await testHelper.randomHex(9), newPassword: await testHelper.randomHex(12) }, schema_user.changePassword);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_password);
				}
			}
		});

		it('should throw error when newPassword is < 10 ', async () => {
			expect.assertions(2);
			try {
				validate_input({ password: await testHelper.randomHex(12), newPassword: await testHelper.randomHex(9) }, schema_user.changePassword);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_password);
				}
			}
		});

		it('should throw error when Password is empty string', async () => {
			expect.assertions(2);
			try {
				validate_input({ password: '', newPassword: await testHelper.randomHex(12) }, schema_user.changePassword);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_password);
				}
			}
		});

		it('should throw error when newPassword is empty string', async () => {
			expect.assertions(2);
			try {
				validate_input({ password: await testHelper.randomHex(12), newPassword: '' }, schema_user.changePassword);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_password);
				}
			}
		});

		it('should throw error when password is a number', async () => {
			expect.assertions(2);
			try {
				validate_input({ password: '', newPassword: await testHelper.randomHex(12) }, schema_user.changePassword);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_password);
				}
			}
		});

		it('should throw error when newPassword is a number', async () => {
			expect.assertions(2);
			try {
				validate_input({ password: await testHelper.randomHex(12), newPassword: '' }, schema_user.changePassword);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_password);
				}
			}
		});

		it('should throw error when password is a boolean', async () => {
			expect.assertions(2);
			try {
				validate_input({ password: testHelper.randomBoolean(), newPassword: await testHelper.randomHex(12) }, schema_user.changePassword);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_password);
				}
			}
		});

		it('should throw error when newPassword is a boolean', async () => {
			expect.assertions(2);
			try {
				validate_input({ password: await testHelper.randomHex(12), newPassword: testHelper.randomBoolean() }, schema_user.changePassword);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_password);
				}
			}
		});

		it('should throw error when password is a null', async () => {
			expect.assertions(2);
			try {
				validate_input({ password: null, newPassword: await testHelper.randomHex(12) }, schema_user.changePassword);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_password);
				}
			}
		});

		it('should throw error when newPassword is a null', async () => {
			expect.assertions(1);
			let result = false;
			try {
				validate_input({ password: await testHelper.randomHex(12), newPassword: null }, schema_user.changePassword);
			} catch (e) {
				result = true;
			}
			expect(result).toBeTruthy();
		});
		
		it('should fail when invalid token presented', async () => {
			expect.assertions(2);
			try {
				validate_input({ password: await testHelper.randomHex(12), newPassword: await testHelper.randomHex(12), token: await testHelper.randomHex(2) }, schema_user.changePassword);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_token);
				}
			}
		});

		it('should fail when backupToken length !== 16', async () => {
			expect.assertions(2);
			try {
				validate_input({ password: await testHelper.randomHex(12), newPassword: await testHelper.randomHex(12), token: await testHelper.randomHex(9) }, schema_user.changePassword);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_token);
				}
			}
		});

		it('should fail when backupToken not valid hex string', async () => {
			expect.assertions(2);
			try {
				validate_input({ password: await testHelper.randomHex(12), newPassword: await testHelper.randomHex(12), token: `${await testHelper.randomHex(15)}z` }, schema_user.changePassword);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_token);
				}
			}
		});
	});
	
	describe(`twoFASetup schema`, () => {

		it('resolve when valid token presented', async () => {
			expect.assertions(1);
			let result = false;
			try {
				validate_input({ token: '000000' }, schema_user.twoFASetup);
			} catch (e) {
				result = true;
			}
			expect(result).toBeFalsy();
		});

		it('should throw when token.length > 6', async () => {
			expect.assertions(2);
			try {
				validate_input({ token: '0000000' }, schema_user.twoFASetup);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_token);
				}
			}
		});

		it('should throw when token.length < 6', async () => {
			expect.assertions(2);
			try {
				validate_input({ token: '0000' }, schema_user.twoFASetup);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_token);
				}
			}
		});

		it('should fail when token includes non digits', async () => {
			expect.assertions(2);
			try {
				validate_input({ token: 'zzzzzz' }, schema_user.twoFASetup);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_token);
				}
			}
		});

		it('should fail when token empty string', async () => {
			expect.assertions(2);
			try {
				validate_input({ token: '' }, schema_user.twoFASetup);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_token);
				}
			}
		});

		it('should fail when token empty null', async () => {
			expect.assertions(2);
			try {
				validate_input({ token: null }, schema_user.twoFASetup);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_token);
				}
			}
		});

		it('should fail when token empty undefined', async () => {
			expect.assertions(2);
			try {
				validate_input({ token: undefined }, schema_user.twoFASetup);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_token);
				}
			}
		});

		it('should fail when token empty number', async () => {
			expect.assertions(2);
			try {
				validate_input({ token: testHelper.randomNumber() }, schema_user.twoFASetup);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_token);
				}
			}
		});
	});

	describe(`twoFaAlwaysRequired`, () => {

		it('should resolve when alwaysRequired, token, password, twoFABackup presented', async () => {
			expect.assertions(1);
			let result = false;
			try {
				validate_input({ alwaysRequired: true, token: '000000', password: await testHelper.randomHex(12), twoFABackup: true }, schema_user.twoFAAlwaysRequired);
			} catch (e) {
				result = true;
			}
			expect(result).toBeFalsy();
		});

		it('should resolve when alwaysRequired presented', async () => {
			let result = false;
			expect.assertions(1);
			try {
				validate_input({ alwaysRequired: true }, schema_user.twoFAAlwaysRequired);
			} catch (e) {
				result = true;
			}
			expect(result).toBeFalsy();
		});

		it('should throw when alwaysRequired string', async () => {
			expect.assertions(2);
			try {
				validate_input({ alwaysRequired: '0000000' }, schema_user.twoFASetup);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_token);
				}
			}
		});

		it('should throw when alwaysRequired null', async () => {
			expect.assertions(2);
			try {
				validate_input({ alwaysRequired: null }, schema_user.twoFASetup);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_token);
				}
			}
		});

		it('should throw when alwaysRequired undefined', async () => {
			expect.assertions(2);
			try {
				validate_input({ alwaysRequired: undefined }, schema_user.twoFASetup);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_token);
				}
			}
		});

		it('should throw when alwaysRequired undefined', async () => {
			expect.assertions(2);
			try {
				validate_input({ alwaysRequired: false, token: 0 }, schema_user.twoFASetup);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_token);
				}
			}
		});

		it('should throw when password invalid', async () => {
			expect.assertions(2);
			try {
				validate_input({ alwaysRequired: false, password: 'a' }, schema_user.twoFASetup);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_token);
				}
			}
		});
	});
});