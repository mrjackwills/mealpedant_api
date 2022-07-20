import { postgresql } from '../config/db_postgres';
import { randomBytes } from 'crypto';
import { TLogLevels, TErrorLog } from '../types';
import format from 'pg-format';
import { EmailTitle, EmailLineOne, EmailLineTwo, EmailButtonText } from '../types/enum_email';

abstract class Constants {
	readonly logErrorMessage = 'jest_error_test';
	readonly semver_regex = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-((?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+([0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$/;
}
abstract class EmailTemplateConstants extends Constants {
	readonly email_template_title = EmailTitle.PASSWORD_RESET;
	readonly email_template_firstName = 'first_name';
	readonly email_template_lineOne = EmailLineOne.PASSWORD_RESET;
	readonly email_template_lineTwo = EmailLineTwo.DIDNT_PASSWORD;
	readonly email_template_buttonText = EmailButtonText.RESET;
	readonly email_template_buttonLink = 'buttonLink';
	readonly email_template_htmlStarter =`<!doctype html><html xmlns="http://www.w3.org/1999/xhtml" xmlns:v="urn:schemas-microsoft-com:vml" xmlns:o="urn:schemas-microsoft-com:office:office"> <head> <title> Password reset requested </title> <!--[if !mso]><!--`;
	readonly email_template_buttonHtml = `<a href="buttonLink" style="display:inline-block;`;

}

abstract class Helpers extends EmailTemplateConstants {

	sleep (ms = 200): Promise<void> {
		return new Promise((resolve) => setTimeout(resolve, ms));
	}

	async randomHex (num=32): Promise<string> {
		return new Promise((resolve, reject) => {
			randomBytes(num, (e, buff) => {
				if (e) reject(e);
				resolve(buff.toString('hex').substring(0, num));
			});
		});
	}
}

abstract class Queries extends Helpers {

	postgres = postgresql;

	async query_selectErrorCount (level: TLogLevels = 'error'): Promise<number> {
		
		const email_log_query = format('SELECT count(*) FROM error_log WHERE level = %1$L', level);
		const { rows } = await this.postgres.query(email_log_query);
		return Number(rows[0].count);
	}

	async query_selectErrorLatest (): Promise<TErrorLog> {
		const email_log_query = format('SELECT* FROM error_log ORDER BY timestamp DESC LIMIT 1');
		const { rows } = await this.postgres.query(email_log_query);
		return rows[0] as TErrorLog;
	}
	
	async query_selectError (message: string): Promise<TErrorLog> {
		const email_log_query = format('SELECT * FROM error_log WHERE message = %1$L', message);
		const { rows } = await this.postgres.query(email_log_query);
		return rows[0] as TErrorLog;
	}

	protected async cleanDB (): Promise<void> {
		const Client = await this.postgres.connect();
		try {
			await Client.query('BEGIN');

			const errorQuery = format(`DELETE FROM error_log WHERE timestamp >= NOW() - INTERVAL '5 minutes'`);
			await this.postgres.query(errorQuery);

			await Client.query('COMMIT');
		} catch (e) {
			await Client.query('ROLLBACK');
			throw e;
		} finally {
			Client.release();
		}
	}

}

export class TestHelper extends Queries {

	async beforeEach (): Promise<void> {
		try {
			await this.cleanDB();
		} catch (e) {
			// eslint-disable-next-line no-console
			console.log(e);
		}
	}

	async afterAll (): Promise<void> {
		try {
			await new Promise((resolve) => setTimeout(() => resolve(true), 200));
			await this.postgres.end();
		} catch (e) {
			// eslint-disable-next-line no-console
			console.log(e);
		}
	}
	
}
