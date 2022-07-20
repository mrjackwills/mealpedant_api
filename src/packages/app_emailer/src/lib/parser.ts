import { parse } from 'secure-json-parse';
import { schema } from '../config/schema';
import { validateInput } from '../lib/validateInput';
import { log } from '../config/log';

import {
	TMessage,
	TMessage2FABackups,
	TMessage2FA,
	TMessageChangePassword,
	TMessageLoginAttempt,
	TMessageReset,
	TMessageVerify,
	TMessageName
} from '../types';
import { RabbitMessage } from '../types/enum_rabbitMessage';

export const parser = (input: string): undefined | TMessage => {
	try {
		const parsedMessage = parse(input, undefined, { protoAction: 'remove', constructorAction: 'remove' });
		const messageName = <TMessageName>validateInput(parsedMessage.message_name, schema.message);

		switch (messageName) {
	
		case RabbitMessage.EMAIL_CHANGE_PASSWORD:
			return <TMessageChangePassword>validateInput(parsedMessage, schema.change_password);
		case RabbitMessage.EMAIL_LOGIN_ATTEMPT:
			return <TMessageLoginAttempt>validateInput(parsedMessage, schema.login_attempt);
		case RabbitMessage.EMAIL_RESET:
			return <TMessageReset>validateInput(parsedMessage, schema.reset);
		case RabbitMessage.EMAIL_TWO_FA_BACKUP:
			return <TMessage2FABackups>validateInput(parsedMessage, schema.twoFABackup);
		case RabbitMessage.EMAIL_TWO_FA:
			return <TMessage2FA>validateInput(parsedMessage, schema.twoFA);
		case RabbitMessage.EMAIL_VERIFY:
			return <TMessageVerify>validateInput(parsedMessage, schema.verify);
		case RabbitMessage.EMAIL_CUSTOM_ADMIN:
			return <TMessageVerify>validateInput(parsedMessage, schema.verify);
		}
	} catch (e) {
		log.error(e);
		return undefined;
	}

};