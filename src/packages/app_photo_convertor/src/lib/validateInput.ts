import { customTypeError } from '../config/customError';
import { Schema, attempt } from 'joi';
import { log } from '../config/log';

export const validateInput = <T> (input: unknown, schema: Schema): T|undefined => {
	try {
		const validatedInput = attempt(input, schema);
		return validatedInput;
	} catch (e) {
		log.error(e);
		const message = e instanceof Error ? e.message : `Error: ${e}`;
		throw customTypeError(message);
	}
};