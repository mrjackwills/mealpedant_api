import { GenI, RequestMethod } from 'types';

export const wrap: GenI<RequestMethod> = (fn) => async (req, res, next): Promise<void> => {
	try {
		await fn(req, res, next);
	} catch (e) {
		next(e);
	}
};