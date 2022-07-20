import { parse } from 'secure-json-parse';
import { schema } from '../config/schema';
import { TSQLOnly, TFull, TMessageName, TPing } from '../types';
import { validateInput } from '../lib/validateInput';
import { RabbitMessage } from '../types/enum_rabbitMessage';

export const parser = (input: string): TFull | TSQLOnly | TPing | undefined => {
	try {
		const message = parse(input, undefined, { protoAction: 'remove', constructorAction: 'remove' });
		const message_name = <TMessageName>validateInput(message.message_name, schema.message_name);
		const validatedMessage = message_name === RabbitMessage.PING ? <TPing>validateInput(message, schema.ping):
			message_name === RabbitMessage.BACKUP_FULL_BACKUP ? <TFull>validateInput(message, schema.full) : <TSQLOnly>validateInput(message, schema.sqlOnly);
		return validatedMessage;
	} catch (e) {
		return undefined;
	}

};