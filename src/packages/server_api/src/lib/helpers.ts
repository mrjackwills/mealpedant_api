import { promises as fs } from 'fs';
import { randomBytes } from 'crypto';
import { GenI, ReqString, FileExists, RandomHex } from '../types';

export const cleanEmail: GenI<string> = (email) => email.toLowerCase().trim();

export const extractIp: ReqString = (req) => {
	const ip = req.headers['x-real-ip'] || req.socket?.remoteAddress || req.connection?.remoteAddress;
	return String(ip);

};

export const extractUserAgent: ReqString = (req) => {
	const userAgent = req.headers['user-agent'];
	return userAgent ? String(userAgent) : 'UNKNOWN';
};

export const statModeToString = (x: number): string => `0${(x & parseInt('777', 8)).toString(8)}`;

export const fileMode = async (filePath: string): Promise<string> => {
	const file = await fs.stat(filePath);
	return statModeToString(file.mode);
};

export const fileExists: FileExists = async (fileName) => {
	try {
		await fs.access(fileName);
		return true;
	} catch (e) {
		return false;
	}
};

export const randomHex: RandomHex = async (num = 32) => new Promise((resolve, reject) => {
	randomBytes(num, (e, buff) => {
		if (e) reject(e);
		resolve(buff.toString('hex').substring(0, num));
	});
});