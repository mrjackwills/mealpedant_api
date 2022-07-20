import { api_version } from '../../config/api_version';
import { promises as fs } from 'fs';
import { cwd } from 'process';
import { TestHelper } from '../testHelper';
const testHelper = new TestHelper();
import { describe, expect, it } from 'vitest';

describe('api_version test runner', () => {
	
	it('Expect api_version to match major.minor.patch', async () => {
		expect.assertions(2);
		expect(api_version).toBeTruthy();
		expect(api_version).toMatch(testHelper.semver_regex);
	});

	it('Matches package.json version', async () => {
		expect.assertions(1);
		const packagejson = await fs.readFile(`${cwd()}/package.json`, 'utf-8');
		const parsed_packagejson = JSON.parse(packagejson);
		expect(parsed_packagejson.version).toEqual(api_version);
	});
});