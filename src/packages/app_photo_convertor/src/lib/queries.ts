import { APP_NAME } from '../config/env';
import { AppNameId, AppStatusId } from '../types';
import { LogEntry } from 'winston';
import { Pool } from 'pg';
import { postgresql } from '../config/db_postgres';
import format from 'pg-format';


class Queries {

	#db!: Pool;

	constructor (db: Pool) {
		this.#db = db;
	}

	async #select_appNameId (app_name: string): Promise<AppNameId> {
		const query = format(`SELECT app_name_id FROM app_name WHERE app = %1$L`, app_name);
		const { rows } = await this.#db.query(query);
		return rows[0]?.app_name_id ?? this.#insert_app_name(app_name);
	}
	
	async #insert_app_name (app_name: string): Promise<AppNameId> {
		const query = format(`INSERT INTO app_name(app) VALUES(%1$L) RETURNING app_name_id`, app_name);
		const { rows } = await this.#db.query(query);
		return rows[0].app_name_id;
	}
	
	async #select_appStatusId (): Promise<AppStatusId|undefined> {
		const appNameId = await this.#select_appNameId(APP_NAME);
		const query = format(`SELECT app_status_id FROM app_status WHERE app_name_id = %1$L`, appNameId);
		const { rows } = await this.#db.query(query);
		return rows[0]?.app_status_id;
	}
	
	async #update_appStatus (appStatusId: AppStatusId, status: boolean): Promise<void> {
		const query = format(`UPDATE app_status SET online = %1$L, timestamp = NOW() WHERE app_status_id = %2$L`, !!status, appStatusId);
		await this.#db.query(query);
	}

	async insert_error (data: LogEntry): Promise<void> {
		try {
			if (!data.message || !data.level || !data.timestamp) return;
			const query = format('INSERT INTO error_log(timestamp, level, message, stack, uuid) VALUES(%1$L, %2$L, %3$L, %4$L, %5$L)',
				data.timestamp, data.level, data.message, data.stack, data.uuid);
			await this.#db.query(query);
		} catch (e) {
			// eslint-disable-next-line no-console
			console.log(e);
		}
	}
	
	async insert_appStatus (status: boolean): Promise<void> {
		const statusId = await this.#select_appStatusId();
		if (statusId) await this.#update_appStatus(statusId, status);
		else {
			const appNameId = await this.#select_appNameId(APP_NAME);
			const statusQuery = format('INSERT INTO app_status(app_name_id, online) VALUES(%1$L, %2$L)', appNameId, status);
			await this.#db.query(statusQuery);
		}
	}
	
	async select_appStatusIdFromName () : Promise<AppStatusId|undefined> {
		const statusId = await this.#select_appStatusId();
		return statusId;
	}
}

export const queries = new Queries(postgresql);