import {
	API_PORT,
	API_VERSION_MAJOR,
	COOKIE_NAME,
	COOKIE_SECRET,
	DOMAIN,
	INVITE_USER,
	JEST_USER_EMAIL,
	JEST_USER_PASSWORD,
	LOCATION_BACKUP,
	LOCATION_LOG_COMBINED,
	LOCATION_LOG_ERROR,
	LOCATION_LOG_EXCEPTION,
	LOCATION_CWD,
	LOCATION_PHOTO_ORIGINAL,
	LOCATION_PHOTO_CONVERTED,
	LOCATION_TEMP,
	MODE_ENV_DEV,
	MODE_ENV_PRODUCTION,
	MODE_ENV_TEST,
	PG_DATABASE,
	PG_HOST,
	PG_PASS,
	PG_PORT,
	PG_USER,
	RABBITMQ_HOSTNAME,
	RABBITMQ_PASSWORD,
	RABBITMQ_QUEUE_NAME_ARGON,
	RABBITMQ_QUEUE_NAME_EMAILER,
	RABBITMQ_QUEUE_NAME_BACKUP,
	RABBITMQ_QUEUE_NAME_PHOTO_CONVERTOR,
	RABBITMQ_USERNAME,
	RABBITMQ_VHOST,
	REDIS_DB,
	REDIS_HOSTNAME,
	REDIS_PASS,
	REDIS_PORT,
	SHOW_LOGS,
	VUE_APP_COOKIE_KEY
} from '../../config/env';

import { describe, expect, it } from 'vitest';

describe('ENV test runner', () => {
		
	it('Expect all envs to be valid', async () => {
		expect.assertions(38);
		expect(API_PORT).toBeTruthy();
		expect(COOKIE_NAME).toBeTruthy();
		expect(COOKIE_SECRET).toBeTruthy();
		expect(DOMAIN).toBeTruthy();
		expect(INVITE_USER).toBeTruthy();
		expect(JEST_USER_EMAIL).toBeTruthy();
		expect(JEST_USER_PASSWORD).toBeTruthy();
		expect(LOCATION_BACKUP).toBeTruthy();
		expect(LOCATION_CWD).toBeTruthy();
		expect(LOCATION_LOG_COMBINED).toBeTruthy();
		expect(LOCATION_LOG_ERROR).toBeTruthy();
		expect(LOCATION_LOG_EXCEPTION).toBeTruthy();
		expect(LOCATION_PHOTO_ORIGINAL).toBeTruthy();
		expect(LOCATION_PHOTO_CONVERTED).toBeTruthy();
		expect(LOCATION_TEMP).toBeTruthy();
		expect(MODE_ENV_DEV).toBeFalsy(),
		expect(MODE_ENV_PRODUCTION).toBeFalsy(),
		expect(MODE_ENV_TEST).toBeTruthy(),
		expect(PG_DATABASE).toBeTruthy();
		expect(PG_HOST).toBeTruthy();
		expect(PG_PASS).toBeTruthy();
		expect(PG_USER).toBeTruthy();
		expect(REDIS_HOSTNAME).toBeTruthy();
		expect(REDIS_PASS).toBeTruthy();
		expect(RABBITMQ_HOSTNAME).toBeTruthy();
		expect(RABBITMQ_PASSWORD).toBeTruthy();
		expect(RABBITMQ_QUEUE_NAME_ARGON).toBeTruthy();
		expect(RABBITMQ_QUEUE_NAME_BACKUP).toBeTruthy();
		expect(RABBITMQ_QUEUE_NAME_EMAILER).toBeTruthy();
		expect(RABBITMQ_QUEUE_NAME_PHOTO_CONVERTOR).toBeTruthy();
		expect(RABBITMQ_USERNAME).toBeTruthy();
		expect(RABBITMQ_VHOST).toBeTruthy();
		expect(SHOW_LOGS).toBeTruthy();
		expect(typeof API_VERSION_MAJOR).toBe('number');
		expect(typeof PG_PORT).toBe('number');
		expect(typeof REDIS_DB).toBe('number');
		expect(typeof REDIS_PORT).toBe('number');
		expect(VUE_APP_COOKIE_KEY).toBeTruthy();
	});

});