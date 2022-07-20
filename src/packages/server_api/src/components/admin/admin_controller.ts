import { checkPasswordAndToken } from '../../lib/checkAuthentication';
import { cleanEmail, fileExists, randomHex, } from '../../lib/helpers';
import { customError, customTypeError } from '../../config/customError';
import { ErrorMessages } from '../../types/enum_error';
import { HttpCode } from '../../types/enum_httpCode';
import { limiter } from '../../config/rateLimiter';
import { LOCATION_BACKUP, MODE_ENV_TEST } from '../../config/env';
import { log } from '../../config/log';
import { promises as fs } from 'fs';
import { rabbit_backup } from '../../lib/rabbitRpc';
import { send_email } from '../../lib/rabbitSend';
import { schema_admin } from './admin_schema';
import { send } from '../../lib/send';
import { uptime } from 'os';
import { validate_input } from '../../lib/validateinput';
import { sharedQueries } from '../../components/shared/shared_queries';

import {
	TAdminAuthenticate,
	TAdminBackup,
	TAdminBackupFilename,
	TAdminRateLimitDelete,
	TAdminSendEmail,
	TAdminSession,
	TAdminSessionDelete,
	TAdminUserPatch,
} from './admin_schema';

import { adminQueries } from './admin_queries';

import {
	IAdminUptime,
	ListOfBackups,
	RequestMethod,
	TPassportDeserializedUser,
} from '../../types';
import { RabbitMessage } from '../../types/enum_rabbitMessage';

const getListOfAllBackups: ListOfBackups = async () => {
	// Create empty array
	const dataForClient = [];
	// Get promise contents of backup dir
	const files = await fs.readdir(`${LOCATION_BACKUP}`);
	// Loop over contents, add filesize and push to array
	for (const filename of files) {
		// eslint-disable-next-line no-await-in-loop
		const filesize = await fs.stat(`${LOCATION_BACKUP}/${filename}`);
		// push filename and filesize (in mb to 2dp) to array
		dataForClient.push({ filename, filesize: `${(Number(filesize.size)/1024/1024).toFixed(2)}` });
	}
	// Sort in alphabetical reverse order - newest to oldest
	dataForClient.sort().reverse();
	return dataForClient;
};

export const admin_get: RequestMethod = async (_req, res) => {
	send({ res });
};

export const backup_delete: RequestMethod = async (req, res) => {

	const body = <TAdminBackupFilename>validate_input(req.body, schema_admin.backupFilename);

	const fileName =`${LOCATION_BACKUP}/${body.fileName}`;
	const valid = await fileExists(fileName);
	if (!valid) throw customError(HttpCode.BAD_REQUEST, ErrorMessages.FILE_NOT_FOUND);
	await fs.unlink(fileName);
	send({ res });
};

export const backup_get: RequestMethod = async (_req, res) => {
	const response = await getListOfAllBackups();
	send({ res, response });
};

export const backup_post: RequestMethod = async (req, res) => {
	const body = <TAdminBackup>validate_input(req.body, schema_admin.backup);
	const backupType = body.withPhoto ? RabbitMessage.BACKUP_FULL_BACKUP : RabbitMessage.BACKUP_SQL_BACKUP;
	const backupCreated = await rabbit_backup(backupType);
	if (!backupCreated) throw customError(HttpCode.INTERNAL_SERVER_ERROR);

	send({ res });

};

export const backupDownload_get: RequestMethod = async (req, res) => {
	try {
		const params = <TAdminBackupFilename>validate_input(req.params, schema_admin.backupFilename);
		const file = params.fileName;
		const fileName =`${LOCATION_BACKUP}/${file}`;
		const valid = await fileExists(fileName);
		if (!valid) throw customError(HttpCode.BAD_REQUEST, ErrorMessages.FILE_NOT_FOUND);
		res.sendFile(`${file}`, { root: LOCATION_BACKUP });
	} catch (e) {
		log.error(e);
		throw e;
	}
};

export const email_get: RequestMethod = async (_req, res) => {
	const emails = await adminQueries.select_activeEmails();
	send({ res, response: { emails } });
};

export const email_post: RequestMethod = async (req, res) => {

	const body = <TAdminSendEmail>validate_input(req.body, schema_admin.sendEmail);

	const { ipId, userAgentId } = await sharedQueries.select_ipIdUserAgentId_transaction(req);
	
	for (const emailAddress of body.userAddress) {
		const email = emailAddress as string;
		// eslint-disable-next-line no-await-in-loop
		const user = await sharedQueries.select_userActiveIdByEmail(email);
		if (!user) throw customError(HttpCode.BAD_REQUEST, ErrorMessages.USER_NOT_FOUND);
		
		// Client side need to change buttonLink so that it is just format /something/somethingelse, mealpedant.com gets pre-fixed in the email template
		send_email({ message_name: RabbitMessage.EMAIL_CUSTOM_ADMIN, data: {
			email,
			firstName: user.first_name,
			userId: user.registered_user_id,
			ipId,
			userAgentId,
			title: body.emailTitle,
			lineOne: body.lineOne,
			lineTwo: body.lineTwo,
			buttonText: body.button,
			buttonLink: body.link
		} });
	}
	send({ res });
};

export const error_get: RequestMethod = async (_req, res) => {
	const response = await adminQueries.select_error();
	send({ res, response });

};

export const limit_get: RequestMethod = async (_req, res) => {
	const response = { limits: await adminQueries.select_allUserLimits() };
	send({ res, response });
};

export const limit_patch: RequestMethod = async (req, res) => {

	const body = <TAdminRateLimitDelete>validate_input(req.body, schema_admin.rateLimitDelete);

	await limiter.delete(body.client);
	send({ res });
};

export const memory_get: RequestMethod = async (_req, res) => {
	const memory = process.memoryUsage();
	const nodeUptime = Math.trunc(process.uptime());
	const serverUptime = uptime();
	const response: IAdminUptime = {
		rss: (memory.rss / 1024 / 1024).toFixed(2),
		heapUsed: (memory.heapUsed / 1024 / 1024).toFixed(2),
		heapTotal: (memory.heapTotal / 1024 / 1024).toFixed(2),
		external: (memory.external / 1024 / 1024).toFixed(2),
		nodeUptime,
		serverUptime
	};
	send({ res, response });
};

export const memory_put: RequestMethod = async (_req, res) => {
	if (global.gc) global.gc();
	send({ res });
};

export const restart_put: RequestMethod = async (req, res) => {
	<TAdminAuthenticate>validate_input(req.body, schema_admin.authenticate);
	await checkPasswordAndToken(req);
	send({ res });
	if (!MODE_ENV_TEST) process.exit();
};

export const user_get: RequestMethod = async (_req, res) => {
	const response = await adminQueries.select_allUser();
	send({ res, response });
};

export const session_delete: RequestMethod = async (req, res) => {
	const body = <TAdminSessionDelete>validate_input(req.body, schema_admin.sessionDelete);
	await adminQueries.delete_userSession({ sessionName: body.session, currentSession: req.sessionID });
	send({ res });
};

export const session_get: RequestMethod = async (req, res) => {
	const params = <TAdminSession>validate_input(req.params, schema_admin.session);
	const email = cleanEmail(params.email);
	const validUser = await sharedQueries.select_userActiveIdByEmail(email);
	if (!validUser) throw customError(HttpCode.BAD_REQUEST, ErrorMessages.USER_NOT_FOUND);

	const response = await adminQueries.select_userSession({ userId: validUser.registered_user_id, currentSession: req.sessionID as string });
	send({ res, response });
};

// TODo this is dirty - redo into own routes
export const user_patch: RequestMethod = async (req, res) => {

	const body = <TAdminUserPatch>validate_input(req.body, schema_admin.userPatch);
	const user = <TPassportDeserializedUser>req.user;

	const email = cleanEmail(body.email);
	const userToPatch = await adminQueries.select_userIdByEmail(email);
	if (!userToPatch) throw customError(HttpCode.BAD_REQUEST, ErrorMessages.USER_NOT_FOUND);
	if (!body.patch) throw customTypeError('user_patch: !body.patch', HttpCode.BAD_REQUEST);

	if (Object.prototype.hasOwnProperty.call(body.patch, 'active')) {
		if (userToPatch.registered_user_id === user.registered_user_id) throw customError(HttpCode.BAD_REQUEST, ErrorMessages.SELF);
		await adminQueries.update_userActive (userToPatch.registered_user_id, body.patch.active as boolean);
	}
		
	// Revoke password reset
	else if (Object.prototype.hasOwnProperty.call(body.patch, 'passwordResetId')) {
		const resetInProgress = await sharedQueries.select_passwordResetProcess(email);
		// MAYBE change this to if match then update, else throw?
		if (!resetInProgress) throw customError(HttpCode.BAD_REQUEST, ErrorMessages.PASSWORD_RESET_INCORRECT);
		if (resetInProgress.password_reset_id!== body.patch.passwordResetId) throw customError(HttpCode.BAD_REQUEST, ErrorMessages.PASSWORD_RESET_INCORRECT);
		await adminQueries.update_userPasswordConsumed(body.patch.passwordResetId);
	}

	// Force a password reset, with/without password
	else if (Object.prototype.hasOwnProperty.call(body.patch, 'reset')) {
		if (userToPatch.registered_user_id === user.registered_user_id) throw customError(HttpCode.BAD_REQUEST, ErrorMessages.SELF);
		const resetInProgress = await sharedQueries.select_passwordResetProcess(email);
		if (resetInProgress) throw customError(HttpCode.BAD_REQUEST, ErrorMessages.PASSWORD_RESET_INPROGRESS);
		const [ { ipId, userAgentId }, resetString ] = await Promise.all([
			sharedQueries.select_ipIdUserAgentId_transaction(req),
			randomHex(256),
		]);
		await sharedQueries.insert_passwordReset ({ userId: userToPatch.registered_user_id, resetString, ipId, userAgentId });
		if (body.patch.reset?.withEmail) {
			send_email({ message_name: RabbitMessage.EMAIL_RESET, data: { email, ipId, userAgentId, userId: userToPatch.registered_user_id, firstName: userToPatch.first_name, resetString } });
		}
	}
	
	// delete a users twofa status
	else if (Object.prototype.hasOwnProperty.call(body.patch, 'tfaSecret')) {
		if (userToPatch.registered_user_id === user.registered_user_id) throw customError(HttpCode.BAD_REQUEST, ErrorMessages.SELF);
		await sharedQueries.delete_twoFA_transaction(userToPatch.registered_user_id);
	}

	// Set login attempts to 0, although update_userAttmept accepts second param attempt
	else if (Object.prototype.hasOwnProperty.call(body.patch, 'attempt')) await adminQueries.update_userAttempt(userToPatch.registered_user_id);

	send({ res });
};
