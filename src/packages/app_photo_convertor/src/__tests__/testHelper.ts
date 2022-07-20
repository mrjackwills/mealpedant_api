import { randomBytes } from 'crypto';
import { cwd } from 'process';
import { postgresql } from '../config/db_postgres';

import { TLogLevels, TErrorLog } from '../types';
import format from 'pg-format';

abstract class BaseConstants {
	readonly cwd = cwd();
}

abstract class Constants extends BaseConstants {
	readonly cwd = cwd();
	readonly known_hash = '$argon2id$v=19$m=15360,t=6,p=1$Powd7zvLNSspaAOtrdrBEge01wd0uKK97KW3EoKyTDY$OwtiDSNeA2RrTjHZp9byy32uv6goosVwBgy8/zG5EmK4rpQdRPc9eM5DwsUr4HYmz/o6eqJnIR+zlajf33qoTg';
	readonly logErrorMessage = 'jest_error_test';
	readonly password = 'argon2 jest test';
}
abstract class RegexConstants extends Constants {
	readonly regex_argon = /^\$argon2id\$v=19\$m=15360,t=6,p=1\$[a-zA-Z0-9+/=]{43}\$[a-zA-Z0-9+/=]{86}$/;
	readonly regex_semver = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-((?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+([0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$/;
	readonly regex_converted = /^\d{4}-\d{2}-\d{2}_(D|J)_C_[a-fA-F0-9]{16}.jpeg$/;
}

abstract class SchemaErrorConstants extends RegexConstants {
	readonly #prefix = 'Invalid user data:';
	readonly schema_error_password = `${this.#prefix} passwords are required to be 12 characters minimum`;
}

abstract class Responses extends SchemaErrorConstants {
	readonly response_accessForbidden = { response: 'Access forbidden for current user' };
	readonly response_unknownDevice = { response: 'Device not known' };
}

abstract class Helpers extends Responses {

	async sleep (ms = 200): Promise<void> {
		return new Promise((resolve) => setTimeout(resolve, ms));
	}

	zeroPad (unit: number): string {
		return String(unit).padStart(2, '0');
	}

	get randomBoolean (): boolean {
		return Math.random() > 0.5;
	}

	get randomDate (): string {
		const startDate = new Date(2015, 4, 9);
		const endDate = new Date(2020, 9, 13);
		const data = new Date(startDate.getTime() + Math.random() * (endDate.getTime() - startDate.getTime()));
		return `${data.getFullYear()}-${this.zeroPad(data.getMonth() + 1)}-${this.zeroPad(data.getDate())}`;
	}

	randomNumber (min=1, max=1000): number {
		return Math.floor(Math.random() * (max - min) + min);
	}

	get randomPersonInitial (): 'D' | 'J' {
		return Math.random() < 0.5 ? 'D' : 'J';
	}

	get randomNumberAsString (): string {
		return String(this.randomNumber);
	}

	get randomString (): string {
		const output = Math.random().toString(36).substring(7);
		return output;
	}

	get randomMessageName (): string {
		return Math.random() > .5 ? 'photo::convert' : 'ping';
	}

	async randomOriginalFileName (): Promise<string> {
		const randomHex = await this.randomHex(16);
		const randomPerson = this.randomPersonInitial;
		const randomDate = this.randomDate;
		return `${randomDate}_${randomPerson}_O_${randomHex}.jpeg`;
	
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

	async query_selectError (message: string): Promise<TErrorLog> {
		const email_log_query = format('SELECT * FROM error_log WHERE message = %1$L', message);
		const { rows } = await this.postgres.query(email_log_query);
		return rows[0] as TErrorLog;
	}

	async query_selectErrorLatest (): Promise<TErrorLog> {
		const email_log_query = format('SELECT* FROM error_log ORDER BY timestamp DESC LIMIT 1');
		const { rows } = await this.postgres.query(email_log_query);
		return rows[0] as TErrorLog;
	}

}

export class TestHelper extends Queries {

	constructor () {
		super();
	}
	
	async afterAll (): Promise<void> {
		try {
			await new Promise((resolve) => setTimeout(() => resolve(true), 200));
		} catch (e) {
			// eslint-disable-next-line no-console
			console.log(e);
		}
	}
}
