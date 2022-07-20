import { authenticator } from 'otplib';
import { COOKIE_NAME } from '../config/env';
import { customError, customTypeError } from '../config/customError';
import { sharedQueries } from '../components/shared/shared_queries';
import { ErrorMessages } from '../types/enum_error';
import { HttpCode } from '../types/enum_httpCode';
import { TPassportDeserializedUser, CheckPasswordAndToken, DestroySession, backupId } from '../types';
import { RequestHandler } from 'express';
import { redisQueries } from '../lib/redisQueries';
import { rabbit_validateHash } from './rabbitRpc';

authenticator.options = {
	window: 1
};

// Check the user password is valid, only for an already authed user that has a req.user object, and req.body
export const checkPasswordAndToken: CheckPasswordAndToken = async (req) => {
	const eitherInvalid = customError(HttpCode.UNAUTHORIZED, ErrorMessages.PASSWORD_EMAIL_INVALID_TOKEN);
	if (!req?.body?.password) throw eitherInvalid;
	const user = req.user as TPassportDeserializedUser;
	if (!user.password_hash) throw customTypeError(ErrorMessages.UNKNOWN_SESSION);

	let verifiedBackupId: backupId | undefined = undefined;
	const validPassword = await rabbit_validateHash({ known_password_hash: user.password_hash, attempt: req.body.password });
	if (!validPassword) throw eitherInvalid;

	// Check token, or backup token, if enabled on req.user
	// If (validPassword) redudnant here
	if (validPassword && user.two_fa_enabled && user.two_fa_always_required) {
		// 401 correct http code?
		if (!req.body.token) throw eitherInvalid;
		if (req.body.twoFABackup) {
			const backupCodes = await sharedQueries.select_twoFABackup(user.registered_user_id);
			for (const i of backupCodes) {
				// eslint-disable-next-line no-await-in-loop
				const match = await rabbit_validateHash({ known_password_hash: i.two_fa_backup_code, attempt: req.body.token });
				if (match) {
					verifiedBackupId = i.two_fa_backup_id;
					break;
				}
			}
			if (!verifiedBackupId) throw eitherInvalid;
		} else {
			const tokenValid = authenticator.check(req.body.token, user.two_fa_enabled);
			if (!tokenValid) throw eitherInvalid;
		}
	}
	if (verifiedBackupId && validPassword) await sharedQueries.delete_twoFABackupSingle(verifiedBackupId);
	
};

export const destroy_session: DestroySession = async (req, res) => {
	if (req.user) await redisQueries.session_remove(req.user.registered_user_id, req.sessionID);
	if (req.session) req.session.destroy((e) => {
		if (e) throw e;
	});
	req.logout();
	res.clearCookie(String(COOKIE_NAME));
};

const denied = (): never => {
	throw customError(HttpCode.FORBIDDEN);
};

export const isAdmin: RequestHandler = (req, _res, next) : void => req.isAuthenticated() && req.user?.admin ? next() : denied();
export const isAuthenticated: RequestHandler = (req, _res, next): void => req.isAuthenticated() ? next() : denied();
export const isNotAuthenticated: RequestHandler = (req, _res, next): void => !req.isAuthenticated() ? next(): denied();