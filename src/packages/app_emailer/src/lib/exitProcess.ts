import { queries } from './queries';
import { rabbitMq } from '../config/rabbitmq';
import { postgresql } from '../config/db_postgres';

const exit = async (): Promise<void> =>{
	try {
		await queries.insert_appStatus(false);
		await rabbitMq.closeConnection();
		postgresql.end();
	} finally {
		process.exit();
	}
};

export const handleProcessExit = async () : Promise<void> => {
	await queries.insert_appStatus(true);
	process.stdin.resume();
	process.on('exit', async () => exit());
	process.on('SIGINT', () => exit());
	process.on('SIGUSR1', () => exit());
	process.on('SIGUSR2', () => exit());
	process.on('uncaughtException', () => exit());
};
