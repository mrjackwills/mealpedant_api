import { RABBITMQ_PASSWORD, RABBITMQ_VHOST, RABBITMQ_USERNAME, RABBITMQ_QUEUE_NAME_EMAILER, RABBITMQ_HOSTNAME, RABBITMQ_PORT } from './env';
import ampq from 'amqplib';

class RabbitMqConnection {

	readonly queueName = RABBITMQ_QUEUE_NAME_EMAILER;

	#connectionDetails = {
		hostname: RABBITMQ_HOSTNAME,
		username: RABBITMQ_USERNAME,
		password: RABBITMQ_PASSWORD,
		port: RABBITMQ_PORT,
		vhost: RABBITMQ_VHOST,
		heartbeat: 60,
	};
	
	#channel?: ampq.Channel;
	#connection?: ampq.Connection;

	async getConnection (): Promise<ampq.Channel> {
		if (!this.#channel) {
			this.#connection = await ampq.connect(this.#connectionDetails);
			this.#channel = await this.#connection.createChannel();
			await this.#channel.assertQueue(this.queueName, { durable: true, messageTtl: 10000 });
			this.#channel.prefetch(1);
		}
		return this.#channel;
	}

	async closeConnection (): Promise<void> {
		await this.#channel?.close();
		await this.#connection?.close();
	}
}

export const rabbitMq = new RabbitMqConnection();