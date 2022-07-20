import { customError, customTypeError } from '../../config/customError';
import { ErrorMessages } from '../../types/enum_error';
import { HttpCode } from '../../types/enum_httpCode';
import { limiter } from '../../config/rateLimiter';
import { Pool } from 'pg';
import { postgresql } from '../../config/db_postgres';
import { RedisKey } from '../../types/enum_redis';
import * as types from '../../types';
import format from 'pg-format';
import { parse } from 'secure-json-parse';
import { redisQueries } from '../../lib/redisQueries';

class Admin_Queries {

	#db!: Pool;

	constructor (db:Pool) {
		this.#db = db;
	}
	
	async delete_userSession ({ sessionName, currentSession }: types.TDeleteSession): Promise<void> {
		if (!sessionName || !currentSession) throw customTypeError('delete_userSession(): !sessionName || !currentSession');
		if (`${RedisKey.SESSION}${currentSession}` === sessionName) throw customError(HttpCode.BAD_REQUEST, ErrorMessages.SELF);
		const session = await redisQueries.admin_get_session(sessionName);
		if (!session) throw customError(HttpCode.BAD_REQUEST, ErrorMessages.UNKNOWN_SESSION);
		const parsedSession = parse(session);
		if (!parsedSession.passport || !parsedSession.passport.user) throw customTypeError('delete_userSession(): !parsedSession.passport.user');
		await redisQueries.admin_remove_session(sessionName, parsedSession.passport.user);
	}

	async select_activeEmails (): Promise<Array<string>> {
		// TODO this will need to be a join when email table is implemented
		const query = format('SELECT email FROM registered_user WHERE active = true');
		const { rows } = await this.#db.query(query);
		const parsedResult = [];
		for (const result of rows) parsedResult.push(result.email);
		return parsedResult;
	}

	async select_allUser (): Promise<Array<types.TAllUsers>> {
		const query = format (
			// eslint-disable-next-line indent
	`SELECT
		ru.first_name as "firstName", ru.last_name AS "lastName", ru.email, ru.active, ru.timestamp,
		ip.ip AS user_creation_ip,
		la.login_attempt_number,
		pr.password_reset_id, pr.reset_string, pr.timestamp as "passwordResetDate", pr.password_creation_ip as "passwordResetCreationIp", pr.consumed as "passwordResetConsumed",
		lh.login_ip, lh.success as "loginSuccess", lh.timestamp AS login_date, lh.user_agent_string,
		CASE WHEN au.admin IS null THEN false ELSE CASE WHEN au.admin IS true THEN true ELSE false END END AS admin,
		CASE WHEN tfa.two_fa_secret IS NOT null THEN true ELSE false END as "tfaSecret"
	FROM registered_user ru
	LEFT JOIN ip_address ip
		ON ru.ip_id = ip.ip_id
	LEFT JOIN login_attempt la
		ON ru.registered_user_id = la.registered_user_id
	LEFT JOIN admin_user au
		ON ru.registered_user_id = au.registered_user_id
	LEFT JOIN two_fa_secret tfa
		ON ru.registered_user_id = tfa.registered_user_id
	LEFT JOIN
		(
			SELECT
				pr.registered_user_id, pr.password_reset_id, pr.timestamp, pr.reset_string, pr.consumed,
				ip.ip AS password_creation_ip
			FROM password_reset pr
			JOIN ip_address ip
				ON pr.ip_id = ip.ip_id
			WHERE
				NOW () <= pr.timestamp + INTERVAL '1 hour'
				AND pr.consumed = false
		) pr
		ON ru.registered_user_id = pr.registered_user_id
	LEFT JOIN LATERAL
		(
			SELECT
				lh.registered_user_id, lh.timestamp, lh.login_history_id, lh.success,
				ua.user_agent_string,
				ip.ip AS login_ip
		FROM login_history lh
		JOIN ip_address ip
			ON lh.ip_id = ip.ip_id
		JOIN user_agent ua
			ON lh.user_agent_id = ua.user_agent_id
		WHERE
			lh.registered_user_id = ru.registered_user_id
		ORDER BY timestamp DESC limit 1
		) lh
		ON ru.registered_user_id = lh.registered_user_id
	`);
		const { rows } = await this.#db.query(query);
		return rows;
	}

	async select_allUserLimits (): Promise<Array<types.TLimitedClient>> {
		const limtedClients = await redisQueries.admin_get_limited_clients();
		const data: Array<types.TLimitedClient> = [];
		for (const [ index, key ] of Object.entries(limtedClients)) {
			// eslint-disable-next-line no-await-in-loop
			const exists = await redisQueries.get(key);
			if (!exists) {
				// eslint-disable-next-line no-await-in-loop
				await redisQueries.admin_remove_from_limiter_set(key);
				limtedClients.splice(Number(index), 1);
				continue;
			}
			const clientKey = key.split(':')[1];
			if (!clientKey) continue;
			// eslint-disable-next-line no-await-in-loop
			const client = await limiter.get(clientKey);
			if (!client) continue;
			const clientLimitData: types.TLimitedClient = {
				p: client.consumedPoints,
				u: clientKey as string,
				b: false,
			};
			if (client.msBeforeNext >= 60 * 1000 || client.consumedPoints >= 60) {
				clientLimitData.b = true;
				clientLimitData.m = client.msBeforeNext;
			}
			data.push(clientLimitData);
		}
		if (data.length > 1) data.sort((clientA, clientB) => clientA.b < clientB.b ? -1 : clientA.b > clientB.b ? 1 : 0);
		return data;
	}
	
	async select_error (): Promise<Array<types.TErrorLog>> {
		const query = `SELECT * FROM error_log`;
		const { rows } = await this.#db.query(query);
		return rows;
	}
	
	async select_userIdByEmail (email: string): Promise<types.TNameUserId | undefined> {
		const query = format('SELECT registered_user_id, first_name FROM registered_user WHERE email = %1$L', email);
		const { rows } = await this.#db.query(query);
		return rows[0];
	}
	
	async select_userSession ({ userId, currentSession }: types.TSelectSession): Promise<Array<types.TAdminSession>> {
		if (!userId || !currentSession) throw customTypeError('select_userSession(): !userId || !currentSession');
		const userSetName = `${RedisKey.SESSION_SET}${userId}`;
	
		const userSet = await redisQueries.admin_get_setmembers(userSetName);
		const output = [];
	
		// Loop over session array, del redis key if invalid, and remove from array if invalid, else insert as full session data
		for (const [ index, sessionKey ] of Object.entries(userSet)) {
			// eslint-disable-next-line no-await-in-loop
			const key = await redisQueries.get(sessionKey);
			if (key) {
				const session = parse(key);
				if (!session) continue;
				const cookie = session.cookie;
				if (`${RedisKey.SESSION}${currentSession}` === sessionKey) cookie.currentSession = true;
				cookie.sessionKey = sessionKey;
				const query = format(
					// eslint-disable-next-line indent
	`SELECT
		ua.user_agent_string,
		ip.ip
	FROM login_history lh
	JOIN user_agent ua
		ON lh.user_agent_id = ua.user_agent_id
	JOIN ip_address ip
		ON lh.ip_id = ip.ip_id
	WHERE lh.session_name = %1$L`,
					sessionKey.split(':')[1]);
				// eslint-disable-next-line no-await-in-loop
				const { rows } = await this.#db.query(query);
				cookie.userAgent = rows[0].user_agent_string;
				cookie.ip = rows[0].ip;
				output[Number(index)] = cookie;
			}
			else {
				// eslint-disable-next-line no-await-in-loop
				await redisQueries.admin_remove_from_session_set(userSetName, sessionKey);
				output.splice(Number(index), 1);
				continue;
			}
		}
		return output;
	}
	
	async update_userActive (userId: types.UserId, activeStatus: boolean): Promise<void> {
		if (!userId || typeof activeStatus !== 'boolean') throw customTypeError('update_userActive(): !userId || !activeStatus');
		const query = format(`UPDATE registered_user SET active = %1$L WHERE registered_user_id = %2$L`,
			activeStatus, userId);
		await this.#db.query(query);
		await redisQueries.admin_remove_active_user_sessions(userId);
	}
	
	async update_userAttempt (userId: types.UserId, attempt = 0): Promise<void> {
		if (!userId) throw customTypeError('update_userAttempt(): !userId');
		const query = format(`UPDATE login_attempt SET login_attempt_number = %1$L WHERE registered_user_id = %2$L`,
			attempt, userId);
		await this.#db.query(query);
	}
	
	async update_userPasswordConsumed (passwordResetId: string): Promise<void> {
		if (!passwordResetId) throw customTypeError('update_userPasswordConsumed(): !email || !passwordResetId');
		const query = format(`UPDATE password_reset SET consumed = true WHERE password_reset_id = %1$L`, passwordResetId) ;
		await this.#db.query(query);
	}

}

export const adminQueries = new Admin_Queries(postgresql);
