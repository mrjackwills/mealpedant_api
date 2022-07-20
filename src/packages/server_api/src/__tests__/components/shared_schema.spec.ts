import { validate_input } from '../../lib/validateinput';
import { schema_shared } from '../../components/shared/shared_schema';
import { TestHelper } from '../testHelper';
import * as joi from 'types-joi';
import { afterAll, describe, expect, it } from 'vitest';

const testHelper = new TestHelper();

const b64_256_schema = joi.object({ b64_256: schema_shared.base64_256_string.required() });
const nameOriginal_schema = joi.object({ name: schema_shared.imageNameOriginal.required() });
const nameConverted_schema = joi.object({ name: schema_shared.imageNameConverted.required() });
const email_schema = joi.object({ email: schema_shared.email.required() });
const password_schema = joi.object({ password: schema_shared.password.required() });
const token_schema = joi.object({ token: schema_shared.token.required() });
const backupToken_schema = joi.object({ token: schema_shared.backupToken.required() });

describe('schema_meal joi test', () => {

	afterAll(() => testHelper.afterAll());

	describe(`base64_256_regex joi`, () => {

		it('should resolve when 256 hex string passed', async () => {
			expect.assertions(1);
			let result = false;
			try {
				const b64_256 = await testHelper.randomHex(256);
				validate_input({ b64_256 }, b64_256_schema);
			} catch (e) {
				result = true;
			}
			expect(result).toBeFalsy();
		});

		it('should throw error when hex string wrong length', async () => {
			expect.assertions(2);
			try {
				const b64_255 = await testHelper.randomHex(255);
				validate_input({ b64_256: b64_255 }, b64_256_schema);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_shared_verification);
				}
			}
		});

		it('should throw error when hex string wrong length', async () => {
			expect.assertions(2);
			try {
				const b64_256 = await testHelper.randomHex(266);
				validate_input({ b64_256: b64_256 }, b64_256_schema);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_shared_verification);
				}
			}
		});

		it('should throw error when hex string includes no hex char', async () => {
			expect.assertions(2);
			try {
				const b64_256 = await testHelper.randomHex(256);
				validate_input({ b64_256: `${b64_256.substring(0, 255)}z` }, b64_256_schema);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_shared_verification);
				}
			}
		});

		it('should throw error when hex string not a string: number', async () => {
			expect.assertions(2);
			try {
				validate_input({ b64_256: 3333333333 }, b64_256_schema);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_shared_verification);
				}
			}
		});

		it('should throw error when hex string not a string: array', async () => {
			expect.assertions(2);
			try {
				validate_input({ b64_256: [ '3333333333' ] }, b64_256_schema);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_shared_verification);
				}
			}
		});
	});

	describe(`imageNameOriginal joi`, () => {

		it('should resolve when valid imageNameOriginal valid', async () => {
			expect.assertions(1);
			let result = false;
			try {
				const name = `2020-01-01_${testHelper.randomPersonInitial()}_O_${await testHelper.randomHex(16)}.jpeg`;
				validate_input({ name }, nameOriginal_schema);
			} catch (e) {
				result = true;
			}
			expect(result).toBeFalsy();
		});

		it('should resolve when valid imageNameConverted valid', async () => {
			expect.assertions(1);
			let result = false;
			try {
				const name = `2020-01-01_${testHelper.randomPersonInitial()}_C_${await testHelper.randomHex(16)}.jpeg`;
				validate_input({ name }, nameConverted_schema);
			} catch (e) {
				result = true;
			}
			expect(result).toBeFalsy();
		});

		it('should throw error when imageNameOriginal person char invalid: lowercase', async () => {
			expect.assertions(2);
			try {
				const name = `2020-01-01_${testHelper.randomPersonInitial().toLowerCase()}_O_${await testHelper.randomHex(16)}.jpeg`;
				validate_input({ name }, nameOriginal_schema);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_filename);
				}
			}
		});

		it('should throw error when person lowercase', async () => {
			expect.assertions(2);
			try {
				const name = `2020-01-01_${testHelper.randomPersonInitial().toLowerCase()}_C_${await testHelper.randomHex(16)}.jpeg`;
				validate_input({ name }, nameConverted_schema);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_filename);
				}
			}
		});

		it('should throw error when imageNameOriginal person char invalid: person', async () => {
			expect.assertions(2);
			try {
				const name = `2020-01-01_B_O_${await testHelper.randomHex(16)}.jpeg`;
				validate_input({ name }, nameOriginal_schema);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_filename);
				}
			}
		});

		it('should throw error when imageNameOriginal person char invalid: person', async () => {
			expect.assertions(2);
			try {
				const name = `2020-01-01_A_C_${await testHelper.randomHex(16)}.jpeg`;
				validate_input({ name }, nameConverted_schema);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_filename);
				}
			}
		});

		it('should throw error when imageNameConverted date prefix: year', async () => {
			expect.assertions(2);
			try {
				const name = `202a-01-01_${testHelper.randomPersonInitial()}_C_${await testHelper.randomHex(16)}.jpeg`;
				validate_input({ name }, nameConverted_schema);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_filename);
				}
			}
		});

		it('should throw error when imageNameOriginal date prefix: year', async () => {
			expect.assertions(2);
			try {
				const name = `202a-01-01_${testHelper.randomPersonInitial()}_O_${await testHelper.randomHex(16)}.jpeg`;
				validate_input({ name }, nameConverted_schema);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_filename);
				}
			}
		});

		it('should throw error when imageNameConverted date prefix: month', async () => {
			expect.assertions(2);
			try {
				const name = `2020-1-01_${testHelper.randomPersonInitial()}_C_${await testHelper.randomHex(16)}.jpeg`;
				validate_input({ name }, nameConverted_schema);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_filename);
				}
			}
		});

		it('should throw error when imageNameOriginal date prefix: month', async () => {
			expect.assertions(2);
			try {
				const name = `2020-1-01_${testHelper.randomPersonInitial()}_O_${await testHelper.randomHex(16)}.jpeg`;
				validate_input({ name }, nameConverted_schema);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_filename);
				}
			}
		});

		it('should throw error when imageNameConverted date prefix: day', async () => {
			expect.assertions(2);
			try {
				const name = `2020-01-1_${testHelper.randomPersonInitial()}_C_${await testHelper.randomHex(16)}.jpeg`;
				validate_input({ name }, nameConverted_schema);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_filename);
				}
			}
		});

		it('should throw error when imageNameOriginal date prefix: day', async () => {
			expect.assertions(2);
			try {
				const name = `2020-01-1_${testHelper.randomPersonInitial()}_O_${await testHelper.randomHex(16)}.jpeg`;
				validate_input({ name }, nameConverted_schema);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_filename);
				}
			}
		});

		it('should throw error when imageNameOriginal hex invalid', async () => {
			expect.assertions(2);
			try {
				const name = `2020-01-01_${testHelper.randomPersonInitial()}_O_${await testHelper.randomHex(15)}z.jpeg`;
				validate_input({ name }, nameConverted_schema);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_filename);
				}
			}
		});

		it('should throw error when imageNameConverted hex invalid', async () => {
			expect.assertions(2);
			try {
				const name = `2020-01-01_${testHelper.randomPersonInitial()}_C_${await testHelper.randomHex(15)}z.jpeg`;
				validate_input({ name }, nameConverted_schema);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_filename);
				}
			}
		});

		it('should throw error when imageNameConverted jpeg invalid: .JPG', async () => {
			expect.assertions(2);
			try {
				const name = `2020-01-01_${testHelper.randomPersonInitial()}_C_${await testHelper.randomHex(16)}.JPG`;
				validate_input({ name }, nameConverted_schema);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_filename);
				}
			}
		});

		it('should throw error when imageNameConverted jpeg invalid: .J', async () => {
			expect.assertions(2);
			try {
				const name = `2020-01-01_${testHelper.randomPersonInitial()}_C_${await testHelper.randomHex(16)}.J`;
				validate_input({ name }, nameConverted_schema);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_filename);
				}
			}
		});

		it('should throw error when imageNameConverted jpeg invalid: uppercase', async () => {
			expect.assertions(2);
			try {
				const name = `2020-01-01_${testHelper.randomPersonInitial()}_C_${await testHelper.randomHex(16)}.JPEG`;
				validate_input({ name }, nameConverted_schema);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_filename);
				}
			}
		});
		
		it('should throw error when imageNameOriginal jpeg invalid: .JPG', async () => {
			expect.assertions(2);
			try {
				const name = `2020-01-01_${testHelper.randomPersonInitial()}_O_${await testHelper.randomHex(16)}.JPG`;
				validate_input({ name }, nameConverted_schema);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_filename);
				}
			}
		});

		it('should throw error when imageNameOriginal jpeg invalid: .J', async () => {
			expect.assertions(2);
			try {
				const name = `2020-01-01_${testHelper.randomPersonInitial()}_O_${await testHelper.randomHex(16)}.J`;
				validate_input({ name }, nameConverted_schema);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_filename);
				}
			}
		});

		it('should throw error when imageNameOriginal jpeg invalid: uppercase', async () => {
			expect.assertions(2);
			try {
				const name = `2020-01-01_${testHelper.randomPersonInitial()}_O_${await testHelper.randomHex(16)}.JPEG`;
				validate_input({ name }, nameConverted_schema);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_filename);
				}
			}
		});

		it('should throw error when imageNameConverted not a string: boolean', async () => {
			expect.assertions(2);
			try {
				validate_input({ name: testHelper.randomBoolean() }, nameConverted_schema);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_filename);
				}
			}
		});

		it('should throw error when imageNameOriginal not a string: boolean', async () => {
			expect.assertions(2);
			try {
				validate_input({ name: testHelper.randomBoolean() }, nameConverted_schema);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_filename);
				}
			}
		});

		it('should throw error when imageNameConverted not a string: number', async () => {
			expect.assertions(2);
			try {
				validate_input({ name: testHelper.randomNumber() }, nameConverted_schema);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_filename);
				}
			}
		});

		it('should throw error when imageNameOriginal not a string: number', async () => {
			expect.assertions(2);
			try {
				validate_input({ name: testHelper.randomNumber() }, nameConverted_schema);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_filename);
				}
			}
		});
		
	});

	describe(`email joi`, () => {

		it('should resolve when valid email', async () => {
			expect.assertions(1);
			let result = false;
			try {
				validate_input({ email: `${testHelper.randomNumberAsString()}@${await testHelper.randomHex(5)}.com` }, email_schema);
			} catch (e) {
				result = true;
			}
			expect(result).toBeFalsy();

		});

		it('should throw error when no @ sign', async () => {
			expect.assertions(2);
			try {
				validate_input({ email: `${testHelper.randomNumberAsString()}${await testHelper.randomHex(5)}.com` }, email_schema);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_email);
				}
			}
		});

		it('should throw error when no domain', async () => {
			expect.assertions(2);
			try {
				validate_input({ email: `${testHelper.randomNumberAsString()}@.com` }, email_schema);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_email);
				}
			}
		});

		it('should throw error when no tld', async () => {
			expect.assertions(2);
			try {
				validate_input({ email: `${testHelper.randomNumberAsString()}@${await testHelper.randomHex(8)}` }, email_schema);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_email);
				}
			}
		});

		it('should throw error when nothing before @', async () => {
			expect.assertions(2);
			try {
				validate_input({ email: `@${await testHelper.randomHex(8)}.com` }, email_schema);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_email);
				}
			}
		});

	});

	describe(`password_schema`, () => {

		it('should resolve when valid password', async () => {
			expect.assertions(1);
			let result = false;
			try {
				validate_input({ password: await testHelper.randomHex(12) }, password_schema);
			} catch (e) {
				result = true;
			}
			expect(result).toBeFalsy();
		});

		it('should throw error when password too short', async () => {
			expect.assertions(2);
			try {
				validate_input({ password: await testHelper.randomHex(8) }, password_schema);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_password);
				}
			}
		});

		it('should throw error when password empty', async () => {
			expect.assertions(2);
			try {
				validate_input({ password: '' }, password_schema);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_password);
				}
			}
		});

		it('should throw error when password null', async () => {
			expect.assertions(2);
			try {
				validate_input({ password: null }, password_schema);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_password);
				}
			}
		});

		it('should throw error when password undefined', async () => {
			expect.assertions(2);
			try {
				validate_input({ password: undefined }, password_schema);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_password);
				}
			}
		});

		it('should throw error when password a number', async () => {
			expect.assertions(2);
			try {
				validate_input({ password: testHelper.randomNumber() }, password_schema);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_password);
				}
			}
		});

		it('should throw error when password a date', async () => {
			expect.assertions(2);
			try {
				validate_input({ password: new Date() }, password_schema);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_password);
				}
			}
		});

		it('should throw error when password a array', async () => {
			expect.assertions(2);
			try {
				validate_input({ password: [ await testHelper.randomHex(12) ] }, password_schema);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_password);
				}
			}
		});

	});

	describe(`token joi`, () => {
		it('should resolve when token valid: with space', async () => {
			expect.assertions(1);
			let result = false;
			try {
				validate_input({ token: '000 000' }, token_schema);
			} catch (e) {
				result = true;
			}
			expect(result).toBeFalsy();
	
		});

		it('should resolve when token valid: without space', async () => {
			expect.assertions(1);
			let result = false;
			try {
				validate_input({ token: '000000' }, token_schema);
			} catch (e) {
				result = true;
			}
			expect(result).toBeFalsy();
		});

		it('should throw error when length invalid: too short', async () => {
			expect.assertions(2);
			try {
				validate_input({ token: '00000' }, token_schema);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_token);
				}
			}
		});

		it('should throw error when length invalid: too long', async () => {
			expect.assertions(2);
			try {
				validate_input({ token: '0000000' }, token_schema);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_token);
				}
			}
		});

		it('should throw error when token format invalid', async () => {
			expect.assertions(2);
			try {
				validate_input({ token: '00000z' }, token_schema);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_token);
				}
			}
		});

		it('should throw error when token empty', async () => {
			expect.assertions(2);
			try {
				validate_input({ token: '' }, token_schema);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_token);
				}
			}
		});

		it('should throw error when token null', async () => {
			expect.assertions(2);
			try {
				validate_input({ token: null }, token_schema);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_token);
				}
			}
		});

		it('should throw error when token undefined', async () => {
			expect.assertions(2);
			try {
				validate_input({ token: undefined }, token_schema);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_token);
				}
			}
		});

		it('should throw error when token false', async () => {
			expect.assertions(2);
			try {
				validate_input({ token: false }, token_schema);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_token);
				}
			}
		});

		it('should throw error when token date', async () => {
			expect.assertions(2);
			try {
				validate_input({ token: new Date() }, token_schema);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_token);
				}
			}
		});

		it('should throw error when token number', async () => {
			expect.assertions(2);
			try {
				validate_input({ token: testHelper.randomNumber() }, token_schema);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_token);
				}
			}
		});

	});

	describe(`backupToken joi`, () => {
		
		it('should resolve when token valid: with space', async () => {
			expect.assertions(1);
			let result = false;
			try {
				validate_input({ token: await testHelper.randomHex(16) }, backupToken_schema);
			} catch (e) {
				result = true;
			}
			expect(result).toBeFalsy();
	
		});

		it('should throw error when length invalid: too short', async () => {
			expect.assertions(2);
			try {
				validate_input({ token: await testHelper.randomHex(15) }, backupToken_schema);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_backup_token);
				}
			}
		});

		it('should throw error when length invalid: too long', async () => {
			expect.assertions(2);
			try {
				validate_input({ token: await testHelper.randomHex(17) }, backupToken_schema);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_backup_token);
				}
			}
		});

		it('should throw error when token format invalid', async () => {
			expect.assertions(2);
			try {
				validate_input({ token: `${await testHelper.randomHex(16)}z ` }, backupToken_schema);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_backup_token);
				}
			}
		});

		it('should throw error when token empty', async () => {
			expect.assertions(2);
			try {
				validate_input({ token: '' }, backupToken_schema);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_backup_token);
				}
			}
		});

		it('should throw error when token null', async () => {
			expect.assertions(2);
			try {
				validate_input({ token: null }, backupToken_schema);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_backup_token);
				}
			}
		});

		it('should throw error when token undefined', async () => {
			expect.assertions(2);
			try {
				validate_input({ token: undefined }, backupToken_schema);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_backup_token);
				}
			}
		});

		it('should throw error when token false', async () => {
			expect.assertions(2);
			try {
				validate_input({ token: false }, backupToken_schema);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_backup_token);
				}
			}
		});

		it('should throw error when token date', async () => {
			expect.assertions(2);
			try {
				validate_input({ token: new Date() }, backupToken_schema);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_backup_token);
				}
			}
		});

		it('should throw error when token number', async () => {
			expect.assertions(2);
			try {
				validate_input({ token: testHelper.randomNumber() }, backupToken_schema);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_backup_token);
				}
			}
		});

	});

});
