import { promises as fs } from 'fs';
import { backupSpawner } from '../../lib/workerSpawn';
import { LOCATION_BACKUPS } from '../../config/env';

import { TestHelper } from '../testHelper';
import { afterAll, beforeEach, describe, expect, it } from 'vitest';

const testHelper = new TestHelper();

async function deleteAll (): Promise<void> {
	const files = await fs.readdir(`${LOCATION_BACKUPS}`);
	const promiseArray = [];
	for (const file of files) if (file.includes('.tar')) promiseArray.push(fs.unlink(`${LOCATION_BACKUPS}/${file}`));
	await Promise.all(promiseArray);
}

describe('Backupspawner test runner', () => {
	beforeEach(async () => deleteAll());

	afterAll(async () => deleteAll());

	it(`should create a SQL_ONLY backup file`, async () => {
		expect.assertions(5);
		const pre_files = await fs.readdir(`${LOCATION_BACKUPS}`);
		await backupSpawner.create(`SQL_ONLY`);
		const post_files = await fs.readdir(`${LOCATION_BACKUPS}`);
		if (!post_files[0]) throw Error('!post_files[0]');
		const filesize = await fs.stat(`${LOCATION_BACKUPS}/${post_files[0]}`);
		expect(post_files[0].match(testHelper.regex_backupSQLOnly)).toBeTruthy();
		expect(pre_files).toEqual([]);
		expect(post_files.length === 1).toBeTruthy();
		expect(filesize.size).toBeGreaterThan(850570);
		expect(filesize.size).toBeLessThan(1550000);
	});

	it(`should create an FULL backup file`, async () => {
		expect.assertions(4);
		const pre_files = await fs.readdir(`${LOCATION_BACKUPS}`);
		await backupSpawner.create(`FULL`);
		const post_files = await fs.readdir(`${LOCATION_BACKUPS}`);
		if (!post_files[0]) throw Error('!post_files[0]');
		const filesize = await fs.stat(`${LOCATION_BACKUPS}/${post_files[0]}`);
		expect(post_files[0].match(testHelper.regex_backupFull)).toBeTruthy();
		expect(pre_files).toEqual([]);
		expect(post_files.length === 1).toBeTruthy();
		expect(filesize.size).toBeGreaterThan(3000000);

	});

});