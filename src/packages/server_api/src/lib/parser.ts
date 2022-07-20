import { customTypeError } from '../config/customError';
import { ErrorMessages } from '../types/enum_error';
import { log } from '../config/log';
import { parse } from 'secure-json-parse';
import { schema } from '../config/rabbitSchema';
import { Schema, attempt } from 'joi';
import { TResponseValidateHash, TResponseError, TResponseConverted, TResponseCreateHash, TResponseBackup } from '../types';
import { RabbitMessage } from '../types/enum_rabbitMessage';

class Parser {

	#validateInput <T> (input: unknown, schema: Schema): T|undefined {
		try {
			const validatedInput = attempt(input, schema);
			return validatedInput;
		} catch (e) {
			log.error(e);
			throw customTypeError(ErrorMessages.PARSER_ERROR);
		}
	}

	photo (message: string): string {
		const parsedMessage = parse(message, undefined, { protoAction: 'remove', constructorAction: 'remove' });

		if (parsedMessage.message_name === RabbitMessage.PHOTO_CONVERT) {
			const response = <TResponseConverted> this.#validateInput(parsedMessage, schema.convertPhoto);
			return response.data.response;
		} else {
			const response = <TResponseError> this.#validateInput(parsedMessage, schema.error);
			throw response.data.error;
		}
	}

	backup (message: string): boolean {
		const parsedMessage = parse(message, undefined, { protoAction: 'remove', constructorAction: 'remove' });

		if (parsedMessage.message_name !== 'error') {
			const response = <TResponseBackup> this.#validateInput(parsedMessage, schema.backup);
			return response.data.response;
		} else {
			const response = <TResponseError> this.#validateInput(parsedMessage, schema.error);
			throw response.data.error;
		}
	}

	validate (message: string): boolean {
		const parsedMessage = parse(message, undefined, { protoAction: 'remove', constructorAction: 'remove' });
		if (parsedMessage.message_name === RabbitMessage.ARGON_VALIDATE_HASH) {
			const response = <TResponseValidateHash> this.#validateInput(parsedMessage, schema.validateResponse);
			return response.data.response;
		} else {
			const response = <TResponseError> this.#validateInput(parsedMessage, schema.error);
			throw response.data.error;
		}
	}
	
	create (message: string): string {
		const parsedMessage = parse(message, undefined, { protoAction: 'remove', constructorAction: 'remove' });
		if (parsedMessage.message_name === RabbitMessage.ARGON_CREATE_HASH) {
			const response = <TResponseCreateHash> this.#validateInput(parsedMessage, schema.createResponse);
			return response.data.response;
		} else {
			const response = <TResponseError> this.#validateInput(parsedMessage, schema.error);
			throw response.data.error;
		}
	}

}
export const parser = new Parser();
