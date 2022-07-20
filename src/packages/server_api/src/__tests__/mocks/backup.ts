import { LOCATION_BACKUP } from '../../config/env';
import { randomHex } from '../../lib/helpers';
import { promises as fs } from 'fs';
import { RabbitMessage } from '../../types/enum_rabbitMessage';

const zeroPad = (unit: number): string => String(unit).padStart(2, '0');

const randomDate = (): string => {
	const startDate = new Date(2015, 4, 9);
	const endDate = new Date(2020, 9, 13);
	const data = new Date(startDate.getTime() + Math.random() * (endDate.getTime() - startDate.getTime()));
	return `${data.getFullYear()}-${zeroPad(data.getMonth() + 1)}-${zeroPad(data.getDate())}`;
};

const timeNow = (): string => {
	const now = new Date();
	return `${zeroPad(now.getHours())}.${zeroPad(now.getMinutes())}.${zeroPad(now.getSeconds())}`;
};

export const mock_backup = async (scriptName: RabbitMessage.BACKUP_FULL_BACKUP | RabbitMessage.BACKUP_SQL_BACKUP): Promise<boolean> => {

	const full = scriptName === RabbitMessage.BACKUP_FULL_BACKUP;
	const name = `_LOGS_${full? 'PHOTOS_':''}REDIS_SQL_`;
	const fileName = `mealpedant_${randomDate()}_${timeNow()}${name}${await randomHex(8)}.tar.${full? '' : 'gz.'}gpg`;

	const backupSize = full ? 314572800 : 870400;

	await fs.writeFile(`${LOCATION_BACKUP}/${fileName}`, Buffer.alloc(backupSize));
	return true;
};