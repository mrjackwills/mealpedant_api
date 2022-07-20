/* eslint-disable @typescript-eslint/no-empty-interface */
import { TPassportDeserializedUser } from '../types';

declare global {
	namespace Express {
		interface User extends TPassportDeserializedUser {	}
	}
}