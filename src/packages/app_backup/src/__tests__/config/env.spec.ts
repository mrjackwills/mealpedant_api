import {
	APP_NAME,
	MODE_ENV_PRODUCTION,
	SHOW_LOGS,
	LOCATION_LOG_ERROR,
	LOCATION_LOG_COMBINED,
	LOCATION_LOG_EXCEPTION,
	LOCATION_ALL_LOGS,
	LOCATION_BACKUPS,
	LOCATION_SCRIPTS,
	LOCATION_STATIC,
	PG_DATABASE,
	PG_HOST,
	GPG_PASSWORD,
	PG_PASS,
	PG_PORT,
	PG_USER,
	RABBITMQ_PASSWORD,
	RABBITMQ_HOSTNAME,
	RABBITMQ_USERNAME,
	RABBITMQ_VHOST
} from '../../config/env';
import { describe, expect, it } from 'vitest';

describe('ENV test runner', () => {

		
	it('Expect all envs to be valid', async () => {
		expect.assertions(20);
		expect(APP_NAME).toBeTruthy();
		expect(LOCATION_LOG_COMBINED).toBeTruthy();
		expect(LOCATION_LOG_ERROR).toBeTruthy();
		expect(LOCATION_LOG_EXCEPTION).toBeTruthy();
		expect(MODE_ENV_PRODUCTION).toBeFalsy(),
		expect(PG_DATABASE).toBeTruthy();
		expect(PG_HOST).toBeTruthy();
		expect(PG_PASS).toBeTruthy();
		expect(typeof PG_PORT).toBe('number');
		expect(PG_USER).toBeTruthy();
		expect(SHOW_LOGS).toBeTruthy();
		expect(RABBITMQ_PASSWORD).toBeTruthy();
		expect(RABBITMQ_USERNAME).toBeTruthy();
		expect(RABBITMQ_VHOST).toBeTruthy();
		expect(RABBITMQ_HOSTNAME).toBeTruthy();
		expect(GPG_PASSWORD).toBeTruthy();
		expect(LOCATION_ALL_LOGS).toBeTruthy();
		expect(LOCATION_BACKUPS).toBeTruthy();
		expect(LOCATION_SCRIPTS).toBeTruthy();
		expect(LOCATION_STATIC).toBeTruthy();
	});


});