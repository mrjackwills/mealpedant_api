import { api_version } from '../../config/api_version';
import { cleanEmail, randomHex } from '../../lib/helpers';
import { rabbit_createHash } from '../../lib/rabbitRpc';
import { customError, customTypeError } from '../../config/customError';
import { RabbitMessage } from '../../types/enum_rabbitMessage';
import { ErrorMessages } from '../../types/enum_error';
import { HttpCode } from '../../types/enum_httpCode';
import { incognitoQueries } from './incognito_queries';
import { pwnedPassword } from 'hibp';
import { send_email } from '../../lib/rabbitSend';
import { ResponseMessages } from '../../types/enum_response';
import { schema_incogntio, TSchemaIncognitoEmail, TSchemaIncognitoNewUser, TSchemaIncognitoPassword, TSchemaIncognitoResetString, TSchemaIncognitoVerifyString } from './incognito_schema';
import { schema_shared } from '../shared/shared_schema';
import { sharedQueries } from '../shared/shared_queries';
import { send } from '../../lib/send';
import { TNewUser, RequestMethod } from '../../types';
import { validate_input } from '../../lib/validateinput';
import { redisQueries } from '../../lib/redisQueries';
import { authenticator } from 'otplib';

export const signin_body_validator: RequestMethod = async (req, _res, next) => {
	validate_input(req.body, schema_shared.login);
	next();
};

// If condtions met, insert random string in password_reset table, and email user link to this so can reset password
export const forgot_post: RequestMethod = async (req, res) : Promise<void> => {
	const body = <TSchemaIncognitoEmail> validate_input(req.body, schema_incogntio.email);
	const email = cleanEmail(body.email);

	const [ validUser, resetProcess ] = await Promise.all([
		sharedQueries.select_userActiveIdByEmail(email),
		sharedQueries.select_passwordResetProcess(email)
	]);

	const userId = validUser?.registered_user_id;
	const firstName = validUser?.first_name;

	// Check to see if user exists, if not return generic success result
	if (userId && firstName && !resetProcess) {

		// Create random 256bit string, and insert into password_reset, with registered_user_id (from emailSearch)
		const [ resetString, { ipId, userAgentId } ] = await Promise.all([
			randomHex(256),
			sharedQueries.select_ipIdUserAgentId_transaction(req),
		]);
			// Insert password reset, with ipId and random string, into db
		await incognitoQueries.insert_passwordReset({ userId, resetString, ipId, userAgentId });

		// Email user with reset link/code
		// email_resetPassword({ email, resetString, ipId, userAgentId, firstName, userId });
		send_email({ message_name: RabbitMessage.EMAIL_RESET, data: { email, resetString, ipId, userAgentId, firstName, userId } });
	}
	// If user not in db or password reset already valid, delay response
	// else await delayResponse({ start });

	// Send response at the same time no matter the outcome - so clients can't guess if extra work is being done server side
	send({ res, response: ResponseMessages.INSTRUCTION_SENT });
};

export const online_get: RequestMethod = async (_req, res): Promise<void> => {
	send({ res, response: { api_version } });
};

// Register new user, store all details in redis so that postgres isn't touched - bar ip - until user verifies
export const register_post: RequestMethod = async (req, res) => {
	// Make sure the data received from client matches specification

	const body = <TSchemaIncognitoNewUser> validate_input(req.body, schema_incogntio.newUser);

	// lowercase and trim the supplied email address
	const email = cleanEmail(body.email);
		
	// Check if email supplied is in the list of banned domains, throw error if true
	const banned_email = await sharedQueries.select_bannedDomain(email);
	if (banned_email) throw customError(HttpCode.BAD_REQUEST, ErrorMessages.BANNED_EMAIL);
	
	const pwned = await pwnedPassword(body.password);
	if (pwned >=1) throw customError(HttpCode.BAD_REQUEST, ErrorMessages.PWNED_PASSWORD);
		
	// Search in the redis awaiting verification, and postgres user table, for users with the same email as what has been provided
	const [ redisEmailSearch, user ] = await Promise.all([
		redisQueries.verifyEmail_exists(email),
		sharedQueries.select_userActiveIdByEmail(email),
	]);

	// Return error is email found
	const userId = user?.registered_user_id;
	if (!redisEmailSearch && !userId) {

		// Create 256 length random verification string

		const [ verifyString, password_hash, { ipId, userAgentId } ] = await Promise.all([
			randomHex(256),
			rabbit_createHash({ password: body.password }),
			sharedQueries.select_ipIdUserAgentId_transaction(req)
		]);

		// Create user object - with hashed password - to store in redis until email verified
		const newUser: TNewUser = {
			email,
			first_name: body.firstName,
			last_name: body.lastName,
			password_hash,
			ipId,
			userAgentId,
		};

		await redisQueries.verifyUser_set(newUser, verifyString);
		await send_email({ message_name: RabbitMessage.EMAIL_VERIFY, data: { email, firstName: body.firstName, verifyString, ipId, userAgentId } });
	}
	// If user already in db, delay response
	// else await delayResponse({ start });
	send({ res, response: ResponseMessages.INSTRUCTION_SENT });
};

export const resetPassword_get: RequestMethod = async (req, res) => {
	// TODO need to add a 2 flag, so that it is access once here, and then secondly in the patch method below, and then it no longer works
	// so this route would do some_flag - 1, and the route beneath does some_floag -2, then if flag is 0 it is no longer useable

	const params = <TSchemaIncognitoResetString> validate_input(req.params, schema_incogntio.resetString);

	// const resetString = req.params.resetString as string;
	const passwordReset = await incognitoQueries.select_passwordReset_get(params.resetString);

	if (!passwordReset) throw customError(HttpCode.BAD_REQUEST, ErrorMessages.VERIFICATION_INCORRECT);
	send({ res, response: { two_fa_backup: passwordReset.two_fa_backup, two_fa_active: passwordReset.two_fa_active } });
};

export const resetPassword_patch: RequestMethod = async (req, res) => {

	// verify address, and password, are valid
	const params = <TSchemaIncognitoResetString> validate_input(req.params, schema_incogntio.resetString);
	const body = <TSchemaIncognitoPassword> validate_input(req.body, schema_incogntio.password);
	
	// Check string links to a valid password_reset
	const passwordReset = await incognitoQueries.select_passwordReset_patch(params.resetString);

	if (!passwordReset) throw customError(HttpCode.BAD_REQUEST, ErrorMessages.PASSWORD_RESET_INCORRECT);

	if (body.newPassword.toLowerCase().includes(passwordReset.email.toLowerCase())) throw customError(HttpCode.BAD_REQUEST, ErrorMessages.PASSWORD_EMAIL);
	// Check new password against pwnedPasswords, also done client side
	const pwned = await pwnedPassword(body.newPassword);
	if (pwned >=1) throw customError(HttpCode.BAD_REQUEST, ErrorMessages.PWNED_PASSWORD);

	// Check token there

	// const user = await passportQueries.select_userPassportLogin(passwordReset.email);
	if (passwordReset?.two_fa_secret) {
		if (!body.token) throw customError(HttpCode.BAD_REQUEST, ErrorMessages.TOKEN_INVALID);
		const tokenValid = authenticator.check(body.token, passwordReset.two_fa_secret);
		if (!tokenValid) throw customError(HttpCode.BAD_REQUEST, ErrorMessages.TOKEN_INVALID);
	}

	const { ipId, userAgentId } = await sharedQueries.select_ipIdUserAgentId_transaction(req);

	const password_hash = await rabbit_createHash({ password: body.newPassword });
	await incognitoQueries.update_passwordReset_transaction({ stringId: passwordReset.password_reset_id, password_hash, userId: passwordReset.registered_user_id });
	send({ res, response: ResponseMessages.PASSWORD_RESET });
	send_email({ message_name: RabbitMessage.EMAIL_CHANGE_PASSWORD, data: { email: passwordReset.email, firstName: passwordReset.first_name, userId: passwordReset.registered_user_id, ipId, userAgentId } });
};

// User visits specific link, if valid, insert new user into postgres
export const verify_get: RequestMethod = async (req, res) => {

	const params = <TSchemaIncognitoVerifyString> validate_input(req.params, schema_incogntio.verifyString);

	const user = await redisQueries.verifyUser_get(params.verifyString);

	if (!user) throw customTypeError(ErrorMessages.VERIFICATION_INCORRECT);

	await redisQueries.verifyUser_delete(user, params.verifyString);
	await incognitoQueries.insert_newUser(user);
	send({ res, response: ResponseMessages.VERIFIED });
};
