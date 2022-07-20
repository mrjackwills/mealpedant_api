import { TestHelper } from '../testHelper';
import { promises as fs } from 'fs';
import { photoConvertor } from '../../lib/photoConvertor';
import { LOCATION_WATERMARK, LOCATION_PHOTO_STATIC_ORIGINAL, LOCATION_PHOTO_STATIC_CONVERTED } from '../../config/env';
import { afterAll, describe, expect, it } from 'vitest';

const testHelper = new TestHelper();

describe('Incognito test runner', () => {

	afterAll(async () => {
		await testHelper.afterAll();
	});

	describe(`PhotoConvertor`, () => {

		it('Throw type error with no filename supplied', async () => {
			expect.assertions(2);
			try {
				// eslint-disable-next-line @typescript-eslint/ban-ts-comment
				// @ts-ignore
				await photoConvertor.convert();
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual('PhotoConvertor.convert(): !originalFileName');
				}
			}
		});

		it('Throw type error with unknown file name supplied', async () => {
			expect.assertions(2);
			try {
				const randomFileName = await testHelper.randomHex();
				await photoConvertor.convert(randomFileName);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual('PhotoConvertor.convert(): !originalFileNameExists');
				}
			}
		});

		it(`Throw error when watermark isn't available`, async () => {
			expect.assertions(2);
			const fileName = await testHelper.randomOriginalFileName();
			try {
				await fs.copyFile(`${LOCATION_WATERMARK}/test.jpg`, `${LOCATION_PHOTO_STATIC_ORIGINAL}/${fileName}`);
				await fs.rename(`${LOCATION_WATERMARK}/watermark.png`, `${LOCATION_WATERMARK}/old_watermark.png`);
				await photoConvertor.convert(fileName);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(Error);
					expect(e.message).toEqual('Watermark not found');
				}
			} finally {
				await fs.rename(`${LOCATION_WATERMARK}/old_watermark.png`, `${LOCATION_WATERMARK}/watermark.png`);
				await fs.rm(`${LOCATION_PHOTO_STATIC_ORIGINAL}/${fileName}`);

			}
		});

		it('convert method to return name of converted file', async () => {
			expect.assertions(1);
			const fileName = await testHelper.randomOriginalFileName();
			try {
				await fs.copyFile(`${LOCATION_WATERMARK}/test.jpg`, `${LOCATION_PHOTO_STATIC_ORIGINAL}/${fileName}`);
				const convertedFileName = await photoConvertor.convert(fileName);
				expect(convertedFileName).toMatch(testHelper.regex_converted);
			} finally {
				await fs.rm(`${LOCATION_PHOTO_STATIC_ORIGINAL}/${fileName}`);
			}
		});

		it('Converted file on disk and correct filesize ', async () => {
			expect.assertions(1);
			const fileName = await testHelper.randomOriginalFileName();
			try {
				await fs.copyFile(`${LOCATION_WATERMARK}/test.jpg`, `${LOCATION_PHOTO_STATIC_ORIGINAL}/${fileName}`);
				const convertedFileName = await photoConvertor.convert(fileName);
				const converted = await fs.stat(`${LOCATION_PHOTO_STATIC_CONVERTED}/${convertedFileName}`);
				expect(converted.size).toEqual(16339);
			} finally {
				await fs.rm(`${LOCATION_PHOTO_STATIC_ORIGINAL}/${fileName}`);
			}
		});

	});
});
