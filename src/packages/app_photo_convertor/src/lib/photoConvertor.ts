import { customTypeError } from '../config/customError';
import { isString } from '../types/typeGuard';
import { LOCATION_WATERMARK, LOCATION_PHOTO_STATIC_CONVERTED, LOCATION_PHOTO_STATIC_ORIGINAL } from '../config/env';
import { promises as fs } from 'fs';
import { randomBytes } from 'crypto';
import sharp from 'sharp';

class PhotoConvertor {

	async #fileExists (fileName: string): Promise<boolean> {
		try {
			await fs.access(fileName);
			return true;
		} catch (e) {
			return false;
		}
	}

	async #randomHex (num = 32): Promise<string> {
		return new Promise((resolve, reject) => {
			randomBytes(num, (e, buff) => {
				if (e) reject(e);
				resolve(buff.toString('hex').substring(0, num));
			});
		});
	}

	async convert (originalFileName: string) : Promise<string> {
		// Not actually needed, as using parser to verify data
		if (!isString(originalFileName)) throw customTypeError('PhotoConvertor.convert(): !originalFileName');

		// Check original file exists
		const originalFile = `${LOCATION_PHOTO_STATIC_ORIGINAL}/${originalFileName}`;
		const convertedFileExists = await this.#fileExists(originalFile);
		if (!convertedFileExists) throw customTypeError('PhotoConvertor.convert(): !originalFileNameExists');

		// Check watermark exists
		const watermarkFileName = `${LOCATION_WATERMARK}/watermark.png`;
		const watermarkExists = await this.#fileExists(watermarkFileName);
		if (!watermarkExists) throw customTypeError('Watermark not found');

		const convertedFileName = `${originalFileName.substring(0, 12)}_C_${await this.#randomHex(16)}.jpeg`;
	
		const image_sharp = sharp(originalFile);

		await image_sharp.resize({ width: 1000, height: 1000, fit: 'inside' }).composite([ { input: watermarkFileName, gravity: 'southeast' } ]).toFile(`${LOCATION_PHOTO_STATIC_CONVERTED}/${convertedFileName}`);
		return convertedFileName;
	}

}

export const photoConvertor = new PhotoConvertor();