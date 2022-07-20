import { AdminRoutes } from './rest_admin';
import { API_VERSION_MAJOR } from '../config/env';
import { AuthenticatedRoutes } from './rest_authenticated';
import { customError } from './customError';
import { ErrorMessages } from '../types/enum_error';
import { HttpCode } from '../types/enum_httpCode';
import { incognito } from '../components/incognito/incognito_router';
import { rateLimiter } from '../config/rateLimiter';
import { RequestHandler, Router } from 'express';
import { RoutesRoot } from '../types/enum_routes';

const rest = Router({ mergeParams: true });

// maybe throw custom error with status 404?
const notFound: RequestHandler = (_req, _res) => {
	throw customError(HttpCode.NOT_FOUND, ErrorMessages.ENDPOINT);
};

const prefixVersion = (route: RoutesRoot) : string => `/v${API_VERSION_MAJOR}${route}`;

// Apply rate limiting to all routes
rest.use(rateLimiter);
rest.use(`${prefixVersion(RoutesRoot.INCOGNITO)}`, incognito);
rest.use(`${prefixVersion(RoutesRoot.AUTHENTICATED)}`, AuthenticatedRoutes);
rest.use(`${prefixVersion(RoutesRoot.ADMIN)}`, AdminRoutes);
rest.all(RoutesRoot.CATCH_ALL, notFound);

export { rest };