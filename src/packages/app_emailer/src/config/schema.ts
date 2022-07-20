import * as joi from 'joi';
import { RabbitMessage } from '../types/enum_rabbitMessage';

class Schema {

	readonly #email = joi.string().email().lowercase().required().label('email required');
	readonly #id_regex = /^[0-9]+$/;
	readonly #verifyString_regex = /^[0-9a-fA-F]{256}$/;
	readonly #id = joi.string().regex(this.#id_regex);
	readonly #ipId = this.#id.label('ipId').required();
	readonly #userAgentId = this.#id.label('userAgentId').required();
	readonly #userId = this.#id.label('userId').required();
	readonly #verifyString = joi.string().trim().required().regex(this.#verifyString_regex).label('incorrect verification data');
	readonly #firstName = joi.string().required().label('first name');
	readonly #baseUser = joi.object({
		email: this.#email,
		ipId: this.#ipId,
		userId: this.#userId,
		userAgentId: this.#userAgentId,
		firstName: this.#firstName
	}).required();

	#data_label (name: RabbitMessage): string {
		return `${name}-data`;
	}

	readonly change_password = joi.object({
		message_name: joi.string().valid(RabbitMessage.EMAIL_CHANGE_PASSWORD).required().label('message'),
		data: this.#baseUser.label(this.#data_label(RabbitMessage.EMAIL_CHANGE_PASSWORD))
	}).required().label(RabbitMessage.EMAIL_CHANGE_PASSWORD);

	readonly login_attempt = joi.object({
		message_name: joi.string().valid(RabbitMessage.EMAIL_LOGIN_ATTEMPT).required().label('message'),
		data: this.#baseUser.label(this.#data_label(RabbitMessage.EMAIL_LOGIN_ATTEMPT))
	}).required().label(RabbitMessage.EMAIL_LOGIN_ATTEMPT);

	readonly reset = joi.object({
		message_name: joi.string().valid(RabbitMessage.EMAIL_RESET).required().label('message'),
		data: joi.object({
			resetString: this.#verifyString,
			email: this.#email,
			ipId: this.#ipId,
			userAgentId: this.#userAgentId,
			userId: this.#userId,
			firstName: this.#firstName,
		}).required().label(this.#data_label(RabbitMessage.EMAIL_RESET))
	}).required().label(RabbitMessage.EMAIL_RESET);

	readonly twoFABackup = joi.object({
		message_name: joi.string().valid(RabbitMessage.EMAIL_TWO_FA_BACKUP).required().label('message'),
		data: joi.object({
			email: this.#email,
			ipId: this.#ipId,
			userId: this.#userId,
			userAgentId: this.#userAgentId,
			firstName: this.#firstName,
			enabled: joi.boolean().required()
		}).required().label(this.#data_label(RabbitMessage.EMAIL_TWO_FA_BACKUP))
	}).required().label(RabbitMessage.EMAIL_TWO_FA_BACKUP);

	readonly twoFA = joi.object({
		message_name: joi.string().valid(RabbitMessage.EMAIL_TWO_FA).required().label('message'),
		data: joi.object({
			email: this.#email,
			ipId: this.#ipId,
			userId: this.#userId,
			userAgentId: this.#userAgentId,
			firstName: this.#firstName,
			enabled: joi.boolean().required()
		}).required().label(this.#data_label(RabbitMessage.EMAIL_TWO_FA))
	}).required().label(RabbitMessage.EMAIL_TWO_FA);

	readonly verify = joi.object({
		message_name: joi.string().valid(RabbitMessage.EMAIL_VERIFY).required().label('message'),
		data: joi.object({
			verifyString: this.#verifyString,
			email: this.#email,
			ipId: this.#ipId,
			userAgentId: this.#userAgentId,
			firstName: this.#firstName,
		}).required().label(this.#data_label(RabbitMessage.EMAIL_VERIFY))
	}).required().label(RabbitMessage.EMAIL_VERIFY);

	readonly custom_admin = joi.object({
		message_name: joi.string().valid(RabbitMessage.EMAIL_CUSTOM_ADMIN).required().label('message'),
		data: joi.object({
			email: this.#email,
			firstName: this.#firstName,
			ipId: this.#ipId,
			userAgentId: this.#id,
			lineOne: joi.string().required().label('lineOne'),
			lineTwo: joi.string().label('lineTwo'),
			title: joi.string().required().label('title'),
			buttonText: joi.string().label('buttonText'),
			buttonLink: joi.string().label('buttonLink'),
		}).required().label(this.#data_label(RabbitMessage.EMAIL_CUSTOM_ADMIN))
	}).required().label(RabbitMessage.EMAIL_CUSTOM_ADMIN);

	readonly message = joi.string()
		.valid(
			RabbitMessage.EMAIL_CHANGE_PASSWORD,
			RabbitMessage.EMAIL_CUSTOM_ADMIN,
			RabbitMessage.EMAIL_LOGIN_ATTEMPT,
			RabbitMessage.EMAIL_RESET,
			RabbitMessage.EMAIL_TWO_FA_BACKUP,
			RabbitMessage.EMAIL_TWO_FA,
			RabbitMessage.EMAIL_VERIFY,
		)
		.required()
		.label('message');

}

export const schema = new Schema();
