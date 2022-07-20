import * as joi from 'joi';
import { RabbitMessage } from '../types/enum_rabbitMessage';

class Schemas {

	readonly convertPhoto = joi.object({
		message_name: joi.string().valid(RabbitMessage.PHOTO_CONVERT).required().label('message_name'),
		data: joi.object({
			response: joi.string().required().label('convertPhoto-response'),
		}).required().label('convertPhoto-data')
	}).required().label('convertPhoto');

	readonly backup = joi.object({
		message_name: joi.string().valid(RabbitMessage.BACKUP_FULL_BACKUP, RabbitMessage.BACKUP_SQL_BACKUP).required().label('message_name'),
		data: joi.object({
			response: joi.boolean().required().label('backup-response'),
		}).required().label('backup-data')
	}).required().label('backup');

	readonly error = joi.object({
		message_name: joi.string().valid(RabbitMessage.ERROR).required().label('message_name'),
		data: joi.object({
			error: joi.string().required().min(1).label('error-response'),
		}).required().label('error-data')
	}).required().label('error');
	
	readonly validateResponse = joi.object({
		message_name: joi.string().valid(RabbitMessage.ARGON_VALIDATE_HASH).required().label('message_name'),
		data: joi.object({
			response: joi.boolean().required().label('validateResponse-response'),
		}).required().label('validateResponse-data')
	}).required().label('validateResponse');

	readonly createResponse = joi.object({
		message_name: joi.string().valid(RabbitMessage.ARGON_CREATE_HASH).required().label('message_name'),
		data: joi.object({
			response: joi.string().required().label('createResponse-response'),
		}).required().label('createResponse-data')
	}).required().label('createResponse');
	
}

export const schema = new Schemas();