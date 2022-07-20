import * as joi from 'joi';
import { RabbitMessage } from '../types/enum_rabbitMessage';

class Schema {

	readonly sqlOnly = joi.object({
		message_name: joi.string().valid(RabbitMessage.BACKUP_SQL_BACKUP).required().label('message'),
	}).required().label(RabbitMessage.BACKUP_SQL_BACKUP);

	readonly full = joi.object({
		message_name: joi.string().valid(RabbitMessage.BACKUP_FULL_BACKUP).required().label('message'),
	}).required().label(RabbitMessage.BACKUP_FULL_BACKUP);

	readonly ping = joi.object({
		message_name: joi.string().valid(RabbitMessage.PING).required().label('message'),
	}).required().label(RabbitMessage.PING);

	readonly message_name = joi.string().valid(RabbitMessage.PING, RabbitMessage.BACKUP_FULL_BACKUP, RabbitMessage.BACKUP_SQL_BACKUP).required().label('message_name');
}

export const schema = new Schema();
