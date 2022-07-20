import { log } from '../config/log';
import { rabbitMq } from '../config/rabbitmq';
import { parser } from '../lib/parser';
import { Channel } from 'amqplib';
import { backupSpawner } from '../lib/workerSpawn';
import { RabbitMessage } from '../types/enum_rabbitMessage';

class RabbitServer {

	#channel!: Channel;
	#connection_retry_amount = 0;

	async #sleep (ms = 1000): Promise<void> {
		return new Promise((resolve) => setTimeout(resolve, ms));
	}

	async #init (): Promise<void> {
		try {
			this.#channel = await rabbitMq.getConnection();
		} catch (e) {
			const message = e instanceof Error ? e.message : 'getConnection Error';
			log.error(message);
			if (this.#connection_retry_amount <= 20) {
				log.error('sleep 1000ms, then try re-connecting');
				this.#connection_retry_amount ++;
				await this.#sleep();
				return this.#init();
			}
			else {
				process.exit();
			}
		}
	}

	async listen () : Promise<void> {
		await this.#init();
		log.verbose(`waiting for messages in queue_name: ${rabbitMq.queueName}`);
		this.#channel.consume(rabbitMq.queueName, async (msg) => {
			if (!msg) return;
			try {
				const parsedMessage = parser(msg.content.toString());
				if (!parsedMessage) throw Error('Parsing error');
				const messageName = parsedMessage.message_name;
				
				const response = messageName === RabbitMessage.PING ? RabbitMessage.PONG : messageName === RabbitMessage.BACKUP_FULL_BACKUP ?
					await backupSpawner.create('FULL'): await backupSpawner.create('SQL_ONLY');

				this.#channel.sendToQueue(
					msg.properties.replyTo,
					Buffer.from(JSON.stringify({ message_name: messageName, data: { response } })),
					{ correlationId: msg.properties.correlationId }
				);
			} catch (e) {
				const message = e instanceof Error ? e.message: RabbitMessage.ERROR;
				this.#channel.sendToQueue(
					msg.properties.replyTo,
					Buffer.from(JSON.stringify({ message_name: RabbitMessage.ERROR, data: { error: message } })),
					{ correlationId: msg.properties.correlationId }
				);
				log.error(e);
			} finally {
				this.#channel.ack(msg);
			}
		});
	}
}

export const rabbitServer = new RabbitServer();