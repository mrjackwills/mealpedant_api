import { rabbitMq } from '../../config/rabbitmq';
import { afterAll, describe, expect, it } from 'vitest';

describe('rabbitmq test suite', () => {

	afterAll(async () => {
		await new Promise((resolve) => setTimeout(() => resolve(true), 200));
		await rabbitMq.closeConnection();
	});

	describe(`Valid rabbitmq connection`, () => {

		it('should return a truthy connection object', async () => {
			expect.assertions(1);
			const connection = await rabbitMq.getConnection();
			expect(connection).toBeTruthy();
		});
	});

});