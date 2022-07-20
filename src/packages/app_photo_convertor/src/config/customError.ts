import { ErrorMessages } from '../types/enum_error';

class ErrorWithStatus extends Error {
	constructor (public message: string) {
		super();
	}
}

class TypeErrorWithStatus extends TypeError {
	constructor (public message: string) {
		super();
	}
}

export const customError = (message?: ErrorMessages | number): ErrorWithStatus => {
	const errorMessage = message ?? ErrorMessages.INTERNAL;
	return new ErrorWithStatus(String(errorMessage));
};

export const customTypeError = (message: string): TypeErrorWithStatus => {
	const errorMessage = message ?? ErrorMessages.TYPE;
	return new TypeErrorWithStatus(errorMessage);
};