import * as joi from 'joi';
import { RabbitMessage } from '../types/enum_rabbitMessage';

class Schema {

	#data_label (name: RabbitMessage): string {
		return `${name}-data`;
	}

	readonly createHash = joi.object({
		message_name: joi.string().valid(RabbitMessage.ARGON_CREATE_HASH).required().label('message'),
		data: joi.object({
			password: joi.string().required().min(1).label('password'),
		}).required().label(this.#data_label(RabbitMessage.ARGON_CREATE_HASH))
	}).required().label(RabbitMessage.ARGON_CREATE_HASH);
	
	readonly validateHash = joi.object({
		message_name: joi.string().valid(RabbitMessage.ARGON_VALIDATE_HASH).required().label('message'),
		data: joi.object({
			attempt: joi.string().required().min(1).label('attempt'),
			known_password_hash: joi.string().required().min(1).label('known_password_hash')
		}).required().label(this.#data_label(RabbitMessage.ARGON_VALIDATE_HASH))
	}).required().label(RabbitMessage.ARGON_VALIDATE_HASH);

	readonly ping = joi.object({
		message_name: joi.string().valid(RabbitMessage.PING).required().label('message'),
	}).required().label(RabbitMessage.PING);

	readonly message_name = joi.string().valid(RabbitMessage.ARGON_CREATE_HASH, RabbitMessage.ARGON_VALIDATE_HASH, RabbitMessage.PING).required().label('message_name');
}

export const schema = new Schema();
