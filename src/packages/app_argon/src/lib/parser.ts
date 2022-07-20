import { parse } from 'secure-json-parse';
import { schema } from '../config/schema';
import { TCreateHash, TValidateHash, TMessage, TPing } from '../types';
import { validateInput } from '../lib/validateInput';
import { RabbitMessage } from '../types/enum_rabbitMessage';

export const parser = (input: string): TValidateHash | TCreateHash | TPing | undefined => {
	try {
		const message = parse(input, undefined, { protoAction: 'remove', constructorAction: 'remove' });
		const message_name = <TMessage>validateInput(message.message_name, schema.message_name);
		const validatedMessage = message_name === RabbitMessage.PING ? <TPing>validateInput(message, schema.ping):
			message_name === RabbitMessage.ARGON_VALIDATE_HASH ? <TValidateHash>validateInput(message, schema.validateHash) : <TCreateHash>validateInput(message, schema.createHash);
		return validatedMessage;
	} catch (e) {
		return undefined;
	}

};