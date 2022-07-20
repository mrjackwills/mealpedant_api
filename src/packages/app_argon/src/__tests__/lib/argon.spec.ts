import { argon } from '../../lib/argon';
import { TestHelper } from '../testHelper';
import { describe, expect, it } from 'vitest';
const testHelper = new TestHelper();

describe('Test argon lib', () => {

	describe(`hashPassword function`, () => {
	
		it('return a valid argon2 string', async () => {
			expect.assertions(2);
			const hash = await argon.createHash(testHelper.password);
			expect(hash).toBeTruthy();
			expect(hash).toMatch(testHelper.regex_argon);
		});

		it('return a two valid argon2 strings from the same password', async () => {
			const hash_one = await argon.createHash(testHelper.password);
			const hash_two = await argon.createHash(testHelper.password);
			expect(hash_one).toBeTruthy();
			expect(hash_one).toStrictEqual(expect.stringMatching(testHelper.regex_argon));
			expect(hash_two).toBeTruthy();
			expect(hash_two).toStrictEqual(expect.stringMatching(testHelper.regex_argon));
			expect(hash_one).not.toEqual(hash_two);
		});
		
		it('hashPassword to throw error when no password provided', async () => {
			expect.assertions(2);
			try {
				await argon.createHash('');
			} catch (e) {
				if (e instanceof TypeError) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toStrictEqual('createHash: !password');
				}
			}
		});

		it('hashPassword to throw error when password is a number', async () => {
			expect.assertions(2);
			try {
				// eslint-disable-next-line @typescript-eslint/ban-ts-comment
				// @ts-ignore
				await argon.createHash(testHelper.randomNumber());
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toStrictEqual('createHash: !password');
				}
			}
		});

		it('hashPassword to throw error when password is a boolean', async () => {
			expect.assertions(2);
			try {
				// eslint-disable-next-line @typescript-eslint/ban-ts-comment
				// @ts-ignore
				await argon.createHash(testHelper.randomBoolean);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toStrictEqual('createHash: !password');
				}
			}
		});
	});

	describe(`verifyPassword function`, () => {
		
		it('verify password as true', async () => {
			expect.assertions(1);
			const result = await argon.validateHash({ known_password_hash: testHelper.known_hash, attempt: testHelper.password });
			expect(result).toBeTruthy();
		});
		
		it('verify two unique hash strings, with same password, as true', async () => {
			expect.assertions(3);
			const hash_one = await argon.createHash(testHelper.password);
			const hash_two = await argon.createHash(testHelper.password);
			const result01 = await argon.validateHash({ known_password_hash: hash_one, attempt: testHelper.password });
			const result02 = await argon.validateHash({ known_password_hash: hash_two, attempt: testHelper.password });
			expect(hash_one).not.toEqual(hash_two);
			expect(result01).toBeTruthy();
			expect(result02).toBeTruthy();
		});
	
		it('verify password as false when extra char added', async () => {
			expect.assertions(1);
			const randomChar = await testHelper.randomHex(1);
			const result = await argon.validateHash({ known_password_hash: testHelper.known_hash, attempt: `${testHelper.known_hash}${randomChar}` });
			expect(result).toBeFalsy();
		});
		it('verify password as false when password random hex', async () => {
			expect.assertions(1);
			const attempt = await testHelper.randomHex();
			const result = await argon.validateHash({ known_password_hash: testHelper.known_hash, attempt });
			expect(result).toBeFalsy();
		});

		it('verifyPassword to throw error when no params', async () => {
			expect.assertions(2);
			try {
				// eslint-disable-next-line @typescript-eslint/ban-ts-comment
				// @ts-ignore
				await argon.validateHash({});
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toStrictEqual('validateHash: !known_password_hash');
				}
			}
		});

		it('verifyPassword to throw error when no password_hash', async () => {
			expect.assertions(2);
			try {
				// eslint-disable-next-line @typescript-eslint/ban-ts-comment
				// @ts-ignore
				await argon.validateHash({ attempt: testHelper.password });
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toStrictEqual('validateHash: !known_password_hash');
				}
			}
		});
		
		it('verifyPassword to throw error when password_hash empty string', async () => {
			expect.assertions(2);
			try {
				await argon.validateHash({ known_password_hash: '', attempt: testHelper.password });
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toStrictEqual('validateHash: !known_password_hash');
				}
			}
		});

		it('verifyPassword to throw error when password_hash is a number', async () => {
			expect.assertions(2);
			try {
				const randomNumber = testHelper.randomNumber();
				// eslint-disable-next-line @typescript-eslint/ban-ts-comment
				// @ts-ignore
				await argon.validateHash({ known_password_hash: randomNumber, attempt: testHelper.password });
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toStrictEqual('validateHash: !known_password_hash');
				}
			}
		});

		it('verifyPassword to throw error when password_hash is a boolean', async () => {
			expect.assertions(2);
			try {
				const randomBoolean = testHelper.randomBoolean;
				// eslint-disable-next-line @typescript-eslint/ban-ts-comment
				// @ts-ignore
				await argon.validateHash({ known_password_hash: randomBoolean, attempt: testHelper.password });
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toStrictEqual('validateHash: !known_password_hash');
				}
			}
		});

		it('verifyPassword to throw error when no attempt', async () => {
			expect.assertions(2);
			try {
				// eslint-disable-next-line @typescript-eslint/ban-ts-comment
				// @ts-ignore
				await argon.validateHash({ known_password_hash: testHelper.password });
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toStrictEqual('validateHash: !attempt');
				}
			}
		});
	
		it('verifyPassword to throw error when attempt empty string', async () => {
			expect.assertions(2);
			try {
				await argon.validateHash({ known_password_hash: testHelper.known_hash, attempt: '' });
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toStrictEqual('validateHash: !attempt');
				}
			}
		});
		
		it('verifyPassword to throw error when attempt is a number', async () => {
			expect.assertions(2);
			try {
				const randomNumber = testHelper.randomNumber();
				// eslint-disable-next-line @typescript-eslint/ban-ts-comment
				// @ts-ignore
				await argon.validateHash({ known_password_hash: testHelper.password, attempt: randomNumber });
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toStrictEqual('validateHash: !attempt');
				}
			}
		});

		it('verifyPassword to throw error when attempt is a boolean', async () => {
			expect.assertions(2);
			try {
				const randomBoolean = testHelper.randomBoolean;
				// eslint-disable-next-line @typescript-eslint/ban-ts-comment
				// @ts-ignore
				await argon.validateHash({ known_password_hash: testHelper.password, attempt: randomBoolean });
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toStrictEqual('validateHash: !attempt');
				}
			}
		});

	});
});