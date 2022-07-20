import { schema_shared } from '../shared/shared_schema';
import * as joi from 'types-joi';

class Admin {
	
	private readonly regex_session = /^session:[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/;
	private readonly regex_fileName = /^mealpedant_\d{4}-\d{2}-\d{2}_\d{2}\.\d{2}\.\d{2}_LOGS_(PHOTOS_)?REDIS_SQL_[0-9a-fA-F]{8}\.tar(\.gz)?\.gpg$/;
	private readonly fileName = joi.string().required().regex(this.regex_fileName).label('incorrect filename');
	private readonly sessionName = joi.string().required().regex(this.regex_session).label('incorrect sessionId data');

	readonly authenticate = joi.object({
		password: schema_shared.password,
		token: schema_shared.token.required(),
		twoFABackup: joi.boolean().strict(true),
	}).required();

	readonly backup = joi.object({
		withPhoto: joi.boolean().strict(true).required().label('backup option'),
	}).required();

	readonly backupFilename = joi.object({
		fileName: this.fileName
	}).required();

	// This needs work
	readonly rateLimitDelete = joi.object({
		client: joi.alternatives([ joi.string().email(), joi.string().ip() ]).required()

	}).required();

	readonly sendEmail = joi.object({
		userAddress: joi.array().items(joi.string().email()).required().min(1).label('email address'),
		emailTitle: schema_shared.stringRequired.label('title required'),
		lineOne: schema_shared.stringRequired.label('line one required'),
		lineTwo: joi.string().min(1).label('line two error'),
		link: joi.string().uri().label('link error'),
		button: joi.string().min(1).label('button text error'),
	}).required();
	
	readonly session = joi.object({
		email: schema_shared.email
	}).required();

	readonly sessionDelete = joi.object({
		session: this.sessionName
	}).required();

	readonly userPatch = joi.object({
		patch: joi.object({
			active: joi.boolean().strict(true).allow(null).label('active'),
			attempt: joi.boolean().strict(true).allow(null).label('attempt'),
			passwordResetId: joi.string().regex(/^\d+$/).allow(null).label('passwordResetId'),
			reset: joi.object({ withEmail: joi.boolean() }).allow(null).label('reset'),
			tfaSecret: joi.boolean().strict(true).allow(null).label('tfaSecret'),
		}),
		email: schema_shared.email
	}).required();

}

export const schema_admin = new Admin();

export type TAdminAuthenticate = joi.InterfaceFrom<typeof schema_admin.authenticate>
export type TAdminBackup = joi.InterfaceFrom<typeof schema_admin.backup>
export type TAdminBackupFilename = joi.InterfaceFrom<typeof schema_admin.backupFilename>
export type TAdminRateLimitDelete = joi.InterfaceFrom<typeof schema_admin.rateLimitDelete>
export type TAdminSendEmail = joi.InterfaceFrom<typeof schema_admin.sendEmail>
export type TAdminSession = joi.InterfaceFrom<typeof schema_admin.session>
export type TAdminSessionDelete = joi.InterfaceFrom<typeof schema_admin.sessionDelete>
export type TAdminUserPatch = joi.InterfaceFrom<typeof schema_admin.userPatch>