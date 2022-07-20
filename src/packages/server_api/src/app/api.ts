import { COOKIE_NAME, COOKIE_SECRET, DOMAIN, MODE_ENV_PRODUCTION, MODE_ENV_TEST, MODE_ENV_DEV, SHOW_LOGS } from '../config/env';
import { corsAsync } from '../config/cors';
import { errorHandler } from '../lib/errorHandler';
import { passport } from '../config/passport';
import { randomUUID } from 'crypto';
import { Redis } from '../config/db_redis';
import { RedisKey } from '../types/enum_redis';
import { rest } from '../config/rest';
import { RoutesRoot } from '../types/enum_routes';
import connectRedis from 'connect-redis';
import cookieParser from 'cookie-parser';
import cors from 'cors';
import express, { CookieOptions } from 'express';
import morgan from 'morgan';
import session, { SessionOptions } from 'express-session';

const RedisStore = connectRedis(session);
export const api = express();

const cookie: CookieOptions = {
	domain: DOMAIN,
	httpOnly: MODE_ENV_PRODUCTION,
	sameSite: MODE_ENV_PRODUCTION,
	secure: MODE_ENV_PRODUCTION,
	maxAge: 1000 * 60 * 60 * 24,
};

const sessionOptions: SessionOptions = {
	secret: String(COOKIE_SECRET),
	name: String(COOKIE_NAME),
	cookie,
	genid: (_req) => randomUUID({ disableEntropyCache: true }),
	resave: false,
	saveUninitialized: false,
	store: new RedisStore({
		client: Redis,
		disableTouch: true,
		prefix: RedisKey.SESSION
	})
};

export const parse_session = session(sessionOptions);

if (MODE_ENV_DEV || SHOW_LOGS && !MODE_ENV_TEST) api.use(morgan('dev'));
api.enable('trust proxy');

api.use(cors(corsAsync));
api.use(cookieParser());
api.use(express.json());
api.use(express.urlencoded({
	extended: true
}));
api.use(parse_session);
api.use(passport.initialize());
api.use(passport.session());
api.use(RoutesRoot.BASE, rest);
api.use(errorHandler);