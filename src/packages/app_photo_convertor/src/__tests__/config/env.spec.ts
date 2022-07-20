import {
	APP_NAME,
	LOCATION_LOG_COMBINED,
	LOCATION_LOG_ERROR,
	LOCATION_LOG_EXCEPTION,
	LOCATION_PHOTO_STATIC_CONVERTED,
	LOCATION_PHOTO_STATIC_ORIGINAL,
	LOCATION_WATERMARK,
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
	RABBITMQ_QUEUE_NAME_PHOTO_CONVERTOR,
	RABBITMQ_USERNAME,
	RABBITMQ_VHOST,
	SHOW_LOGS,
} from '../../config/env';

import { describe, expect, it } from 'vitest';

describe('ENV test runner', () => {
		
	it('Expect all envs to be valid', async () => {
		expect.assertions(21);
		expect(APP_NAME).toBeTruthy();
		expect(LOCATION_LOG_COMBINED).toBeTruthy();
		expect(LOCATION_LOG_ERROR).toBeTruthy();
		expect(LOCATION_LOG_EXCEPTION).toBeTruthy();
		expect(LOCATION_WATERMARK).toBeTruthy();
		expect(LOCATION_PHOTO_STATIC_ORIGINAL).toBeTruthy();
		expect(LOCATION_PHOTO_STATIC_CONVERTED).toBeTruthy();
		expect(MODE_ENV_DEV).toBeFalsy(),
		expect(MODE_ENV_PRODUCTION).toBeFalsy(),
		expect(MODE_ENV_TEST).toBeTruthy(),
		expect(PG_DATABASE).toBeTruthy();
		expect(PG_HOST).toBeTruthy();
		expect(PG_PASS).toBeTruthy();
		expect(PG_USER).toBeTruthy();
		expect(RABBITMQ_HOSTNAME).toBeTruthy();
		expect(RABBITMQ_PASSWORD).toBeTruthy();
		expect(RABBITMQ_QUEUE_NAME_PHOTO_CONVERTOR,).toBeTruthy();
		expect(RABBITMQ_USERNAME,).toBeTruthy();
		expect(RABBITMQ_VHOST).toBeTruthy();
		expect(SHOW_LOGS).toBeTruthy();
		expect(typeof PG_PORT).toBe('number');
	});

});