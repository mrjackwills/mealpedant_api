/* eslint-disable no-console */
/* eslint-disable @typescript-eslint/no-explicit-any, @typescript-eslint/ban-types */
import '../config/env';
import { api } from '../app/api';
import { API_VERSION_MAJOR, INVITE_USER, JEST_USER_EMAIL } from '../config/env';
import { argonMock } from './mocks/argon';
import { authenticator } from 'otplib';
import { cwd } from 'process';
import { exec } from 'child_process';
import { mock_backup } from './mocks/backup';
import { mock_photoConvertor } from './mocks/photo_convertor';
import { mocked } from 'jest-mock';
import { postgresql } from '../config/db_postgres';
import { rabbit_validateHash, rabbit_createHash, rabbit_ping, rabbit_photoConvertor, rabbit_backup } from '../lib/rabbitRpc';
import { randomBytes } from 'crypto';
import { Redis } from '../config/db_redis';
import { send_email } from '../lib/rabbitSend';
import { TLogLevels, TErrorLog, TInsertMeal } from '../types';
import { vi } from 'vitest';
import Axios, { AxiosError, AxiosInstance, AxiosResponse, AxiosRequestConfig } from 'axios';
import format from 'pg-format';
import http from 'http';
import { Dictionary, Request } from 'express-serve-static-core';

vi.mock('../lib/rabbitRpc');
vi.mock('../lib/rabbitSend');

abstract class BaseConstants {
	readonly axios_port = 9000;
	readonly axios_ip = '127.0.0.1';
	readonly mockedRabbitSendEmail = mocked(send_email, true);
	readonly mockedCreateHash = mocked(rabbit_createHash, true);
	readonly mockedValidateHash = mocked(rabbit_validateHash, true);
	readonly mockedRabbitPing = mocked(rabbit_ping, true);
	readonly mockedPhotoConvertor= mocked(rabbit_photoConvertor, true);
	readonly mockedBackup= mocked(rabbit_backup, true);
	readonly apiVersionMajor = API_VERSION_MAJOR;
	readonly VMajor = `/v${API_VERSION_MAJOR}`;
	readonly cwd = cwd();
}

abstract class UserConstants extends BaseConstants {
	readonly email = JEST_USER_EMAIL;
	readonly email_anon = `anon_${this.email}`;
	readonly firstName = 'John';
	readonly invite = INVITE_USER;
	readonly lastName = 'Smith';
	readonly password = 'argon2 jest test';
	readonly userAgent = 'jest-userAgent';
	readonly argon_known_hash = '$argon2id$v=19$m=15360,t=6,p=1$4YLrt7i7kqJKRaMnle7fvDdOc9xkkM04VZAow0QApjo$k0qcO3yUki9sHC00uMzest/yW6f17Fw8ITFwp6R6CwpNggwf44/YykimlhNNlM4R+WOkJsv5S9z2Av+I2rL+RA';
	readonly two_fa_backups = [ '0000000000000000', '1111111111111111' ] as const;
	readonly two_fa_backups_argon = [ argonMock.createHash(this.two_fa_backups[0]), argonMock.createHash(this.two_fa_backups[1]) ];
}

abstract class DBConstants extends UserConstants {
	protected readonly backupName = `dev_mealpedant_pg_dump.tar`;
	readonly originalMealId = '1654';
	readonly logErrorMessage = 'jest_error_test';
	readonly knownResponse = {
		meal: {
			id: '2746',
			date: '2019-02-09',
			person: 'Jack',
			category: 'GERMAN',
			description: 'Käsespätzle, side salad, and also made a strawberry cheesecake, veggie',
			restaurant: false,
			takeaway: false,
			vegetarian: true,
			meal_photo_id: '80',
			photoNameOriginal: '2019-02-09_J_O_e0a940b221a1e988.jpeg',
			photoNameConverted: '2019-02-09_J_C_c82bc642089df43f.jpeg'
		}
	};
	
}

abstract class ResponseConstants extends DBConstants {
	readonly response_empty = { response: '' };
	readonly response_incorrectPasswordOrToken = { response: 'Invalid password and/or Two Factor Authentication token' };
	readonly response_invalidLogin = { response: 'Invalid email and/or password' };
	readonly response_invalidUserData = { response: 'Invalid user data' };
	readonly response_invalidToken = { response: 'Two Factor Authentication token invalid' };
	readonly response_noPassword = { response: 'Password not provided' };
	readonly response_noToken = { response: 'Two Factor Authentication token not provided' };
	readonly response_unauthorized = { response: 'Invalid Authorization' };
	readonly response_onself = { response: 'Not allowed to perform on self' };
	readonly response_unknown = { response: 'Unknown endpoint' };
	readonly responseUser = {
		response: {
			email: this.email,
			two_fa_backup: false,
			two_fa_count: 0,
			two_fa_always_required: false,
			two_fa_active: false,
		}
	};
	readonly responseAdmin = {
		response: {
			... this.responseUser.response,
			admin: true
		}
	};
}

abstract class SchemaErrorConstants extends ResponseConstants {
	private readonly prefix = 'Invalid user data:';
	readonly schema_error_password = `${this.prefix} passwords are required to be 10 characters minimum`;
	readonly schema_error_token = `${this.prefix} token format incorrect`;
	readonly schema_error_backup_token = `${this.prefix} backup token format incorrect`;
	readonly schema_error_shared_verification = `${this.prefix} incorrect verification data`;
	readonly schema_error_filename = `${this.prefix} incorrect filename`;
	readonly schema_error_email = `${this.prefix} email address`;
	readonly schema_error_date = `${this.prefix} date invalid`;
	readonly schema_error_person = `${this.prefix} person unrecognised`;
	readonly schema_error_description = `${this.prefix} description`;
	readonly schema_error_category = `${this.prefix} category`;
	readonly schema_error_restaurant = `${this.prefix} restaurant`;
	readonly schema_error_takeaway = `${this.prefix} takeaway`;
	readonly schema_error_vegetarian = `${this.prefix} vegetarian`;
	readonly schema_error_photo_converted = `${this.prefix} converted photo`;
	readonly schema_error_photo_original = `${this.prefix} original photo`;
}

abstract class RegexConstants extends SchemaErrorConstants {
	readonly regex_backupToken = /^[0-9a-fA-F]{16}$/;
	readonly regex_passwordResetString = /^[0-9a-fA-F]{256}$/;
	readonly regex_verifyString = /^verify:string:[0-9a-fA-F]{256}$/;
	readonly regex_photoConverted = /^\d{4}-\d{2}-\d{2}_(D|J)_C_[a-fA-F0-9]{16}.jpeg$/;
	readonly regex_photoOriginal = /^\d{4}-\d{2}-\d{2}_(D|J)_O_[a-fA-F0-9]{16}.jpeg$/;
	readonly regex_session = /^session:[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$/;
	readonly regex_sessionSet = /^set:session:\d+$/;
	readonly regex_sql_full = /^mealpedant_\d{4}-\d{2}-\d{2}_\d{2}\.\d{2}\.\d{2}_LOGS_PHOTOS_REDIS_SQL_[0-9a-fA-F]{8}\.tar\.gpg$/;
	readonly regex_sql_only = /^mealpedant_\d{4}-\d{2}-\d{2}_\d{2}\.\d{2}\.\d{2}_LOGS_REDIS_SQL_[0-9a-fA-F]{8}\.tar\.gz\.gpg$/;
	readonly regex_error = /^Internal server error: [a-z0-9]{8}-[a-z0-9]{4}-[a-z0-9]{4}-[a-z0-9]{4}-[a-z0-9]{12}$/;
	readonly regex_argon = /^\$argon2id\$v=19\$m=15360,t=6,p=1\$[a-zA-Z0-9+/=]{43}\$[a-zA-Z0-9+/=]{86}$/;
	readonly regex_semver = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-((?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+([0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$/;
	readonly regex_otpSecretRegex = /^[a-zA-Z0-9]{16}$/;

}

abstract class HelperFunctions extends RegexConstants {
	
	sleep (ms = 250): Promise<void> {
		return new Promise((resolve) => setTimeout(resolve, ms));
	}

	zeroPad (unit: number): string {
		return String(unit).padStart(2, '0');
	}

	randomPerson (): 'Dave' | 'Jack' {
		return Math.random() < 0.5 ? 'Dave' : 'Jack';
	}

	randomPersonInitial (): 'D' | 'J' {
		return Math.random() < 0.5 ? 'D' : 'J';
	}

	randomBoolean (): boolean {
		return Math.random() > 0.5;
	}

	randomDate (): string {
		const startDate = new Date(2015, 4, 9);
		const endDate = new Date(2020, 9, 13);
		const data = new Date(startDate.getTime() + Math.random() * (endDate.getTime() - startDate.getTime()));
		return `${data.getFullYear()}-${this.zeroPad(data.getMonth() + 1)}-${this.zeroPad(data.getDate())}`;
	}
	
	randomHex (num=32): Promise<string> {
		return new Promise((resolve, reject) => {
			randomBytes(num, (e, buff) => {
				if (e) reject(e);
				resolve(buff.toString('hex').substring(0, num));
			});
		});
	}

	randomNumber (min=1, max=1000): number {
		return Math.floor(Math.random() * (max - min) + min);
	}

	randomNumberAsString (): string {
		return String(this.randomNumber());
	}

	generateTomorrow () : string {
		const now = new Date();
		const tomorrow = new Date(now.setDate(now.getDate() + 1));
		const year = tomorrow.getFullYear();
		const month = this.zeroPad(tomorrow.getMonth() + 1);
		const day = this.zeroPad(tomorrow.getDate());
		return `${year}-${month}-${day}`;
	}

	generateToday (): string {
		const now = new Date();
		return `${now.getFullYear()}-${this.zeroPad(now.getMonth() + 1)}-${this.zeroPad(now.getDate())}`;
	}

	async createMeal (photo?: boolean): Promise<object> {
		const date = this.randomDate();
		const person = this.randomPerson();
		const output: TInsertMeal = {
			date,
			person,
			category: await this.randomHex(10),
			description: await this.randomHex(10),
			restaurant: false,
			takeaway: false,
			vegetarian: false,
		};
		if (photo) {
			output.photoNameOriginal = `${date}_${person.substring(0, 1)}_O_${await this.randomHex(16) }.jpeg`;
			output.photoNameConverted = `${date}_${person.substring(0, 1)}_C_${await this.randomHex(16) }.jpeg`;
		}

		return output;
	}
}

abstract class Server extends HelperFunctions {

	_server?: http.Server;

	createServer (): void {
		if (this._server) return;
		this._server = http.createServer(api);
		this._server.listen(this.axios_port, this.axios_ip, () => undefined);
	}

	async closeSever (): Promise<void> {
		return new Promise((resolve) => {
			if (!this._server) resolve();
			this._server?.close(() => {
				resolve();
			});
		}
		);
	}
}
	
abstract class BaseAxios extends Server {

	protected cookie = '';

	axios!: AxiosInstance;
	uploadAxios!: AxiosInstance;
	
	constructor () {
		super();
		this.axios = Axios.create({
			baseURL: `http://127.0.0.1:${this.axios_port}${this.VMajor}`,
			withCredentials: true,
			headers: {
				'Accept': 'application/json',
				'Content-Type': 'application/json; charset=utf-8',
				'Cache-control': 'no-cache',
			},
			
		});
	
		this.axios.interceptors.response.use(
			(config) => Promise.resolve(config),
			(error) => Promise.reject(error)
		);

		this.uploadAxios = Axios.create({
			baseURL: `http://127.0.0.1:${this.axios_port}${this.VMajor}`,
			withCredentials: true,
			headers: {
				'Content-Type': 'multipart/form-data',
				'Cache-control': 'no-cache'
			},
			maxContentLength: Infinity,
			maxBodyLength: Infinity,
			
		});
	
		this.uploadAxios.interceptors.response.use(
			(config) => Promise.resolve(config),
			(error) => Promise.reject(error)
		);
		
	}

	injectAuthCookies (): void {
		this.axios.interceptors.request.use((config: AxiosRequestConfig): AxiosRequestConfig => {
			config.headers = { ...config.headers, Cookie: this.cookie, 'User-Agent': this.userAgent };
			return config;
		});
		this.uploadAxios.interceptors.request.use((config: AxiosRequestConfig): AxiosRequestConfig => {
			config.headers = { ...config.headers, Cookie: this.cookie, 'User-Agent': this.userAgent };
			return config;
		});
	}

	returnAxiosError (e: unknown): AxiosError {
		return <AxiosError>e;
	}
}

abstract class Cookie extends BaseAxios {

	two_fa_secret?: string;

	get authedCookie (): string {
		return this.cookie;
	}

	setCookieUsingResponse (response: AxiosResponse): string {
		if (response.headers['set-cookie']) this.cookie = response?.headers['set-cookie'][0] ?? '';
		// this.cookie = String(response?.headers['set-cookie'][0]);
		this.injectAuthCookies();
		return this.cookie;
	}

	clearCookie (): void {
		this.cookie = '';
	}

	generateToken (secret: string): string {
		return authenticator.generate(secret);
	}

	generateTokenFromString (secret: string): string {
		return authenticator.generate(secret);
	}

	generateSecret (): string {
		const secret = authenticator.generateSecret();
		this.two_fa_secret = secret;
		return secret;
	}
	
	generateBadResponse (text: string): {response: string} {
		return { response: `${this.response_invalidUserData.response}: ${text}` };
	}

	generateIncorrectToken (token: string): string {
		const index = Math.floor(Math.random() * (token.length -1));
		const randomChar = token[index];
		const newDigit = String(Number(randomChar) + 1).substring(0, 1);
		const tmp = token.split('');
		tmp[index] = newDigit;
		const output = tmp.join('');
		return output;
	}
}

abstract class Requests extends Cookie {
	
	async request_signin (data: {body?: object, token?: string} = {}): Promise<AxiosResponse> {
		const correct = { email: this.email, password: this.password };
		const body = data.body ? data.body : data.token ? { ...correct, token: data.token } : correct;

		const request = await this.axios.post(`incognito/signin`, body);
		if (request.status === 200) this.setCookieUsingResponse(request);
		return request;
	}

}

abstract class PostgresIds extends Requests {
	user_agent_id?: string;
	registered_user_id?: string;
	anon_registered_user_id?: string;
	ip_id?: string;
	admin_id?: string;
	two_fa_secret_id?: string;
	
}

abstract class Databases extends PostgresIds {
	postgres = postgresql;
	redis = Redis;
}

abstract class Queries extends Databases {
	
	async insertUser (): Promise<void> {
		const Client = await this.postgres.connect();
		try {
			await Client.query('BEGIN');

			// If no result, then should insert
			const ip_address_select_query = format(`SELECT ip_id FROM ip_address WHERE ip = %1$L`, this.axios_ip);
			const { rows: ipResult } = await Client.query(ip_address_select_query);
			this.ip_id = ipResult[0].ip_id;

			const user_agent_insert_query = format(`INSERT INTO user_agent (user_agent_string) VALUES (%1$L) RETURNING user_agent_id`, this.userAgent);
			const { rows: userAgentResult } = await Client.query(user_agent_insert_query);
			this.user_agent_id = userAgentResult[0].user_agent_id;

			const registered_user_query = format(
				`INSERT INTO
				registered_user (first_name, last_name, email, active, password_hash, ip_id, user_agent_id)
				VALUES (%1$L, %2$L, %3$L, %4$L, %5$L, %6$L, %7$L) RETURNING registered_user_id`, this.firstName, this.lastName, this.email, true, this.argon_known_hash, this.ip_id, this.user_agent_id);
			const { rows: registeredUserResult } = await Client.query(registered_user_query);
			this.registered_user_id = registeredUserResult[0].registered_user_id;
		
			await Client.query('COMMIT');
		} catch (e) {
			await Client.query('ROLLBACK');
			throw e;
		} finally {
			Client.release();
		}
	}

	async insertAnonUser (): Promise<void> {
		const Client = await this.postgres.connect();
		if (!this.registered_user_id) throw Error('insertAnonUser: !this.registered_user_id');
		try {
			await Client.query('BEGIN');

			const registered_user_query = format(
				`INSERT INTO
				registered_user (first_name, last_name, email, active, password_hash, ip_id, user_agent_id)
				VALUES (%1$L, %2$L, %3$L, %4$L, %5$L, %6$L, %7$L) RETURNING registered_user_id`, this.firstName, this.lastName, this.email_anon, true, this.argon_known_hash, this.ip_id, this.user_agent_id);
			const { rows: registeredUserResult } = await Client.query(registered_user_query);
			this.anon_registered_user_id = registeredUserResult[0].registered_user_id;
		
			await Client.query('COMMIT');
		} catch (e) {
			await Client.query('ROLLBACK');
			throw e;
		} finally {
			Client.release();
		}
	}

	async insertAdminUser (): Promise<void> {
		await this.insertUser();
		const Client = await this.postgres.connect();
		try {
			await Client.query('BEGIN');
			const admin_insert_query = format(`INSERT INTO admin_user (registered_user_id, ip_id, admin) VALUES (%1$L, %2$L, %3$L) RETURNING admin_id`, this.registered_user_id, this.ip_id, true);
			const { rows: admin_result } = await Client.query(admin_insert_query);
			this.admin_id = admin_result[0].admin_id;
			await Client.query('COMMIT');
		} catch (e) {
			await Client.query('ROLLBACK');
			throw e;
		} finally {
			Client.release();
		}
	}

	async insert2FA (): Promise<void> {
		if (!this.registered_user_id) throw Error('!this.registered_user_id');
		const secret = this.generateSecret();
		const admin_insert_query = format(`INSERT INTO two_fa_secret (registered_user_id, ip_id, user_agent_id, two_fa_secret) VALUES (%1$L, %2$L, %3$L, %4$L) RETURNING two_fa_secret_id`, this.registered_user_id, this.ip_id, this.user_agent_id, secret);
		const { rows: admin_result } = await this.postgres.query(admin_insert_query);
		this.two_fa_secret_id = admin_result[0].two_fa_secret_id;
	}

	async insert2FAAlwaysRequired (): Promise<void> {
		if (!this.registered_user_id) throw Error('!this.registered_user_id');
		const secret = this.generateSecret();
		const admin_insert_query = format(`INSERT INTO two_fa_secret (registered_user_id, ip_id, user_agent_id, two_fa_secret, always_required) VALUES (%1$L, %2$L, %3$L, %4$L, %5$L) RETURNING two_fa_secret_id`,
			this.registered_user_id, this.ip_id, this.user_agent_id, secret, true);
		const { rows: admin_result } = await this.postgres.query(admin_insert_query);
		this.two_fa_secret_id = admin_result[0].two_fa_secret_id;
	}

	async insertAnon2FA (): Promise<string> {
		if (!this.anon_registered_user_id) throw Error('!this.registered_user_id');
		const secret = this.generateSecret();
		const admin_insert_query = format(`INSERT INTO two_fa_secret (registered_user_id, ip_id, user_agent_id, two_fa_secret) VALUES (%1$L, %2$L, %3$L, %4$L) RETURNING two_fa_secret_id`, this.anon_registered_user_id, this.ip_id, this.user_agent_id, secret);
		await this.postgres.query(admin_insert_query);
		return secret;
	}

	async insert2FABackup (): Promise<void> {
		if (!this.registered_user_id) throw Error('!this.registered_user_id');
	
		const admin_insert_query = format(`
		INSERT INTO two_fa_backup (registered_user_id, ip_id, user_agent_id, two_fa_backup_code)
		VALUES (%1$L, %2$L, %3$L, %4$L),
		(%1$L, %2$L, %3$L, %5$L)`,
		this.registered_user_id, this.ip_id, this.user_agent_id, this.two_fa_backups_argon[0], this.two_fa_backups_argon[1]);
		await this.postgres.query(admin_insert_query);
	}

	async query_selectLoginCount (): Promise<number> {
		const query = format(`SELECT count(*) FROM login_history WHERE registered_user_id = %1$L AND success = 'true'`, this.registered_user_id);
		const { rows } = await this.postgres.query(query);
		return rows[0].count;
	}

	async query_selectBackupCodes (): Promise<Array<any>> {
		const query = format(`SELECT * FROM two_fa_backup WHERE registered_user_id = %1$L`, this.registered_user_id);
		const { rows } = await this.postgres.query(query);
		return rows;
	}

	async query_selectUserAgent () :Promise<string|undefined> {
		const query = format(`SELECT * FROM user_agent WHERE user_agent_string = %1$L`, this.userAgent);
		const { rows } = await this.postgres.query(query);
		return rows[0].user_agent_string;
	}

	async query_selectIp () :Promise<string|undefined> {
		const query = format(`SELECT * FROM ip_address WHERE ip = %1$L`, this.axios_ip);
		const { rows } = await this.postgres.query(query);
		return rows[0]?.ip;
	}

	async query_select2FABackupCount (): Promise<number> {
		if (!this.registered_user_id) throw Error('!this.registered_user_id');
		const query = format(`SELECT COUNT(*) FROM two_fa_backup WHERE registered_user_id = %1$L`,
			this.registered_user_id);
		const { rows } = await this.postgres.query(query);
		return Number(rows[0].count);
	}

	async query_selectError (message: string): Promise<TErrorLog> {
		const email_log_query = format('SELECT * FROM error_log WHERE message = %1$L', message);
		const { rows } = await this.postgres.query(email_log_query);
		return rows[0] as TErrorLog;
	}

	async query_selectErrorCount (level: TLogLevels = 'error'): Promise<number> {
		const email_log_query = format('SELECT count(*) FROM error_log WHERE level = %1$L', level);
		const { rows } = await this.postgres.query(email_log_query);
		return Number(rows[0].count);
	}

	async query_selectUser (): Promise<any> {
		const query = format(`SELECT * FROM registered_user WHERE email = %1$L`, this.email);
		const { rows } = await this.postgres.query(query);
		return rows[0];
	}

	async query_selectPasswordReset (): Promise<any> {
		const query = format(
			// eslint-disable-next-line indent
		`SELECT ru.email, pr.password_reset_id, pr.timestamp, pr.reset_string
		FROM password_reset pr
		LEFT JOIN registered_user ru
		ON pr.registered_user_id = ru.registered_user_id
		WHERE email = %1$L`,
			this.email);
		const { rows } = await this.postgres.query(query);
		return rows[0];
	}

	async getRedisCache () : Promise<Array<string | null>> {
		const data = await Promise.all([
			this.redis.hget('cache:allCategory', 'data'),
			this.redis.hget('cache:allMeal', 'data'),
			this.redis.get('cache:lastMealEditId'),
		]);
		return data;
	}

	async cleanDB (): Promise<void> {
		const Client = await this.postgres.connect();
		this.clearCookie();

		try {
			await Client.query('BEGIN');
			if (this.admin_id) {
				const adminDeleteQuery = format(`DELETE from admin_user WHERE admin_id = %1$L`, this.admin_id);
				await Client.query(adminDeleteQuery);
			}
		
			const twoFASecretBackupDeleteQuery = format(`DELETE FROM two_fa_backup WHERE registered_user_id = %1$L OR registered_user_id = %2$L`, this.registered_user_id, this.anon_registered_user_id);
			await Client.query(twoFASecretBackupDeleteQuery);

			const twoFASecretDeleteQuery = format(`DELETE FROM two_fa_secret WHERE registered_user_id = %1$L OR registered_user_id = %2$L`, this.registered_user_id, this.anon_registered_user_id);
			await Client.query(twoFASecretDeleteQuery);

			const twoFABackupDeleteQuery = format(`DELETE FROM two_fa_backup WHERE registered_user_id = %1$L OR registered_user_id = %2$L`, this.registered_user_id, this.anon_registered_user_id);
			await Client.query(twoFABackupDeleteQuery);

			const userDeleteQuery = format(`DELETE from registered_user WHERE email = %1$L`, this.email);
			await Client.query(userDeleteQuery);

			const userAnonDeleteQuery = format(`DELETE from registered_user WHERE email = %1$L`, this.email_anon);
			await Client.query(userAnonDeleteQuery);

			const loginDeleteQuery = format(`DELETE from login_attempt WHERE registered_user_id = %1$L OR registered_user_id = %2$L`, this.registered_user_id, this.anon_registered_user_id);
			await Client.query(loginDeleteQuery);
		
			const userAgentDeleteQuery = format(`DELETE from user_agent WHERE user_agent_string = %1$L`, this.userAgent);
			await Client.query(userAgentDeleteQuery);
			const errorQuery = format(`DELETE FROM error_log WHERE timestamp >= NOW() - INTERVAL '5 minutes'`);

			await Client.query(errorQuery);
		
			this.user_agent_id = undefined;
			this.registered_user_id = undefined;
			this.ip_id = undefined;
			this.admin_id = undefined;
			this.two_fa_secret = undefined;
			this.two_fa_secret_id = undefined;

			await this.redis.flushdb();

			await Client.query('COMMIT');
		} catch (e) {
			await Client.query('ROLLBACK');
			// eslint-disable-next-line no-console
			console.log(e);
			throw e;
		} finally {
			Client.release();
		}
	}
}

export class TestHelper extends Queries {

	async pgDump (): Promise<void> {
		return new Promise((resolve, reject) => {
			exec(`pg_dump -U jack dev_mealpedant -h 127.0.0.1 --no-owner --format=t > /tmp/${this.backupName}`, undefined, (err, _stdout, _stderr) => {
				if (err) return reject(err);
				return resolve();
			});
		});
	}

	async pgRestore (): Promise<void> {
		return new Promise((resolve, reject) => {
			exec(`pg_restore -U jack -c -d dev_mealpedant -h 127.0.0.1 /tmp/${this.backupName}`, undefined, (err, _stdout, _stderr) => {
				if (err) return reject(err);
				return resolve();
			});
		});
	}

	implementMocks (): void {
		this.mockedValidateHash.mockImplementation(async (x) => {
			const output = await argonMock.validateHash(x);
			return output;
		});
		this.mockedCreateHash.mockImplementation(async (x) => {
			const output = await argonMock.createHash(x.password);
			return output;
		});
		this.mockedPhotoConvertor.mockImplementation(async (x) => {
			const output = await mock_photoConvertor(x);
			return output;
		});
		this.mockedRabbitPing.mockImplementation(async () => void '');

		this.mockedRabbitSendEmail.mockImplementation(async () => undefined);

		this.mockedBackup.mockImplementation(async (x) => {
			const output = await mock_backup(x);
			return output;
		});
	}

	async beforeAll (): Promise<void> {
		try {
			this.cookie = '';
			await this.cleanDB();
			vi.resetAllMocks();
			this.implementMocks();
			await this.cleanDB();
			await this.redis.flushdb();
			this.createServer();
		} catch (e) {
			// eslint-disable-next-line no-console
			console.log(e);
		}
	}

	async beforeEach (): Promise<void> {
		try {
			await this.cleanDB();
			this.cookie = '';
			vi.resetAllMocks();
			this.implementMocks();
		} catch (e) {
			// eslint-disable-next-line no-console
			console.log(e);
		}
	}

	async afterAll (): Promise<void> {
		try {
			await this.closeSever();
			await this.cleanDB();
			await this.redis.flushdb();
			await Redis.quit();
			Redis.disconnect();
			await postgresql.end();
			// console.log('done');
		} catch (e) {
			// eslint-disable-next-line no-console
			console.log(e);
		}
	}

}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export const mockRequest = (options?: Dictionary<any>): Request => {
	const getMethod = vi.fn();

	const mock: unknown = {
		app: {},
		baseUrl: '',
		body: {},
		cookies: {},
		fresh: true,
		hostname: '',
		ip: '127.0.0.1',
		ips: [],
		method: '',
		originalUrl: '',
		params: {},
		path: '',
		protocol: 'https',
		query: {},
		route: {},
		secure: true,
		signedCookies: {},
		stale: false,
		subdomains: [],
		xhr: true,
		accepts: vi.fn(),
		acceptsCharsets: vi.fn(),
		acceptsEncodings: vi.fn(),
		acceptsLanguages: vi.fn(),
		get: getMethod,
		header: getMethod,
		is: vi.fn(),
		range: vi.fn(),
		...options,
	};
	return <Request> mock;
};
