import * as joi from 'joi';
import { RabbitMessage } from '../types/enum_rabbitMessage';

class Schema {

	readonly #originalRegex = /^\d{4}-\d{2}-\d{2}_(D|J)_O_[a-fA-F0-9]{16}.jpeg$/;

	readonly convertPhoto = joi.object({
		message_name: joi.string().valid(RabbitMessage.PHOTO_CONVERT).required().label('message'),
		data: joi.object({
			originalFileName: joi.string().regex(this.#originalRegex).required().label('originalFileName'),
		}).required().label(`${RabbitMessage.PHOTO_CONVERT}-data`)
	}).required().label(RabbitMessage.PHOTO_CONVERT);

	readonly ping = joi.object({
		message_name: joi.string().valid(RabbitMessage.PING).required().label(RabbitMessage.PING),
		
	}).required().label('ping');

	readonly message_name = joi.string().valid(RabbitMessage.PING, RabbitMessage.PHOTO_CONVERT).required().label('message_name');

}

export const schema = new Schema();
