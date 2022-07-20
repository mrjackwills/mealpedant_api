import { customTypeError } from '../config/customError';
import { Schema, attempt, ValidationError } from 'joi';
import { ErrorMessages } from '../types/enum_error';

export const validateInput = <T> (input: unknown, schema: Schema): T|undefined => {
	try {
		const validatedInput = attempt(input, schema);
		return validatedInput;
	} catch (e) {
		if (e instanceof ValidationError && e._original?.password) e._original.password = null;
		if (e instanceof ValidationError && e._original?.newPassword) e._original.newPassword = null;
		const message = e instanceof ValidationError && e.details[0]?.context?.label ? e.details[0].context.label : ErrorMessages.INPUT_VALIDATION;
		const errorToThrow = customTypeError(message);
		throw errorToThrow;
	}
};