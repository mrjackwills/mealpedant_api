import { ErrorMessages } from '../types/enum_error';
import { HttpCode } from '../types/enum_httpCode';
import { UserId } from '../types/index';

class ErrorWithStatus extends Error {
	constructor (public httpCode: HttpCode, public message: string, public userId: UserId|undefined) {
		super();
	}
}

class TypeErrorWithStatus extends TypeError {
	constructor (public httpCode: HttpCode, public message: string) {
		super();
	}
}

// TODO add user id?
export const customError = (httpCode?: HttpCode, message?: ErrorMessages | number, userId? : UserId): ErrorWithStatus => {
	const errorCode = httpCode ?? HttpCode.INTERNAL_SERVER_ERROR;
	const errorMessage = message ?? ErrorMessages.INTERNAL;
	const errorToThrow = new ErrorWithStatus(errorCode, String(errorMessage), userId);
	return errorToThrow;
};

export const customTypeError = (message: string, httpCode?: HttpCode): TypeErrorWithStatus => {
	const errorCode = httpCode ?? HttpCode.INTERNAL_SERVER_ERROR;
	const errorMessage = message ?? ErrorMessages.TYPE;
	const errorToThrow = new TypeErrorWithStatus(errorCode, errorMessage);
	return errorToThrow;
};