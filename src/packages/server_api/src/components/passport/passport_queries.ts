import { customTypeError } from '../../config/customError';
import { Pool } from 'pg';
import { postgresql } from '../../config/db_postgres';
import * as types from 'types';
import format from 'pg-format';

class Passport_Queries {
	#db!: Pool;

	constructor (db:Pool) {
		this.#db = db;
	}

	async delete_twoFABackupCode (backupId: types.backupId): Promise<void> {
		const query = format('DELETE FROM two_fa_backup WHERE two_fa_backup_id = %1$L', backupId);
		await this.#db.query(query);
	}
	
	async insert_loginHistory ({ ipId, userId, userAgentId, success=false, sessionId = undefined }: types.TLoginHistory): Promise<void> {
		if (!ipId ||!userId ||! userAgentId) throw customTypeError('insert_loginHistory: !ipId ||!userId ||! userAgentId');
		const query = format(
			// eslint-disable-next-line indent
	`INSERT INTO
		login_history(ip_id, success, session_name, user_agent_id, registered_user_id)
		VALUES(%1$L, %2$L, %3$L, %4$L, %5$L)
		RETURNING login_history_id`,
			ipId, success, sessionId, userAgentId, userId);
		await this.#db.query(query);
	}
	
	async select_userPassportDeserialize (userId: types.UserId): Promise<types.TPassportDeserializedUser> {
		if (!userId) throw customTypeError('select_userPassportDeserialize: !userId');
		const query = format(
			// eslint-disable-next-line indent
	`SELECT
		ru.registered_user_id, ru.active, ru.email, ru.password_hash, ru.first_name,
		au.admin,
		tfs.two_fa_secret AS two_fa_enabled, tfs.always_required AS two_fa_always_required,
		CASE WHEN (SELECT COUNT(*) FROM two_fa_backup WHERE registered_user_id = ru.registered_user_id) > 0 THEN true ELSE false END AS two_fa_backup
	FROM registered_user ru
	LEFT JOIN two_fa_secret tfs
		ON ru.registered_user_id = tfs.registered_user_id
	LEFT JOIN admin_user au
		ON ru.registered_user_id = au.registered_user_id
	WHERE ru.registered_user_id = %1$L AND active = true;`,
			userId);
		const { rows } = await this.#db.query(query);
		return rows[0];
	}
	
	async select_userPassportLogin (email: string): Promise<types.TUserLogin|undefined> {
		const query = format(
			// eslint-disable-next-line indent
	`SELECT
		ru.registered_user_id, ru.active, ru.email, ru.password_hash, ru.first_name,
		la.login_attempt_number,
		tfs.two_fa_secret AS two_fa_enabled, tfs.always_required,
		au.admin,
		(
			SELECT NULLIF(COUNT(*),0)
			FROM two_fa_backup
			WHERE registered_user_id = ru.registered_user_id
		)
		AS two_fa_backup
	FROM registered_user ru
	LEFT JOIN two_fa_secret tfs
		ON ru.registered_user_id = tfs.registered_user_id
	LEFT JOIN login_attempt la
		ON ru.registered_user_id = la.registered_user_id
	LEFT JOIN admin_user au
		ON ru.registered_user_id = au.registered_user_id
	WHERE ru.email = %1$L AND active = true`,
			email);
		const { rows } = await this.#db.query(query);
		return rows[0];
	}
	
	async update_attemptReset (userId: types.UserId): Promise<void> {
		if (!userId) throw customTypeError('update_attemptReset: !userId');
	
		const query = format('UPDATE login_attempt SET login_attempt_number = 0 WHERE registered_user_id = %1$L',
			userId);
		await this.#db.query(query);
	}
	
	async update_loginAttempt (userId: types.UserId): Promise<void> {
		if (!userId) throw customTypeError('update_loginAttempt: !userId');
		const query = format(
			// eslint-disable-next-line indent
	`INSERT	INTO
		login_attempt (login_attempt_number, registered_user_id)
	VALUES
		(%1$L, %2$L)
	ON CONFLICT
		(registered_user_id)
	DO UPDATE
		SET login_attempt_number = login_attempt.login_attempt_number +1`,
			1, userId);
		await this.#db.query(query);
	}
	
}

export const passportQueries = new Passport_Queries(postgresql);
