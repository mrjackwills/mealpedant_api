import { validate_input } from '../../lib/validateinput';
import { schema_photo } from '../../components/photo/photo_schema';
import { TestHelper } from '../testHelper';

import { afterAll, describe, expect, it } from 'vitest';

const testHelper = new TestHelper();

describe('schema_photoDelete tests', () => {

	afterAll(() => testHelper.afterAll());

	describe(`schema_photoDelete`, () => {

		it('should resolve when filename date, person, type, hex, jpeg valid', async () => {
			const b16 = await testHelper.randomHex(16);
			expect.assertions(1);
			const nameC = `2020-01-01_D_C_${b16}.jpeg`;
			const nameO = `2020-01-01_D_O_${b16}.jpeg`;
			let result = false;
			try {
				validate_input({ o: nameO, c: nameC }, schema_photo.delete);
			} catch (e) {
				result = true;
			}
			expect(result).toBeFalsy();
		});

		it('should throw error when o not present', async () => {
			expect.assertions(2);
			const nameC = `2020-01-01_D_C_${ await testHelper.randomHex(16)}.jpeg`;
			try {
				validate_input({ c: nameC }, schema_photo.delete);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_filename);
				}
			}
		});

		it('should throw error when c not present', async () => {
			expect.assertions(2);
			const nameO = `2020-01-01_D_O_${ await testHelper.randomHex(16)}.jpeg`;
			try {
				validate_input({ o: nameO }, schema_photo.delete);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_filename);
				}
			}
		});

		it('should throw error when o invalid', async () => {
			expect.assertions(2);
			const nameO = `2020-01-01_D_O_${ await testHelper.randomHex(16)}.jpeg`;
			try {
				validate_input({ o: nameO, c: nameO }, schema_photo.delete);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_filename);
				}
			}
		});

		it('should throw error when c invalid', async () => {
			expect.assertions(2);
			const nameO = `2020-01-01_D_O_${ await testHelper.randomHex(16)}.jpeg`;
			try {
				validate_input({ o: nameO, c: nameO }, schema_photo.delete);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_filename);
				}
			}
		});

	});
});