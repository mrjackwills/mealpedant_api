import { CorsOptions, CorsOptionsDelegate } from 'cors';
import { DOMAIN, MODE_ENV_DEV, MODE_ENV_TEST, LOCAL_VUE_CONNECT } from './env';
import { PV } from '../types';

const origin = `https://www${DOMAIN}`;

export const corsAsync: CorsOptionsDelegate = async (req, next): PV => {
	const corsOptions: CorsOptions = {
		credentials: true,
		origin: origin === req.headers.origin || LOCAL_VUE_CONNECT || MODE_ENV_DEV || MODE_ENV_TEST ? true : false,
	};
	next(null, corsOptions);
};
