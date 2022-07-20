import { promises as fs } from 'fs';
import { randomBytes } from 'crypto';

export const statModeToString = (x: number): string => `0${(x & parseInt('777', 8)).toString(8)}`;

export const fileMode = async (filePath: string): Promise<string> => {
	const file = await fs.stat(filePath);
	return statModeToString(file.mode);
};

export const fileExists = async (fileName: string): Promise<boolean> => {
	try {
		await fs.access(fileName);
		return true;
	} catch (e) {
		return false;
	}
};

export const randomHex = async (num = 32): Promise<string> => new Promise((resolve, reject) => {
	randomBytes(num, (e, buff) => {
		if (e) reject(e);
		resolve(buff.toString('hex').substring(0, num));
	});
});