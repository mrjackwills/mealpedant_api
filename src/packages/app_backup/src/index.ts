import { APP_NAME } from './config/env';
import { log } from './config/log';
import { rabbitServer } from './app/api';
import { cronScheduler } from './lib/cronScheduler';

const __main__ = async () : Promise<void> => {
	log.debug(`${APP_NAME} started`);
	cronScheduler.init();
	await rabbitServer.listen();
};

__main__();