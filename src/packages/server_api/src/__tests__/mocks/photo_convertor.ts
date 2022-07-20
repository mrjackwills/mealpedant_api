import { LOCATION_PHOTO_CONVERTED, LOCATION_PHOTO_ORIGINAL } from '../../config/env';
import { randomHex } from '../../lib/helpers';
import { promises as fs } from 'fs';

const fileExists = async (fileName: string): Promise<boolean> => {
	try {
		await fs.access(fileName);
		return true;
	} catch (e) {
		return false;
	}
};

export const mock_photoConvertor = async (originalFileName: string): Promise<string> => {
	const originalFile = `${LOCATION_PHOTO_ORIGINAL}/${originalFileName}`;
	const convertedFileExists = await fileExists(originalFile);
	if (!convertedFileExists) throw Error('PhotoConvertor.convert(): !originalFileNameExists');
	const convertedFileName = `${originalFileName.substring(0, 12)}_C_${await randomHex(16)}.jpeg`;
	const fileName = `/${LOCATION_PHOTO_CONVERTED}/${convertedFileName}`;
	const emptyFile = Buffer.alloc(16339);
	await fs.writeFile(fileName, emptyFile);
	return convertedFileName;
};