import { customTypeError } from '../../config/customError';
import { Pool } from 'pg';
import { postgresql } from '../../config/db_postgres';
import * as types from '../../types';
import format from 'pg-format';

class Incognito_Queries {
	#db!: Pool;

	constructor (db:Pool) {
		this.#db = db;
	}

	async insert_newUser (user: types.TNewUser): Promise<void> {
		if (!user) throw customTypeError('insert_newUser: !user');
		const insertUser_query = format(
			// eslint-disable-next-line indent
	`INSERT INTO
		registered_user(first_name, last_name, email, active, password_hash, ip_id, user_agent_id)
	VALUES
		(%1$L, %2$L, %3$L, %4$L, %5$L, %6$L, %7$L)`,
			user.first_name, user.last_name, user.email, true, user.password_hash, user.ipId, user.userAgentId);
		await this.#db.query(insertUser_query);
	}
	
	async insert_passwordReset ({ userId, resetString, ipId, userAgentId }: types.TInsertPasswordReset): Promise<void> {
		const query = format('INSERT INTO password_reset (registered_user_id, reset_string, ip_id, user_agent_id) VALUES(%1$L, %2$L, %3$L, %4$L)',
			userId, resetString, ipId, userAgentId);
		await this.#db.query(query);
	}
	
	async select_passwordReset_get (resetString: string): Promise<types.TPasswordResetSelectGet|undefined> {
		if (!resetString) throw customTypeError('select_passwordReset: !resetString');
		const query = format(
			// eslint-disable-next-line indent
	`SELECT
		CASE WHEN tfs.two_fa_secret IS NOT NULL then true ELSE false END as two_fa_active,
		CASE WHEN (SELECT COUNT(*) FROM two_fa_backup WHERE registered_user_id = pr.registered_user_id) > 0 THEN true ELSE false END AS two_fa_backup
	FROM
		password_reset pr
	LEFT JOIN
		two_fa_secret tfs
		ON pr.registered_user_id = tfs.registered_user_id
	WHERE
		pr.reset_string = %1$L
		AND NOW () <= pr.timestamp + INTERVAL '1 hour'
		AND pr.consumed = false`,
			resetString);

		const { rows } = await this.#db.query(query);
		return rows[0];
	}
		
	async select_passwordReset_patch (resetString: string): Promise<types.TPasswordResetSelectPatch|undefined> {
		if (!resetString) throw customTypeError('select_passwordReset: !resetString');

		const query = format(
			// eslint-disable-next-line indent
	`SELECT
		pr.password_reset_id, pr.registered_user_id,
		ru.first_name, ru.email,
		tfs.two_fa_secret,
		CASE WHEN (SELECT COUNT(*) FROM two_fa_backup WHERE registered_user_id = pr.registered_user_id) > 0 THEN true ELSE false END AS two_fa_backup
	FROM
		password_reset pr
	LEFT JOIN
		two_fa_secret tfs
		ON pr.registered_user_id = tfs.registered_user_id
	LEFT JOIN
		registered_user ru
		ON pr.registered_user_id = ru.registered_user_id
	WHERE
		pr.reset_string = %1$L
		AND NOW () <= pr.timestamp + INTERVAL '1 hour'
		AND pr.consumed = false`,
			resetString);
		const { rows } = await this.#db.query(query);
		return rows[0];
	}

	async update_passwordReset_transaction ({ stringId, password_hash, userId }: types.TPasswordResetInsert): Promise<void> {
		if (!stringId || !password_hash ||! userId) throw customTypeError('update_passwordReset_transaction: !stringId || !password_hash ||! userId ');
		const client = await this.#db.connect();
		try {
			await client.query('BEGIN');
			const passwordUpdate_query = format('UPDATE registered_user SET password_hash = %1$L WHERE registered_user_id = %2$L AND active = true', password_hash, userId);
			const resetString_query = format('UPDATE password_reset SET consumed = true WHERE password_reset_id = %1$L', stringId);
			await client.query(passwordUpdate_query);
			await client.query(resetString_query);
			await client.query('COMMIT');
		} catch (e) {
			await client.query('ROLLBACK');
			throw e;
		} finally {
			client.release();
		}
	}
}

export const incognitoQueries = new Incognito_Queries(postgresql);
