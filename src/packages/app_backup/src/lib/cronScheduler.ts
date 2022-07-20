import { schedule } from 'node-cron';
import { backupSpawner } from './workerSpawn';

class CronScheduler {
	
	async init () :Promise<void> {
		schedule('0 3 * * *', () => backupSpawner.create('FULL'));
		schedule('1 3 * * *', () => backupSpawner.create('SQL_ONLY'));
	}
	
}

export const cronScheduler = new CronScheduler();
