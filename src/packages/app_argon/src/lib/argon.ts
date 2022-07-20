import { customTypeError } from '../config/customError';
import { isString } from '../types/typeGuard';
import { TArgonValidate } from '../types';
import * as argon2 from 'argon2';

class Argon {

	#settings = {
		type: argon2.argon2id,
		// default 3
		timeCost: 190,
		// default 4096, 4MiB per thread, 64 ** 2
		memoryCost: 4096,
		// default 1, how many threads to run on
		parallelism: 1,
		// default 16
		saltLength: 32,
		// default 32
		hashLength: 64,
	};

	async validateHash ({ known_password_hash, attempt }: TArgonValidate): Promise<boolean> {
		if (!isString(known_password_hash)) throw customTypeError('validateHash: !known_password_hash');
		if (!isString(attempt)) throw customTypeError('validateHash: !attempt');
		const result = await argon2.verify(known_password_hash, attempt);
		return result;
	}
	
	async createHash (password: string): Promise<string> {
		if (!isString(password)) throw customTypeError('createHash: !password');
		const result = await argon2.hash(password, this.#settings);
		return result;
	}
}

export const argon = new Argon();
