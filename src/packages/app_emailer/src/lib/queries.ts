import { APP_NAME } from '../config/env';
import { AppNameId, AppStatusId, EmailId, TSendEmail } from '../types';
import { LogEntry } from 'winston';
import { Pool } from 'pg';
import { postgresql } from '../config/db_postgres';
import format from 'pg-format';
import { customTypeError } from '../config/customError';

class Queries {

	#db!: Pool;

	constructor (db: Pool) {
		this.#db = db;
	}

	private async select_appNameId (app_name: string): Promise<AppNameId> {
		const query = format(`SELECT app_name_id FROM app_name WHERE app = %1$L`, app_name);
		const { rows } = await this.#db.query(query);
		return rows[0]?.app_name_id ?? this.insert_app_name(app_name);
	}
	
	private async insert_app_name (app_name: string): Promise<AppNameId> {
		const query = format(`INSERT INTO app_name(app) VALUES(%1$L) RETURNING app_name_id`, app_name);
		const { rows } = await this.#db.query(query);
		return rows[0].app_name_id;
	}
	
	private async select_appStatusId (): Promise<AppStatusId|undefined> {
		const appNameId = await this.select_appNameId(APP_NAME);
		const query = format(`SELECT app_status_id FROM app_status WHERE app_name_id = %1$L`, appNameId);
		const { rows } = await this.#db.query(query);
		return rows[0]?.app_status_id;
	}
	
	private async update_appStatus (appStatusId: AppStatusId, status: boolean): Promise<void> {
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
		const statusId = await this.select_appStatusId();
		if (statusId) await this.update_appStatus(statusId, status);
		else {
			const appNameId = await this.select_appNameId(APP_NAME);
			const statusQuery = format('INSERT INTO app_status(app_name_id, online) VALUES(%1$L, %2$L)', appNameId, status);
			await this.#db.query(statusQuery);
		}
	}
	
	async select_appStatusIdFromName () : Promise<AppStatusId|undefined> {
		const statusId = await this.select_appStatusId();
		return statusId;
	}

	async insert_emailLog ({ email, userId, title, rawBody, ipId, userAgentId }: TSendEmail): Promise<void> {
		if (!email || !title || !rawBody || !ipId || !userAgentId) throw customTypeError('insert_emailLog: !email || !title || !rawBody || !ipId || !userAgentId');
		const query = format(
		// eslint-disable-next-line indent
`INSERT INTO
	email_log(email_body, email_title, email, registered_user_id, user_agent_id, ip_id )
VALUES
	(%1$L, %2$L, %3$L, %4$L, %5$L, %6$L)`,
			rawBody, title, email, userId, userAgentId, ipId);
		await this.#db.query(query);
	}

	/**
	** How many non security emails have been sent today?
	*/
	async select_emailTodayCount (email: string): Promise<number> {
		if (!email) throw customTypeError('select_emailTodayCount: !email');
		const query = format (
		// eslint-disable-next-line indent
`SELECT COUNT(*) 
FROM
	email_log el
LEFT JOIN
	email_address ea
	ON
		el.email_address_id = ea.email_address_id
WHERE
	ea.email = %1$L
AND
	DATE(el.timestamp) >= DATE(NOW()) AND DATE(el.timestamp) <= DATE(NOW())
AND
	el.security_email = FALSE
AND
	el.sent = TRUE`,
			email
		);
		const { rows } = await this.#db.query(query);
		return rows[0]?.count;
	}

	/**
	** Has a non security email been sent in the past 15 seconds?
	*/
	async select_emailRecent (email: string): Promise<boolean> {
		if (!email) throw customTypeError('select_emailRecent: !email');
		const query = format (
		// eslint-disable-next-line indent
`SELECT
	*
FROM
	email_log el
LEFT JOIN
	email_address ea
	ON
		el.email_address_id = ea.email_address_id
WHERE
	email = %1$L
	AND NOW () <= el.timestamp + INTERVAL %2$L
	AND el.security_email = FALSE
	AND el.sent = TRUE`,
			email, '5 seconds'
		);
		const { rows } = await this.#db.query(query);
		return rows[0] ? true : false;
	}

	async select_emailAddressId (email: string): Promise<EmailId|undefined> {
		if (!email) throw customTypeError('select_emailId: !email');
		const query = format(`SELECT email_address_id FROM email_address WHERE email = lower(%1$L)`,
			email);
		const { rows } = await this.#db.query(query);
		return rows[0] ? rows[0].email_address_id : undefined;
	}

}

export const queries = new Queries(postgresql);