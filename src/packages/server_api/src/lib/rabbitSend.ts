import { rabbitMq } from '../config/rabbitMq';
import { RABBITMQ_QUEUE_NAME_EMAILER } from '../config/env';
import { TRabbitEmail } from '../types';

export const send_email = async (data: TRabbitEmail): Promise<void> => {
	const connection = await rabbitMq.getConnection();
	await connection.assertQueue(RABBITMQ_QUEUE_NAME_EMAILER, { durable: true, messageTtl: 10000 });
	connection.sendToQueue(RABBITMQ_QUEUE_NAME_EMAILER, Buffer.from(JSON.stringify(data)), { persistent: true });
};