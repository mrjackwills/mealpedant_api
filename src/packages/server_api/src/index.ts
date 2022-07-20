import { api } from './app/api';
import { API_HOSTNAME, API_PORT, API_VERSION_MAJOR, DOMAIN } from './config/env';
import { log } from './config/log';
import { rabbit_ping } from './lib/rabbitRpc';
import { handleProcessExit } from './lib/processExit';
import http from 'http';

const __main__ = async (): Promise<void> => {
	await handleProcessExit();
	await rabbit_ping();
	const server = http.createServer(api);
	server.listen(API_PORT, API_HOSTNAME, () => log.verbose(`API @${API_HOSTNAME}:${API_PORT}/v${API_VERSION_MAJOR}/ server started - domain: ${DOMAIN}`));
};

__main__();