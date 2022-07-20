import { api_version } from './api_version';
import { config } from 'dotenv';
import { resolve } from 'path';
config({ path: resolve(__dirname, '../../.env.local') });

const cwd = process.cwd();
const major = api_version.split('.')[0];

const env = process.env;
if (!env.API_HOSTNAME) throw new Error('!env.API_HOSTNAME');
if (!env.API_PORT) throw new Error('!env.API_PORT');
if (isNaN(Number(env.API_PORT))) throw new Error('isNaN(env.API_PORT)');
if (isNaN(Number(major))) throw new Error('!env major');

if (!env.COOKIE_NAME) throw new Error('!env.COOKIE_NAME');
if (!env.COOKIE_SECRET) throw new Error('!env.COOKIE_SECRET');
if (!env.DOMAIN) throw new Error('!env.DOMAIN');

if (!env.INVITE_USER) throw new Error('!env.INVITE_USER');

if (!env.LOCATION_BACKUP) throw new Error('!env.LOCATION_BACKUP');
if (!env.LOCATION_LOG_COMBINED) throw new Error('!env.LOCATION_LOG_COMBINED');
if (!env.LOCATION_LOG_ERROR) throw new Error('!env.LOCATION_LOG_ERROR');
if (!env.LOCATION_LOG_EXCEPTION) throw new Error('!env.LOCATION_LOG_EXCEPTION');
if (!env.LOCATION_PHOTO_CONVERTED) throw new Error('!env.LOCATION_PHOTO_CONVERTED');
if (!env.LOCATION_PHOTO_ORIGINAL) throw new Error('!env.LOCATION_PHOTO_ORIGINAL');
if (!env.LOCATION_TEMP) throw new Error('!env.LOCATION_TEMP');

if (!env.PG_DATABASE) throw new Error('!env.PG_DATABASE');
if (!env.PG_HOST) throw new Error('!env.PG_HOST');
if (!env.PG_PASS) throw new Error('!env.PG_PASS');
if (!env.PG_PORT) throw new Error('!env.PG_PORT');
if (isNaN(Number(env.PG_PORT))) throw new Error('isNaN(env.PG_PORT)');
if (!env.PG_USER) throw new Error('!env.PG_USER');

if (!env.RABBITMQ_HOSTNAME) throw Error('!env.RABBITMQ_HOSTNAME');
if (!env.RABBITMQ_PORT) throw Error('!env.RABBITMQ_PORT');
if (!env.RABBITMQ_USERNAME) throw Error('!env.RABBITMQ_USERNAME');
if (!env.RABBITMQ_PASSWORD) throw Error('!env.RABBITMQ_PASSWORD');
if (!env.RABBITMQ_VHOST) throw Error('!env.RABBITMQ_VHOST');
if (!env.RABBITMQ_QUEUE_NAME_ARGON) throw Error('!env.RABBITMQ_QUEUE_NAME_ARGON');
if (!env.RABBITMQ_QUEUE_NAME_EMAILER) throw Error('!env.RABBITMQ_QUEUE_NAME_EMAILER');
if (!env.RABBITMQ_QUEUE_NAME_PHOTO_CONVERTOR) throw Error('!env.RABBITMQ_QUEUE_NAME_PHOTO_CONVERTOR');
if (!env.RABBITMQ_QUEUE_NAME_BACKUP) throw Error('!env.RABBITMQ_QUEUE_NAME_BACKUP');

if (!env.REDIS_DB) throw new Error('!env.REDIS_DB');
if (!env.REDIS_HOSTNAME) throw new Error('!env.REDIS_HOSTNAME');
if (!env.REDIS_PASS) throw new Error('!env.REDIS_PASS');
if (!env.REDIS_PORT) throw new Error('!env.REDIS_PORT');
if (isNaN(Number(env.REDIS_DB))) throw new Error('isNaN(env.REDIS_DB)');
if (isNaN(Number(env.REDIS_PORT))) throw new Error('isNaN(env.REDIS_PORT)');

if (!env.VUE_APP_COOKIE_KEY) throw new Error('!env.VUE_APP_COOKIE_KEY');

// eslint-disable-next-line no-console
if (env.LOCAL_VUE_CONNECT) for (const _i of new Array(10)) console.log('WARNING - CONNECT FROM LOCAL ENABLED\n\n');

export const API_HOSTNAME = env.API_HOSTNAME;
export const API_PORT = Number(env.API_PORT);
export const API_VERSION_MAJOR = Number(major);

export const COOKIE_NAME = env.COOKIE_NAME;
export const COOKIE_SECRET = env.COOKIE_SECRET;
export const DOMAIN = env.DOMAIN;

export const INVITE_USER = env.INVITE_USER;
export const JEST_USER_EMAIL = String(env.JEST_USER_EMAIL);
export const JEST_USER_PASSWORD = String(env.JEST_USER_PASSWORD);

export const LOCATION_BACKUP = env.LOCATION_BACKUP;
export const LOCATION_CWD = cwd;
export const LOCATION_LOG_COMBINED = env.LOCATION_LOG_COMBINED;
export const LOCATION_LOG_ERROR = env.LOCATION_LOG_ERROR;
export const LOCATION_LOG_EXCEPTION = env.LOCATION_LOG_EXCEPTION;
export const LOCATION_PHOTO_CONVERTED = env.LOCATION_PHOTO_CONVERTED;
export const LOCATION_PHOTO_ORIGINAL = env.LOCATION_PHOTO_ORIGINAL;
export const LOCATION_TEMP = env.LOCATION_TEMP;

export const MODE_ENV_DEV = env.NODE_ENV === 'development';
export const MODE_ENV_PRODUCTION = env.NODE_ENV === 'production';
export const MODE_ENV_TEST = env.NODE_ENV === 'test';

export const PG_DATABASE = env.PG_DATABASE;
export const PG_HOST = env.PG_HOST;
export const PG_PASS = env.PG_PASS;
export const PG_PORT = Number(env.PG_PORT);
export const PG_USER = env.PG_USER;

export const RABBITMQ_HOSTNAME = env.RABBITMQ_HOSTNAME;
export const RABBITMQ_PORT = Number(env.RABBITMQ_PORT);
export const RABBITMQ_USERNAME = env.RABBITMQ_USERNAME;
export const RABBITMQ_PASSWORD = env.RABBITMQ_PASSWORD;
export const RABBITMQ_VHOST	= env.RABBITMQ_VHOST;
export const RABBITMQ_QUEUE_NAME_ARGON = env.RABBITMQ_QUEUE_NAME_ARGON;
export const RABBITMQ_QUEUE_NAME_EMAILER = env.RABBITMQ_QUEUE_NAME_EMAILER;
export const RABBITMQ_QUEUE_NAME_PHOTO_CONVERTOR = env.RABBITMQ_QUEUE_NAME_PHOTO_CONVERTOR;
export const RABBITMQ_QUEUE_NAME_BACKUP = env.RABBITMQ_QUEUE_NAME_BACKUP;

export const REDIS_DB = Number(env.REDIS_DB);
export const REDIS_HOSTNAME = env.REDIS_HOSTNAME;
export const REDIS_PASS = env.REDIS_PASS;
export const REDIS_PORT = Number(env.REDIS_PORT);

export const VUE_APP_COOKIE_KEY = env.VUE_APP_COOKIE_KEY;

export const SHOW_LOGS = env.SHOW_LOGS;
export const LOCAL_VUE_CONNECT = env.LOCAL_VUE_CONNECT;