import {
	APP_NAME,
	EMAIL_ADDRESS,
	EMAIL_HOST,
	EMAIL_NAME,
	EMAIL_PASSWORD,
	EMAIL_PORT,
	LOCATION_LOG_COMBINED,
	LOCATION_LOG_ERROR,
	LOCATION_LOG_EXCEPTION,
	LOCATION_TMP,
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
	RABBITMQ_QUEUE_NAME_EMAILER,
	RABBITMQ_USERNAME,
	RABBITMQ_VHOST,
	SHOW_LOGS,
	WWW_DOMAIN
} from '../../config/env';

import { describe, expect, it } from 'vitest';

describe('ENV test runner', () => {
		
	it('Expect all envs to be valid', async () => {
		expect.assertions(25);
		expect(APP_NAME).toBeTruthy();
		expect(EMAIL_ADDRESS).toBeTruthy();
		expect(EMAIL_HOST).toBeTruthy();
		expect(EMAIL_NAME).toBeTruthy();
		expect(EMAIL_PASSWORD).toBeTruthy();
		expect(typeof EMAIL_PORT).toBe('number');
		expect(LOCATION_LOG_COMBINED).toBeTruthy();
		expect(LOCATION_LOG_ERROR).toBeTruthy();
		expect(LOCATION_LOG_EXCEPTION).toBeTruthy();
		expect(LOCATION_TMP).toBeTruthy();
		expect(MODE_ENV_DEV).toBeFalsy(),
		expect(MODE_ENV_PRODUCTION).toBeFalsy(),
		expect(MODE_ENV_TEST).toBeTruthy(),
		expect(PG_DATABASE).toBeTruthy();
		expect(PG_HOST).toBeTruthy();
		expect(PG_PASS).toBeTruthy();
		expect(typeof PG_PORT).toBe('number');
		expect(PG_USER).toBeTruthy();
		expect(RABBITMQ_HOSTNAME).toBeTruthy();
		expect(RABBITMQ_PASSWORD).toBeTruthy();
		expect(RABBITMQ_QUEUE_NAME_EMAILER).toBeTruthy();
		expect(RABBITMQ_USERNAME).toBeTruthy();
		expect(RABBITMQ_VHOST).toBeTruthy();
		expect(SHOW_LOGS).toBeTruthy();
		expect(WWW_DOMAIN).toBeTruthy();
	});

});