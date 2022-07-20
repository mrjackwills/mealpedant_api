import { customTypeError } from '../../config/customError';
import { postgresql } from '../../config/db_postgres';
import { Pool } from 'pg';
import format from 'pg-format';
import * as types from '../../types';

class User_Queries {
	#db!: Pool;

	constructor (db:Pool) {
		this.#db = db;
	}

	async delete_twoFABackupCodeAll (userId: types.UserId): Promise<void> {
		if (!userId) throw customTypeError('delete_twoFaBackupCodeAll(): !userId');
		const query = format('DELETE FROM two_fa_backup WHERE registered_user_id = %1$L', userId);
		await this.#db.query(query);
	}
	
	async insert_twoFABackup_transaction ({ ipId, userAgentId, userId, backupArray }: types.TInsertBackup): Promise<void> {
		const Client = await this.#db.connect();
		if (!backupArray || backupArray.length !== 10 || !ipId || !userAgentId || !userId) {
			throw customTypeError('insert_twoFABackup_transaction: !backupArray }| backupArray.length !== 10 || !ipId || !userAgentId || !userId');
		}
		try {
			Client.query('BEGIN');
			const promiseArray = [];
			for (const backup of backupArray) {
				const query = format('INSERT INTO two_fa_backup(two_fa_backup_code, ip_id, user_agent_id, registered_user_id) VALUES(%1$L, %2$L, %3$L, %4$L)',
					backup, ipId, userAgentId, userId);
				promiseArray.push(Client.query(query));
			}
			await Promise.all(promiseArray);
			await Client.query('COMMIT');
		} catch (e) {
			Client.query('ROLLBACK');
			throw e;
		} finally {
			Client.release();
		}
	}

	async insert_twoFASecret ({ userId, secret, ipId, userAgentId }: types.TInsertTFASecret): Promise<void> {
		if (!userId ||!secret|| !ipId ||!userAgentId) throw customTypeError('insert_twoFASecret: !userId ||!secret|| !ipId ||!userAgentId');

		const query = format('INSERT INTO two_fa_secret (ip_id, two_fa_secret, user_agent_id, registered_user_id) VALUES(%1$L, %2$L, %3$L, %4$L)',
			ipId, secret, userAgentId, userId);
		await this.#db.query(query);
	}

	async select_twoFABackupCount (userId: types.UserId): Promise<number> {
		if (!userId) throw customTypeError('select_twoFABackupCount: !userId');
		const query = format (`SELECT COUNT(*) FROM two_fa_backup WHERE registered_user_id = %1$L`, userId);
		const { rows } = await this.#db.query(query);
		return Number(rows[0].count);

	}

	async update_passwordHash ({ userId, passwordHash }: types.TUpdatePasswordHash): Promise<void> {
		if (!userId || !passwordHash) throw customTypeError('update_passwordHash: !userId || !passwordHash');
		const query = format('UPDATE registered_user SET password_hash = %1$L WHERE registered_user_id = %2$L AND active = true', passwordHash, userId);
		await this.#db.query(query);
	}

	async update_twoFAAlwaysRequired (userId: types.UserId, alwaysRequired: boolean): Promise<void> {
		if (!userId) throw customTypeError('update_twoFAAlwaysRequired: !userId');
		const query = format('UPDATE two_fa_secret SET always_required = %1$L WHERE registered_user_id = %2$L AND two_fa_secret IS NOT NULL',
			alwaysRequired, userId);
		await this.#db.query(query);
	}
}

export const userQueries = new User_Queries(postgresql);