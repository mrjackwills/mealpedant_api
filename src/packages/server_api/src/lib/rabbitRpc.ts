
import { HttpCode } from '../types/enum_httpCode';
import { TRabbitData, TVerifyPassword, TCreateHash } from '../types';
import { RABBITMQ_QUEUE_NAME_ARGON, RABBITMQ_QUEUE_NAME_PHOTO_CONVERTOR, RABBITMQ_QUEUE_NAME_BACKUP } from '../config/env';
import { rabbitMq } from '../config/rabbitMq';
import { customError } from '../config/customError';
import { ErrorMessages } from '../types/enum_error';
import { parser } from './parser';
import { randomUUID } from 'crypto';
import { log } from '../config/log';
import { RabbitMessage } from '../types/enum_rabbitMessage';

const rpcSender = async (input: TRabbitData, queueName: string, ttl: number) : Promise<string> => {

	const channel = await rabbitMq.getConnection();
	const queue = await channel.assertQueue('', { exclusive: true, messageTtl: ttl });
	return new Promise((resolve, reject) => {
		const timeout = setTimeout(() => {
			reject(customError(HttpCode.INTERNAL_SERVER_ERROR, <ErrorMessages><unknown>`${ErrorMessages.RABBITMQ_TIMEOUT}: ${queueName}`));
		}, ttl);
			
		const clear = [ RabbitMessage.PING, RabbitMessage.BACKUP_FULL_BACKUP, RabbitMessage.BACKUP_SQL_BACKUP ];
		const toSend = {
			message_name: input.message_name,
			data: clear.includes(input.message_name) ? undefined : {
				...input.data,
				// set as undefined to strip the rabbit_uuid key/value from the object
				rabbit_uuid: undefined
			}
		};

		channel.sendToQueue(queueName, Buffer.from(JSON.stringify(toSend)), {
			replyTo: queue.queue,
			correlationId: input.data.rabbit_uuid,
		});
	
		channel.consume(queue.queue, (msg) => {
			if (!msg) return;
			channel.ack(msg);
			if (msg.properties.correlationId === input.data.rabbit_uuid) {
				clearTimeout(timeout);
				resolve(msg.content.toString());
			}
	
		});
	});
};

export const rabbit_ping = async (): Promise<void> => {
	try {
		const rabbit_uuid = randomUUID({ disableEntropyCache: true });
		const toSend = { message_name: RabbitMessage.PING, data: { rabbit_uuid } } as const;
		await Promise.all([
			rpcSender(toSend, RABBITMQ_QUEUE_NAME_PHOTO_CONVERTOR, 1500),
			rpcSender(toSend, RABBITMQ_QUEUE_NAME_ARGON, 1500),
			rpcSender(toSend, RABBITMQ_QUEUE_NAME_BACKUP, 1500)
		]);
	} catch (e) {
		log.error(e);
	}
};

export const rabbit_backup = async (backupType: RabbitMessage.BACKUP_SQL_BACKUP | RabbitMessage.BACKUP_FULL_BACKUP): Promise<boolean|void> => {
	try {
		const rabbit_uuid = randomUUID({ disableEntropyCache: true });
		const toSend = { message_name: backupType, data: { rabbit_uuid } } as const;
		const ttl = 20000;
		const message = await rpcSender(toSend, RABBITMQ_QUEUE_NAME_BACKUP, ttl);
		const output = parser.backup(message);
		return output;
	} catch (e) {
		log.error(e);
	}
};

export const rabbit_photoConvertor = async (input: string) : Promise<string> => {
	const rabbit_uuid = randomUUID({ disableEntropyCache: true });
	const message = await rpcSender({
		message_name: RabbitMessage.PHOTO_CONVERT,
		data: {
			originalFileName: input,
			rabbit_uuid
		}
	}, RABBITMQ_QUEUE_NAME_PHOTO_CONVERTOR, 5000);
	const output = parser.photo(message);
	return output;
};

export const rabbit_validateHash = async (data: TVerifyPassword) : Promise<boolean> => {
	const rabbit_uuid = randomUUID({ disableEntropyCache: true });
	const message = await rpcSender({
		message_name: RabbitMessage.ARGON_VALIDATE_HASH,
		data: {
			attempt: data.attempt,
			known_password_hash: data.known_password_hash,
			rabbit_uuid
		}
	}, RABBITMQ_QUEUE_NAME_ARGON, 5000);
	const output = parser.validate(message);
	return output;
};

export const rabbit_createHash = async (data: TCreateHash) : Promise<string> => {
	const rabbit_uuid = randomUUID({ disableEntropyCache: true });
	const message = await rpcSender({
		message_name: RabbitMessage.ARGON_CREATE_HASH,
		data: {
			password: data.password,
			rabbit_uuid
		}
	},
	RABBITMQ_QUEUE_NAME_ARGON, 5000);
	const output = parser.create(message);
	return output;
};
