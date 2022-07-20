import { authenticator } from 'otplib';
import { checkPasswordAndToken, destroy_session } from '../../lib/checkAuthentication';
import { customError } from '../../config/customError';
import { ErrorMessages } from '../../types/enum_error';
import { HttpCode } from '../../types/enum_httpCode';
import { pwnedPassword } from 'hibp';
import { rabbit_createHash } from '../../lib/rabbitRpc';
import { RabbitMessage } from '../../types/enum_rabbitMessage';
import { randomHex } from '../../lib/helpers';
import { redisQueries } from '../../lib/redisQueries';
import { ResponseMessages } from '../../types/enum_response';
import { schema_shared, TSchemaLogin } from '../shared/shared_schema';
import { schema_user, TUserSchemaChangePassword, TUserSchemaTwoFAAlwaysRequired, TUserSchemaTwoFASetup } from './user_schema';
import { send } from '../../lib/send';
import { send_email } from '../../lib/rabbitSend';
import { sharedQueries } from '../shared/shared_queries';
import { userQueries } from './user_queries';
import { validate_input } from '../../lib/validateinput';
import {
	CreateBackupArray,
	CreateNewBackupCodes,
	TVuexObject,
	RequestMethod,
	TPassportDeserializedUser,
	TwoFAStatus,
	UserObject,
	TRequest,
} from '../../types';

// Set the authenticator to use a window of -/+ 1
authenticator.options = {
	window: 1
};

const createBackupArray: CreateBackupArray= async (backups) => {
	const code = await randomHex(16);
	if (backups.indexOf(code) <0) backups.push(code);
	else await createBackupArray(backups);
	if (backups.length < 10) {
		const recursive = await createBackupArray(backups);
		return recursive;
	}
	else return backups;
};

const createNewBackupCodes: CreateNewBackupCodes = async ({ user, ipId, userAgentId }) => {
	if (!user.two_fa_enabled) throw customError(HttpCode.CONFLICT, ErrorMessages.TWO_FA_NOT_ENABLED);

	const userId = user.registered_user_id;
	await userQueries.delete_twoFABackupCodeAll(userId);

	const backups: Array<string> = [];
	await createBackupArray(backups);
	// Not the best, but argon ram can sky rocket if you do them all at once
	const backupArray: Array<string> = [];
	// eslint-disable-next-line no-await-in-loop
	for (const backup of backups) backupArray.push(await rabbit_createHash({ password: backup }));
	
	await userQueries.insert_twoFABackup_transaction({ userId, ipId, userAgentId, backupArray });
	return backups;
};

const twoFAStatus: TwoFAStatus = async (user) => {
	const two_fa_count = await userQueries.select_twoFABackupCount(user.registered_user_id);
	return {
		two_fa_active: user.two_fa_enabled ? true: false,
		two_fa_backup: user.two_fa_backup ? true : false,
		two_fa_count,
		two_fa_always_required: user.two_fa_always_required? true : false
	};
};

const userObject: UserObject = async (user) => {
	const twoFA = await twoFAStatus(user);
	const output: TVuexObject = {
		email: user.email,
		...twoFA
	};
	if (user.admin) output.admin = user.admin;
	return output;
};

const userCast = (req: TRequest): TPassportDeserializedUser => {
	const user = <TPassportDeserializedUser>req.user;
	return user;
};

export const changePassword_patch: RequestMethod = async (req, res) => {
	const user = userCast(req);

	const body = <TUserSchemaChangePassword>validate_input(req.body, schema_user.changePassword);

	await checkPasswordAndToken(req);

	const newPassword = body.newPassword;
	if (newPassword.trim().toLowerCase().includes(user.email)) throw customError(HttpCode.BAD_REQUEST, ErrorMessages.PASSWORD_EMAIL);
	const isPwned = await pwnedPassword(newPassword);
	if (isPwned >=1) throw customError(HttpCode.BAD_REQUEST, ErrorMessages.PWNED_PASSWORD);

	const userId = user.registered_user_id;
	const newHash = await rabbit_createHash({ password: newPassword });
	const { ipId, userAgentId } = await sharedQueries.select_ipIdUserAgentId_transaction(req);

	await userQueries.update_passwordHash({ userId, passwordHash: newHash });

	send({ res });
	send_email({ message_name: RabbitMessage.EMAIL_CHANGE_PASSWORD, data: { email: user.email, firstName: user.first_name, userId: user.registered_user_id, ipId, userAgentId } });
};

/**
  ** Sign out the user, remove session from user session set, POST ONLY
 */
export const signin_post: RequestMethod = async (req, res) => {
	const body = <TSchemaLogin>validate_input(req.body, schema_shared.login);
	const user = userCast(req);
	// Add this new user sessions into a user-speicific set, so can easily manipulate a single users sessions with smembers('sess:set:${reigstered_user_id})

	await redisQueries.session_add(user.registered_user_id, req.sessionID);

	// If remeber is true, set session length to 26 weeks
	// eslint-disable-next-line require-atomic-updates
	if (body.remember && req.session) req.session.cookie.maxAge = 1000 * 60 * 60 * 24 * 7 * 26;
	send({ res });
};

/**
  ** Sign out the user, remove session from user session set, POST ONLY
 */
export const signout_post: RequestMethod = async (req, res) => {
	await destroy_session(req, res);
	send({ res });
};

/**
  ** Disable 2fa setup - or - check 2fa code from setup is valid, and insert secret into postgres
 */
export const setupTwoFA_delete: RequestMethod = async (req, res) => {
	const user = userCast(req);
	await redisQueries.setup_delete(user.registered_user_id);
	return send({ res });
};

/**
  ** Set up 2fa for logged-in user
 */
export const setupTwoFA_get: RequestMethod = async (req, res) => {
	const user = userCast(req);
	// return if user already has 2fa set up or setup currently in progress (1 minute ttl)

	const tokenInRedis = await redisQueries.setupToken_get(user.registered_user_id);
	if (tokenInRedis) throw customError(HttpCode.CONFLICT, ErrorMessages.TWO_FA_PROGRESS);
	// Throw if 2fa already active
	if (user.two_fa_enabled) throw customError(HttpCode.CONFLICT, ErrorMessages.TWO_FA_ENABLED);
	// generate token
	const secret = authenticator.generateSecret();
	// put token response in redis for 90 seconds

	await redisQueries.setupToken_set(user.registered_user_id, secret);
	// return just otp secret to user
	send({ res, response: { secret } });
};

/**
 ** Toggle the always_enabled 2fa boolean
*/
export const setupTwoFA_patch: RequestMethod = async (req, res) => {
	const user = userCast(req);
	// Reject is 2fa not enabled
	if (!user.two_fa_enabled) throw customError(HttpCode.CONFLICT, ErrorMessages.TWO_FA_NOT_ENABLED);

	const body = <TUserSchemaTwoFAAlwaysRequired>validate_input(req.body, schema_user.twoFAAlwaysRequired);

	if (user.two_fa_always_required && body.alwaysRequired) throw customError(HttpCode.BAD_REQUEST, ErrorMessages.TWO_FA_ALWAYS);
	if (!body.alwaysRequired) await checkPasswordAndToken(req);
	await userQueries.update_twoFAAlwaysRequired(user.registered_user_id, body.alwaysRequired);
	send({ res });
};

/**
  ** Check 2fa secret is valid, then enable 2fa for user
 */
export const setupTwoFA_post: RequestMethod = async (req, res) => {
	const user = userCast(req);
	// Reject is 2fa already enabled
	if (user.two_fa_enabled) throw customError(HttpCode.CONFLICT, ErrorMessages.TWO_FA_ENABLED);

	// Get user
	const userId = user.registered_user_id;
	// Get 2fa secret from redis, reject if undefined
	const secret = await redisQueries.secret_get(user.registered_user_id);
	if (!secret) throw customError(HttpCode.BAD_REQUEST, ErrorMessages.TWO_FA_SETUP);

	// Validate body

	const body = <TUserSchemaTwoFASetup>validate_input(req.body, schema_user.twoFASetup);
		
	// Trim client provded token, validate, reject if invalid
	const token = body.token.replace(/\s/g, '').trim();
	const isValid = authenticator.check(token, secret);
	if (!isValid) throw customError(HttpCode.BAD_REQUEST, ErrorMessages.TWO_FA_CODE);

	// Delete 2fa setup secret from redis
	// await Redis.del(`${RedisKey.TwoFASetup}${userId}`);
	await redisQueries.secret_delete(user.registered_user_id);
	const { ipId, userAgentId } = await sharedQueries.select_ipIdUserAgentId_transaction(req);

	// Insert 2fa secret into postgres
	await userQueries.insert_twoFASecret({ userId, secret, ipId, userAgentId });

	// ! Remove all user sessions except current
	// Find set of all current users logins, and delete all session except for current one
	// const userSessionsSet = await Redis.smembers(`user_sessions:set:${req.user.registered_user_id}`);
	// for (const session of userSessionsSet) if (session !== `sess:${req.sessionID}`) await Redis.del(session);

	// Email user 2fa enabled message
	// email_twoFA({ email: user.email, firstName: user.first_name, ipId, userAgentId, userId, enabled: true });
	send_email({ message_name: RabbitMessage.EMAIL_TWO_FA, data: { email: user.email, firstName: user.first_name, ipId, userAgentId, userId, enabled: true } });
	send({ res, response: ResponseMessages.TWO_FA_ENABLED });
};

// Remove 2fa from user account
export const twoFA_delete: RequestMethod = async (req, res) => {
	const user = userCast(req);
	await checkPasswordAndToken(req);
	await sharedQueries.delete_twoFA_transaction(user.registered_user_id);
	const { ipId, userAgentId } = await sharedQueries.select_ipIdUserAgentId_transaction(req);
	send({ res });
	send_email({ message_name: RabbitMessage.EMAIL_TWO_FA, data: { email: user.email, firstName: user.first_name, ipId, userAgentId, userId: user.registered_user_id, enabled: false } });
};

/**
  ** Get current active + backup 2fa status of user
 */
export const twoFA_get: RequestMethod = async (req, res) => {
	const user = userCast(req);
	const response = await twoFAStatus(user);
	send({ res, response });
};

/**
 ** Create new backups - used when backups already in place, will delete current backups if in place
*/
export const twoFA_patch: RequestMethod = async (req, res) => {
	// !this should be checking yup?
	const user = userCast(req);
	if (!user.two_fa_enabled) throw customError(HttpCode.CONFLICT, ErrorMessages.TWO_FA_NOT_ENABLED);
	await checkPasswordAndToken(req);
	const { ipId, userAgentId } = await sharedQueries.select_ipIdUserAgentId_transaction(req);
	const backups = await createNewBackupCodes({ user, ipId, userAgentId });
	send({ res, response: { backups } });
	send_email({ message_name: RabbitMessage.EMAIL_TWO_FA_BACKUP, data: { email: user.email, firstName: user.first_name, ipId, userAgentId, userId: user.registered_user_id, enabled: true } });
};

/**
  ** Create new backups, will delete current backups if in place
 */
export const twoFA_post: RequestMethod = async (req, res) => {
	const user = userCast(req);
	if (!user.two_fa_enabled) throw customError(HttpCode.CONFLICT, ErrorMessages.TWO_FA_NOT_ENABLED);
	// Check to see if backup codes are already there, if so, throw error
	const backupCount = await userQueries.select_twoFABackupCount(user.registered_user_id);
	if (backupCount && backupCount > 0) throw customError(HttpCode.BAD_REQUEST, ErrorMessages.TWO_FA_BACKUP);
	const { ipId, userAgentId } = await sharedQueries.select_ipIdUserAgentId_transaction(req);
	const backups = await createNewBackupCodes({ user, ipId, userAgentId });
	send({ res, response: { backups } });
	send_email({ message_name: RabbitMessage.EMAIL_TWO_FA_BACKUP, data: { email: user.email, firstName: user.first_name, ipId, userAgentId, userId: user.registered_user_id, enabled: true } });
};

/**
 ** Delete two_fa backup codes
 */
export const twoFA_put: RequestMethod = async (req, res) => {
	const user = userCast(req);
	await checkPasswordAndToken(req);
	const { ipId, userAgentId } = await sharedQueries.select_ipIdUserAgentId_transaction(req);
	await userQueries.delete_twoFABackupCodeAll(user.registered_user_id);
	send({ res });
	send_email({ message_name: RabbitMessage.EMAIL_TWO_FA_BACKUP, data: { email: user.email, firstName: user.first_name, ipId, userAgentId, userId: user.registered_user_id, enabled: false } });
};

/**
  ** simple check user auth status router, return user object, all logic is done in the routing function
 */
export const user_get: RequestMethod = async (req, res) => {
	const user = userCast(req);
	// Loop over a user session set, remove keys if session no longer valid
	const userSet = await redisQueries.userSet_get(user.registered_user_id);

	const sessionPromiseArray = [];
	for (const sessionKey of userSet) sessionPromiseArray.push(redisQueries.session_get(sessionKey));
	const sessions = await Promise.all(sessionPromiseArray);
	const removeSessionPromiseArray = [];
	
	for (const [ index, item ] of userSet.entries()) if (!sessions[index]) removeSessionPromiseArray.push(redisQueries.sessionSet_remove(user.registered_user_id, item));
	await Promise.all(removeSessionPromiseArray);
	
	const vuexObject = await userObject(user);
	send({ res, response: vuexObject });
};