import { authenticator } from 'otplib';
import { cleanEmail } from '../lib/helpers';
import { customError } from '../config/customError';
import { sharedQueries } from '../components/shared/shared_queries';
import { RabbitMessage } from '../types/enum_rabbitMessage';
import { ErrorMessages } from '../types/enum_error';
import { FailedLogin, UserId, sessionId } from '../types';
import { HttpCode } from '../types/enum_httpCode';
import { passportQueries } from '../components/passport/passport_queries';
import { rabbit_validateHash } from '../lib/rabbitRpc';
import { schema_shared, TSchemaLogin } from '../components/shared/shared_schema';
import { send_email } from '../lib/rabbitSend';
import { validate_input } from '../lib/validateinput';
import passport from 'passport';
import passportLocal, { IStrategyOptionsWithRequest } from 'passport-local';

const LocalStrategy = passportLocal.Strategy;

authenticator.options = {
	window: 1
};

const failedLogin: FailedLogin = async ({ ipId, userId, userAgentId, errorString }) => {
	await Promise.all([
		passportQueries.update_loginAttempt(userId),
		passportQueries.insert_loginHistory({ ipId, userId, userAgentId }),
	]);
	throw Error(errorString);
};

passport.use(new LocalStrategy(<IStrategyOptionsWithRequest>
	{
		usernameField: 'email',
		passwordField: 'password',
		passReqToCallback: true,
	},
async (req, email, password, done) => {
	try {
		const userEmail = cleanEmail(email);
		const [ { ipId, userAgentId }, user ] = await Promise.all([
			sharedQueries.select_ipIdUserAgentId_transaction(req),
			passportQueries.select_userPassportLogin(userEmail)
		]);
		// Reqbody as
		const body = <TSchemaLogin>validate_input(req.body, schema_shared.login);

		// Return error if no user, show generic error to user though
		if (!user) throw Error(ErrorMessages.USER_NOT_FOUND);
			
		const userId = user.registered_user_id;

		if (Number(user.login_attempt_number) + 1 === 5) send_email({ message_name: RabbitMessage.EMAIL_LOGIN_ATTEMPT, data: { email: userEmail, firstName: user.first_name, ipId, userAgentId, userId } });
		// If attempts 20 or more completely block signin - can only be reset by admin
		if (Number(user.login_attempt_number) + 1 >= 20) {
			// Maybe here should kill or current sessions?
			await failedLogin({ ipId, userId, userAgentId, errorString: ErrorMessages.PASSPORT_BLOCKED });
		}

		// validate password
		const validPassword = await rabbit_validateHash({ known_password_hash: user.password_hash, attempt: password });
		// If password not valid, increase attempts by one, return error
		if (!validPassword) await failedLogin({ ipId, userId, userAgentId, errorString: ErrorMessages.PASSWORD_EMAIL_INVALID });

		// Two factor authentication checking
		if (user.two_fa_enabled) {
			const token = body.token ? body.token.replace(/\s/g, '').trim(): undefined;
			const backup = body.twoFABackup;
			if (!token && user.two_fa_backup) throw Error(ErrorMessages.PASSPORT_TOKEN_BACKUP);
			if (!token) throw Error(ErrorMessages.PASSPORT_TOKEN);
			// If backup boolean supplied with credentials, test token against postgres backups,
			if (backup) {
				// Return error if no backup codes found in db
				if (!user.two_fa_backup) await failedLogin({ ipId, userId, userAgentId, errorString: ErrorMessages.PASSPORT_INVALID_BACKUP });
				const backupCodes = await sharedQueries.select_twoFABackup(user.registered_user_id);
				let verifiedBackupToken = undefined;
				for (const i of backupCodes) {
					// eslint-disable-next-line no-await-in-loop
					const match = await rabbit_validateHash({ known_password_hash: i.two_fa_backup_code, attempt: token });
					if (match) {
						verifiedBackupToken = i;
						break;
					}
				}
				if (!verifiedBackupToken) await failedLogin({ ipId, userId, userAgentId, errorString: ErrorMessages.PASSPORT_INVALID_BACKUP });
				else await sharedQueries.delete_twoFABackupSingle(verifiedBackupToken.two_fa_backup_id);

			} else {
				// If backup flag not set, check token against otpLib generated token
				if (!authenticator.check(token, user.two_fa_enabled)) await failedLogin({ ipId, userId, userAgentId, errorString: ErrorMessages.PASSPORT_INVALID_TOKEN });
			}
		}
			
		// Reset attempts & insert success login
		await Promise.all([
			passportQueries.update_attemptReset(userId),
			passportQueries.insert_loginHistory({ ipId, userId, userAgentId, success: true, sessionId: req.sessionID as sessionId }),
		]);
		// Return logged in user to passport/session/express
		done(null, user);
	} catch (e) {
		// console.log(e);
		const message = e instanceof Error && Object.values(ErrorMessages).includes(<ErrorMessages>e.message) ? <ErrorMessages>e.message : ErrorMessages.INTERNAL;
		const error = customError(HttpCode.UNAUTHORIZED, message);
		done(error, null);
	}
})
);

passport.serializeUser((user, done) => {
	done(null, user.registered_user_id);
});

passport.deserializeUser(async (userId: UserId, done) => {
	try {
		const user = await passportQueries.select_userPassportDeserialize(userId);
		if (!user) throw customError(HttpCode.BAD_REQUEST, ErrorMessages.USER_NOT_FOUND);
		done(null, user);
	} catch (e) {
		done(null, undefined);
	}
});

export { passport };