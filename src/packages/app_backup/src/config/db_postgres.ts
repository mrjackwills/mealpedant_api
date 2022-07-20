import { PG_DATABASE, PG_HOST, PG_PASS, PG_PORT, PG_USER } from './env';
import { Pool } from 'pg';

export const postgresql = new Pool({
	user: PG_USER,
	host: PG_HOST,
	database: PG_DATABASE,
	password: PG_PASS,
	port: PG_PORT,
	max: 20,
	idleTimeoutMillis: 30000,
	connectionTimeoutMillis: 2000,
});