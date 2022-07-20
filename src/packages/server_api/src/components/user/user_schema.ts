import * as joi from 'types-joi';
import { schema_shared } from '../shared/shared_schema';

class User {

	changePassword = joi.object({
		password: schema_shared.password,
		token: schema_shared.token,
		newPassword: schema_shared.password
	}).required();

	// TODO this can be combined with admin authenticate
	twoFAAlwaysRequired = joi.object({
		alwaysRequired: joi.boolean().strict(true).required(),
		token: schema_shared.token.allow(null),
		password: schema_shared.password.optional().allow(null),
		twoFABackup: joi.boolean().strict(true).allow(null)
	}).required();
	
	twoFASetup = joi.object({
		token: schema_shared.token.required()
	}).required();

}

export const schema_user = new User();

export type TUserSchemaChangePassword = joi.InterfaceFrom<typeof schema_user.changePassword>
export type TUserSchemaTwoFASetup = joi.InterfaceFrom<typeof schema_user.twoFASetup>
export type TUserSchemaTwoFAAlwaysRequired = joi.InterfaceFrom<typeof schema_user.twoFAAlwaysRequired>