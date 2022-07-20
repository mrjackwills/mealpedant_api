import { APP_NAME } from './config/env';
import { log } from './config/log';
import { rabbitServer } from './app/api';

const __main__ = async () : Promise<void> => {
	log.debug(`${APP_NAME} started`);
	await rabbitServer.listen();
};

__main__();