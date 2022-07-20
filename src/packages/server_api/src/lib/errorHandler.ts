import { ErrorRequestHandler } from 'express';
import { ErrorMessages } from '../types/enum_error';
import { HttpCode } from '../types/enum_httpCode';
import { log } from '../config/log';
import { MODE_ENV_DEV, MODE_ENV_TEST, SHOW_LOGS } from '../config/env';
import { send } from './send';
import { randomUUID } from 'crypto';

/**
 ** Global error handler
 */
export const errorHandler: ErrorRequestHandler = (e, req, res, next): void => {
	if (e) {
		// eslint-disable-next-line no-console
		if (MODE_ENV_DEV || SHOW_LOGS && !MODE_ENV_TEST) log.debug(e);
	
		if (e instanceof SyntaxError && Object.prototype.hasOwnProperty.call(e, 'body')) {
			send({ res, response: ErrorMessages.MALFORMED_JSON, status: HttpCode.BAD_REQUEST });
		}
		if (e.httpCode) {
			switch (e.httpCode) {
			case HttpCode.UNAUTHORIZED: {
				switch (e.message) {
				case ErrorMessages.PASSPORT_BLOCKED: {
					send({ res, response: ErrorMessages.BLOCKED, status: HttpCode.UNAUTHORIZED });
					break;
				}
				case ErrorMessages.PASSPORT_INVALID_TOKEN:
				case ErrorMessages.PASSPORT_INVALID_BACKUP: {
					const message = req.user ? ErrorMessages.PASSWORD_EMAIL_INVALID_TOKEN : ErrorMessages.PASSWORD_EMAIL_INVALID;
					send({ res, response: message, status: HttpCode.UNAUTHORIZED });
					break;
				}
				case ErrorMessages.PASSPORT_TOKEN_BACKUP: {
					send({ res, response: { twoFABackup: true }, status: HttpCode.ACCEPTED });
					break;
				}
				case ErrorMessages.PASSPORT_TOKEN: {
					send({ res, status: HttpCode.ACCEPTED });
					break;
				}
				case ErrorMessages.TOKEN_INVALID: {
					send({ res, response: ErrorMessages.TOKEN_INVALID, status: HttpCode.UNAUTHORIZED });
					break;
				}
				default: {
					const message = req.user ? ErrorMessages.PASSWORD_EMAIL_INVALID_TOKEN : ErrorMessages.PASSWORD_EMAIL_INVALID;
					send({ res, response: message, status: HttpCode.UNAUTHORIZED });
					break;
				}}
				break;
			}
			case HttpCode.FORBIDDEN: {
				send({ res, response: ErrorMessages.AUTHORIZATION, status: HttpCode.FORBIDDEN });
				break;
			}
			case HttpCode.NOT_FOUND: {
				const message = e.message ?? ErrorMessages.ENDPOINT;
				send({ res, response: message, status: HttpCode.NOT_FOUND });
				break;
			}
			case HttpCode.CONFLICT: {
				const message = e.message ?? ErrorMessages.INVALID_DATA;
				send({ res, response: message, status: HttpCode.CONFLICT });
				break;
			}
			case HttpCode.PAYLOAD_TOO_LARGE: {
				const message = e.message ?? ErrorMessages.PAYLOAD;
				send({ res, response: message, status: HttpCode.PAYLOAD_TOO_LARGE });
				break;
			}
			case HttpCode.TOO_MANY_REQUESTS: {
				const message = e.message ?? ErrorMessages.RATELIMIT;
				send({ res, response: message, status: HttpCode.TOO_MANY_REQUESTS });
				break;
			}
			default: {
				const message = e.message ?? ErrorMessages.INVALID_DATA;
				send({ res, response: message, status: HttpCode.BAD_REQUEST });
				break;
			}
			}
		} else {
			e.uuid = randomUUID({ disableEntropyCache: true });
			const status = e.httpCode ?? HttpCode.INTERNAL_SERVER_ERROR;
			const message = `${ErrorMessages.INTERNAL}: ${e.uuid}`;
			log.error(e);
			send({ res, response: message as ErrorMessages, status });
		}
	} else {
		next();
	}
};