import { customTypeError } from '../config/customError';
import { ErrorMessages } from '../types/enum_error';
import { HttpCode } from '../types/enum_httpCode';
import { ValidationError } from 'joi';
import * as joi from 'types-joi';

export const validate_input= <T>(input: unknown, schema: joi.ObjectSchema<T>): T => {
	try {
		if (!input) throw customTypeError('validate_input: !input');
		if (!schema) throw customTypeError('validate_input: !schema');
		const result = joi.attempt(input, schema);
		return result;
	} catch (e) {
		if (e instanceof ValidationError && e._original?.password) e._original.password = null;
		if (e instanceof ValidationError && e._original?.newPassword) e._original.newPassword = null;
		const message = e instanceof ValidationError && e.details[0]?.context?.label ? `${ErrorMessages.INVALID_DATA}: ${e.details[0].context.label}` : ErrorMessages.INVALID_DATA;
		const errorToThrow = customTypeError(message, HttpCode.BAD_REQUEST);
		throw errorToThrow;
	}
};