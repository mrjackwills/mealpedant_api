
import { customTypeError } from '../../config/customError';
import { extractIp, extractUserAgent } from '../../lib/helpers';
import { LogEntry } from 'winston';
import { Pool } from 'pg';
import { postgresql } from '../../config/db_postgres';
import { Request } from 'express';
import * as types from '../../types';
import format from 'pg-format';

class Shared_Queries {
	#db!: Pool;

	constructor (db:Pool) {
		this.#db = db;
	}

	async delete_twoFABackupSingle (backupId: types.backupId): Promise<void> {
		if (!backupId) throw customTypeError('delete_twoFABackupSingle: !backupId');
		const query = format('DELETE FROM two_fa_backup WHERE two_fa_backup_id = %1$L', backupId);
		await this.#db.query(query);
	}

	async delete_twoFA_transaction (userId: types.UserId): Promise<void> {
		if (!userId) throw customTypeError('delete_twoFA_transaction: !userId');
		const Client = await this.#db.connect();
		try {
			await Client.query('BEGIN');
			const twoFA_query = format('DELETE FROM two_fa_secret WHERE registered_user_id = %1$L', userId);
			const twoFABackup_query = format('DELETE FROM two_fa_backup WHERE registered_user_id = %1$L', userId);
			await Client.query(twoFA_query);
			await Client.query(twoFABackup_query);
			await Client.query('COMMIT');
		} catch (e) {
			await Client.query('ROLLBACK');
			throw e;
		} finally {
			Client.release();
		}
	}

	async insert_error ({ message, level, timestamp, stack, uuid, httpCode }: LogEntry): Promise<void> {
		if (!message || !level || !timestamp) throw customTypeError('insert_error: !message || !level || !timestamp || !stack');
		const query = format(
		// eslint-disable-next-line indent
`INSERT INTO
	error_log(timestamp, level, message, stack, uuid, http_code)
VALUES
	(%1$L, %2$L, %3$L, %4$L, %5$L, %6$L)`,
			timestamp, level, message, stack, uuid, httpCode);
		await this.#db.query(query);
	}

	async insert_passwordReset ({ userId, resetString, ipId, userAgentId }: types.TInsertPasswordReset): Promise<void> {
		if (!userId || !resetString || !ipId || !userAgentId) throw customTypeError('insertPasswordReset(): !registered_user_id || !resetString || !ipId || !userAgentId');
		const query = format(`INSERT INTO password_reset (registered_user_id, reset_string, ip_id, user_agent_id) VALUES(%1$L, %2$L, %3$L, %4$L)`,
			userId, resetString, ipId, userAgentId);
		await this.#db.query(query);
	}

	async select_ipIdUserAgentId_transaction (req: Request): Promise<types.TIpUserAgent> {
		if (!req) throw customTypeError('select_ipIdUserAgentId_transaction(): !req');
		const Client = await this.#db.connect();
		try {
			await Client.query('BEGIN');
			const ip = extractIp(req);
			const userAgent = extractUserAgent(req);

			if (!ip) throw customTypeError('select_ipIdUserAgentId_transaction(): !ip');
			const select_ipId = format(`SELECT ip_id FROM ip_address WHERE ip = %1$L`, ip);
			const ipId = await Client.query(select_ipId);

			const select_userAgentId = format('SELECT user_agent_id FROM user_agent WHERE user_agent_string = %1$L', userAgent);
			const userAgentId = await Client.query(select_userAgentId);

			let output_id: types.ipId;
			let output_userAgent: types.userAgentId;

			if (ipId.rows[0]?.ip_id) output_id = ipId.rows[0].ip_id;
			else {
				const insert_ip = format('INSERT INTO ip_address (ip) VALUES(%L) RETURNING ip_id', ip);
				const { rows } = await Client.query(insert_ip);
				output_id = rows[0].ip_id;
			}
		
			if (userAgentId.rows[0]?.user_agent_id) output_userAgent = userAgentId.rows[0].user_agent_id;
			else {
				const insert_userAgent = format('INSERT INTO user_agent(user_agent_string) VALUES(%1$L) RETURNING user_agent_id', userAgent);
				const { rows } = await Client.query(insert_userAgent);
				output_userAgent = rows[0].user_agent_id;
			}
			await Client.query('COMMIT');

			const output: types.TIpUserAgent = {
				ipId: output_id,
				userAgentId: output_userAgent,
			};
			return output;
		} catch (e) {
			await Client.query('ROLLBACK');
			throw e;
		} finally {
			Client.release();
		}
	}

	async select_bannedDomain (email: string): Promise<boolean> {
		if (!email) throw customTypeError('select_bannedDomain: !email');
		const domain = email.split('@')[1];
		const query = format (`SELECT * from banned_email_domain WHERE domain = %1$L`,
			domain);
		const { rows } = await this.#db.query(query);
		return rows[0] ? true : false;
	}

	async select_passwordResetProcess (email: string) : Promise<types.TAdminPasswordReset> {
		if (!email) throw customTypeError('select_passwordResetProcess: !email');
		const query = format(
		// eslint-disable-next-line indent
`SELECT
	ru.registered_user_id, ru.email,
	pr.timestamp, pr.password_reset_id
FROM password_reset pr
JOIN registered_user ru
	ON pr.registered_user_id = ru.registered_user_id
WHERE
	ru.email = %1$L
	AND pr.timestamp >= NOW () - INTERVAL %2$L
	AND pr.consumed IS NOT TRUE`,
			email, '1 hour');
		const { rows } = await this.#db.query(query);
		return rows[0];
	}

	async select_twoFABackup (userId: types.UserId): Promise<Array<types.TTwoFaBackup>> {
		if (!userId) throw customTypeError('select_twoFABackup: !userId');
		const query = format('SELECT two_fa_backup_code, two_fa_backup_id FROM two_fa_backup WHERE registered_user_id = %1$L',
			userId);
		const { rows } = await this.#db.query(query);
		return rows;
	}

	async select_userActiveIdByEmail (email: string): Promise<types.TNameUserId|undefined> {
		if (!email) throw customTypeError('select_userActiveIdByEmail: !email');
		const query = format('SELECT registered_user_id, first_name FROM registered_user WHERE active = true AND email = %1$L', email);
		const { rows } = await this.#db.query(query);
		return rows[0];
	}

}

export const sharedQueries = new Shared_Queries(postgresql);