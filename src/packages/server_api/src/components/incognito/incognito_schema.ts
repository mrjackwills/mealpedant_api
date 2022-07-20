import { INVITE_USER } from '../../config/env';
import * as joi from 'types-joi';
import { schema_shared } from '../shared/shared_schema';

class Incognito {

	// Forogt password, body is just email

	readonly #inviteRegex = new RegExp(INVITE_USER);

	readonly email = joi.object({
		email: schema_shared.email
	}).required();

	// Schema for registering users

	readonly newUser = joi.object({
		firstName: joi.string().trim().required().label('A first name is required'),
		lastName: joi.string().trim().required().label('A last name is required'),
		email: schema_shared.email,
		password: schema_shared.password,
		invite: joi.string().required().regex(this.#inviteRegex).label('the invite code provided is incorrect'),
	}).required();

	// reset password schema
	readonly password = joi.object({
		// password: schema_shared.password,
		token: schema_shared.token.allow(''),
		newPassword: schema_shared.password,
		twoFABackup: joi.boolean().strict(true).allow(null)
	}).required();

	// Used by verify and reset
	readonly resetString = joi.object({
		resetString: schema_shared.base64_256_string
	}).required();

	// Used by verify and reset
	readonly verifyString = joi.object({
		verifyString: schema_shared.base64_256_string
	}).required();

}
export const schema_incogntio = new Incognito();

export type TSchemaIncognitoEmail = joi.InterfaceFrom<typeof schema_incogntio.email>
export type TSchemaIncognitoNewUser = joi.InterfaceFrom<typeof schema_incogntio.newUser>
export type TSchemaIncognitoPassword = joi.InterfaceFrom<typeof schema_incogntio.password>
export type TSchemaIncognitoResetString = joi.InterfaceFrom<typeof schema_incogntio.resetString>
export type TSchemaIncognitoVerifyString = joi.InterfaceFrom<typeof schema_incogntio.verifyString>