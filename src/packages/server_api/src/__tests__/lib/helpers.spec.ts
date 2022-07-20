import { cleanEmail, randomHex, fileExists, extractIp, extractUserAgent } from '../../lib/helpers';
import { LOCATION_CWD } from '../../config/env';
import { describe, expect, it } from 'vitest';
import { mockRequest } from '../testHelper';

describe('Test helpers lib', () => {

	describe(`extractIp function`, () => {

		it.concurrent('should extract x-real-ip', async () => {
			expect.assertions(1);
			const result = extractIp(mockRequest({ headers: { 'x-real-ip': '127.0.0.1' }, connection: { remoteAddress: '192.168.0.1' } }));
			expect(result).toEqual('127.0.0.1');
		});

		it.concurrent('should extract connection.remoteAddress', async () => {
			expect.assertions(1);
			const result = extractIp(mockRequest({ headers: { not: 'relevant' }, connection: { remoteAddress: '192.168.0.1' } }));
			expect(result).toEqual('192.168.0.1');
		});
	});

	describe(`extractUserAgent function`, () => {

		it.concurrent('should extract "jest-userAgent"', async () => {
			expect.assertions(1);
			const result = extractUserAgent(mockRequest({ headers: { 'user-agent': 'jest-userAgent' } }));
			expect(result).toEqual('jest-userAgent');
		});

		it.concurrent('should extract "UNKNOWN"', async () => {
			expect.assertions(1);
			const result = extractUserAgent(mockRequest({ headers: { not: 'relevant' } }));
			expect(result).toEqual('UNKNOWN');
		});
	});

	describe(`cleanEmail function`, () => {

		it.concurrent('should return a lowercase trimmed string', async () => {
			expect.assertions(1);
			const result = cleanEmail('  EMAIL@EXAMPLE.COM  ');
			expect(result).toEqual('email@example.com');
		});
	});

	describe(`randomHex function`, () => {

		it.concurrent(`Should return a random hex string of given length`, async () => {
			expect.assertions(8);
			const randomNumbers = new Array(8);
			for (const [ index, _item ] of randomNumbers.entries()) randomNumbers[index] = Math.floor(Math.random() * 200 - 1) + 1;
			const promiseArray = [];
			for (const i of randomNumbers) promiseArray.push(randomHex(i));
			const results = await Promise.all(promiseArray);
			for (const [ index, item ] of Object.entries(results)) {
				const regexMatcher = RegExp(`^[0-9a-f]{${randomNumbers[Number(index)]}}$`);
				expect(item).toMatch(regexMatcher);
			}
		});
	});

	describe(`fileExists function`, () => {

		it ('should return true for a known file', async () => {
			expect.assertions(1);
			const result = await fileExists(`${LOCATION_CWD}/package.json`);
			expect(result).toBeTruthy();

		});
		
		it ('should return false due to a none existant file', async () => {
			expect.assertions(1);
			const result = await fileExists(`${LOCATION_CWD}/NOT_A_REAL_FILE.json`);
			expect(result).toBeFalsy();
		});
	});
});