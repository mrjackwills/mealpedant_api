import { Channel } from 'amqplib';
import { emailer } from '../components/emails';
import { log } from '../config/log';
import { parser } from '../lib/parser';
import { rabbitMq } from '../config/rabbitmq';
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
				log.error('sleep 3000ms, then try re-connecting');
				this.#connection_retry_amount ++;
				await this.#sleep(3000);
				return this.#init();
			}
			else {
				process.exit();
			}
		}
	}

	async listen () : Promise<void> {
		try {
			await this.#init();
			log.verbose(`waiting for messages in queue_name: ${rabbitMq.queueName}`);
			this.#channel.consume(rabbitMq.queueName, async (msg) => {
				if (!msg) return;
				const parsedMessage = parser(msg.content.toString());
				log.debug(parsedMessage);
				switch (parsedMessage?.message_name) {
				case RabbitMessage.EMAIL_CHANGE_PASSWORD:
					await emailer.changePassword(parsedMessage.data);
					break;
				case RabbitMessage.EMAIL_LOGIN_ATTEMPT:
					await emailer.loginAttempt(parsedMessage.data);
					break;
				case RabbitMessage.EMAIL_RESET:
					await emailer.resetPassword(parsedMessage.data);
					break;
				case RabbitMessage.EMAIL_TWO_FA_BACKUP:
					await emailer.twoFABackup(parsedMessage.data);
					break;
				case RabbitMessage.EMAIL_TWO_FA:
					await emailer.twoFA(parsedMessage.data);
					break;
				case RabbitMessage.EMAIL_VERIFY:
					await emailer.verifyAccount(parsedMessage.data);
					break;
				case RabbitMessage.EMAIL_CUSTOM_ADMIN:
					await emailer.customAdminEmail(parsedMessage.data);
					break;
				}
				this.#channel.ack(msg);
			});
		
		} catch (e) {
			log.error(e);
			
		}
	}

}

export const rabbitServer = new RabbitServer();