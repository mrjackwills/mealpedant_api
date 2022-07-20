import { validate_input } from '../../lib/validateinput';
import * as joi from 'types-joi';

import { describe, expect, it } from 'vitest';

const basic_schema = joi.object({
	string_key: joi.string().required(),
	number_key: joi.number().required()
});

describe('validate_input tests', () => {

	it.concurrent('validate basic schema', async () => {
		expect.assertions(1);
		let result = false;
		try {
			validate_input({ string_key: 'string', number_key: 10 }, basic_schema);
		} catch (e) {
			result = true;
		}
		expect(result).toBeFalsy();
	});

	it.concurrent('should throw error when key not supplied', async () => {
		expect.assertions(1);
		let result = false;
		try {
			validate_input({ string_key: 'string' }, basic_schema);
		} catch (e) {
			result = true;
		}
		expect(result).toBeTruthy();
	});

	it.concurrent('should throw error when extra key', async () => {
		expect.assertions(1);
		let result = false;
		try {
			validate_input({ string_key: 'string', number_key: 10, extra_key: 'extra_key' }, basic_schema);
		} catch (e) {
			result = true;
		}
		expect(result).toBeTruthy();
	});

	it.concurrent('should throw error when key wrong type', async () => {
		expect.assertions(1);
		let result = false;
		try {
			validate_input({ string_key: 'string', number_key: 'number' }, basic_schema);
		} catch (e) {
			result = true;
		}
		expect(result).toBeTruthy();
	});

	it.concurrent('should throw error when no data supplied', async () => {
		expect.assertions(1);
		let result = false;
		try {
			validate_input(null, basic_schema);
		} catch (e) {
			result = true;
		}
		expect(result).toBeTruthy();
	});

});