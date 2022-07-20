import { customTypeError } from '../../config/customError';

class ArgonMock {

	#knownHash = '$argon2id$v=19$m=15360,t=6,p=1$4YLrt7i7kqJKRaMnle7fvDdOc9xkkM04VZAow0QApjo$k0qcO3yUki9sHC00uMzest/yW6f17Fw8ITFwp6R6CwpNggwf44/YykimlhNNlM4R+WOkJsv5S9z2Av+I2rL+RA';
	#knownAttempt = 'argon2 jest test';

	#fakeHash (password: string): string {
		return `$argon2id$v=19$m=15360,t=6,p=1$${password.substring(0, 6)}i7kqJKRaMnle7fvDdOc9xkkM04VZAow0QApjo$k0qcO3yUki9sHC00uMzest/yW6f17Fw8ITFwp6R6CwpNggwf44/YykimlhNNlM4R+WOkJsv5S9z2Av+I2rL+RA`;
	}

	validateHash ({ known_password_hash, attempt }: {known_password_hash: string, attempt: string}): boolean {
		if (!known_password_hash || typeof known_password_hash !== 'string') throw customTypeError('validateHash: !known_password_hash');
		if (!attempt || typeof attempt !== 'string') throw customTypeError('validateHash: !attempt');
		if (attempt === this.#knownAttempt && known_password_hash === this.#knownHash) return true;
		if (attempt !== this.#knownAttempt && known_password_hash === this.#knownHash || attempt === this.#knownAttempt && known_password_hash !== this.#knownHash) return false;
		return known_password_hash === this.#fakeHash(attempt);
	}

	createHash (password: string): string {
		if (process.env.NODE_ENV === 'production') throw Error('invalid node.env for argon mock');
		return password === this.#knownAttempt ? this.#knownHash : this.#fakeHash(password);
	}
}

export const argonMock = new ArgonMock();