import { parse } from 'secure-json-parse';
import { schema } from '../config/schema';
import { TConvertPhoto, TPing, TMessage } from '../types';
import { validateInput } from './validateInput';
import { RabbitMessage } from '../types/enum_rabbitMessage';

export const parser = (input: string): TConvertPhoto | TPing | undefined => {
	try {
		const message = parse(input, undefined, { protoAction: 'remove', constructorAction: 'remove' });
		const message_name = <TMessage>validateInput(message.message_name, schema.message_name);
		const validatedMessage = message_name === RabbitMessage.PING ? <TPing>validateInput(message, schema.ping): <TConvertPhoto>validateInput(message, schema.convertPhoto);
		return validatedMessage;
	} catch (e) {
		return undefined;
	}

};