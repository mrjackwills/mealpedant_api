import * as joi from 'types-joi';

class Shared {
	readonly regex_base64_256 = /^[0-9a-fA-F]{256}$/;
	readonly regex_imageOriginal = /^\d{4}-\d{2}-\d{2}_(D|J)_O_[a-fA-F0-9]{16}.jpeg$/;
	readonly regex_imageConverted = /^\d{4}-\d{2}-\d{2}_(D|J)_C_[a-fA-F0-9]{16}.jpeg$/;
	readonly regex_token = /^([0-9]{3})(?:\s?)([0-9]{3})$/;
	readonly regex_Backuptoken = /^[0-9a-fA-F]{16}$/;

	readonly base64_256_string = joi.string().trim().required().regex(this.regex_base64_256).label('incorrect verification data');
	readonly email = joi.string().required().email().lowercase().label('email address');
	readonly imageNameConverted = joi.string().regex(this.regex_imageConverted).label('incorrect filename');
	readonly imageNameOriginal = joi.string().regex(this.regex_imageOriginal).label('incorrect filename');
	readonly password = joi.string().min(10).required().label('passwords are required to be 10 characters minimum');
	readonly stringRequired = joi.string().required().min(1);
	readonly token = joi.string().trim().regex(this.regex_token).label('token format incorrect');
	readonly backupToken = joi.string().trim().regex(this.regex_Backuptoken).label('backup token format incorrect');
	readonly eitherToken = joi.alternatives([ this.token, this.backupToken ]);

	readonly login = joi.object({
		email: this.email,
		password: this.password,
		token: this.eitherToken.allow(null),
		twoFABackup: joi.boolean().strict(true).allow(null),
		remember: joi.boolean().strict(true).allow(null)
	}).required();

}

export const schema_shared = new Shared();
export type TSchemaLogin = joi.InterfaceFrom<typeof schema_shared.login>