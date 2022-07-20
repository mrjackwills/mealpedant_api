import { Send } from '../types';
import { HttpCode } from '../types/enum_httpCode';
import { ResponseMessages } from '../types/enum_response';

export const send: Send = async ({ res, response = ResponseMessages.EMPTY, status = HttpCode.OK }) => {
	res.status(status).json({ response });
};