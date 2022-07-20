import { validate_input } from '../../lib/validateinput';
import { schema_meal } from '../../components/meal/meal_schema';
import { TestHelper } from '../testHelper';
import { afterAll, describe, expect, it } from 'vitest';

const testHelper = new TestHelper();

describe('meal_joi tests', () => {

	afterAll(() => testHelper.afterAll());

	describe(`schema_meal.addMeal`, () => {

		it('should resolve when basic meal object valid', async () => {
			expect.assertions(1);
			let result = false;
			try {
				validate_input({ meal: await testHelper.createMeal() }, schema_meal.addMeal);
			} catch (e) {
				result = true;
			}
			expect(result).toBeFalsy();
		});

		it('should resolve when valid photo names provided', async () => {
			expect.assertions(1);
			let result = false;
			try {
				validate_input({ meal: await testHelper.createMeal(true) }, schema_meal.addMeal);
			} catch (e) {
				result = true;
			}
			expect(result).toBeFalsy();
		});
	
		it('should throw error when date not present', async () => {
			expect.assertions(2);
			try {
				const meal = await testHelper.createMeal(true);
				// eslint-disable-next-line @typescript-eslint/ban-ts-comment
				// @ts-ignore
				delete meal.date;
				validate_input({ meal: meal }, schema_meal.addMeal);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_date);
				}
			}
		});

		it('should throw error when date not valid: boolean', async () => {
			expect.assertions(2);
			try {
				const meal = await testHelper.createMeal(true);
				// eslint-disable-next-line @typescript-eslint/ban-ts-comment
				// @ts-ignore
				meal.date = true;
				validate_input({ meal: meal }, schema_meal.addMeal);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_date);
				}
			}
		});

		it('should throw error when date not valid: missing day', async () => {
			expect.assertions(2);
			try {
				const meal = await testHelper.createMeal(true);
				// eslint-disable-next-line @typescript-eslint/ban-ts-comment
				// @ts-ignore
				meal.date = testHelper.randomDate().substring(0, 7);
				validate_input({ meal: meal }, schema_meal.addMeal);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_date);
				}
			}
		});
			
		it('should throw error when date not valid: missing hypens', async () => {
			expect.assertions(2);
			try {
				const meal = await testHelper.createMeal(true);
				// eslint-disable-next-line @typescript-eslint/ban-ts-comment
				// @ts-ignore
				meal.date = testHelper.randomDate().replace('-', '');
				validate_input({ meal: meal }, schema_meal.addMeal);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_date);
				}
			}
		});

		it('should throw error when person missing', async () => {
			expect.assertions(2);
			try {
				const meal = await testHelper.createMeal(true);
				// eslint-disable-next-line @typescript-eslint/ban-ts-comment
				// @ts-ignore
				delete meal.person;
				validate_input({ meal: meal }, schema_meal.addMeal);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_person);
				}
			}
		});

		it('should throw error when person incorrect: wrong name', async () => {
			expect.assertions(2);
			try {
				const meal = await testHelper.createMeal(true);
				// eslint-disable-next-line @typescript-eslint/ban-ts-comment
				// @ts-ignore
				meal.person = await testHelper.randomHex(8);
				validate_input({ meal: meal }, schema_meal.addMeal);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_person);
				}
			}
		});

		it('should throw error when person incorrect: boolean', async () => {
			expect.assertions(2);
			try {
				const meal = await testHelper.createMeal(true);
				// eslint-disable-next-line @typescript-eslint/ban-ts-comment
				// @ts-ignore
				meal.person = testHelper.randomBoolean();
				validate_input({ meal: meal }, schema_meal.addMeal);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_person);
				}
			}
		});

		it('should throw error when person incorrect: lowercase', async () => {
			expect.assertions(2);
			try {
				const meal = await testHelper.createMeal(true);
				// eslint-disable-next-line @typescript-eslint/ban-ts-comment
				// @ts-ignore
				meal.person = meal.person.toLowerCase();
				validate_input({ meal: meal }, schema_meal.addMeal);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_person);
				}
			}
		});

		it('should throw error when description not presented', async () => {
			expect.assertions(2);
			try {
				const meal = await testHelper.createMeal(true);
				// eslint-disable-next-line @typescript-eslint/ban-ts-comment
				// @ts-ignore
				delete meal.description;
				validate_input({ meal: meal }, schema_meal.addMeal);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_description);
				}
			}
		});

		it('should throw error when description invalid: empty string', async () => {
			expect.assertions(2);
			try {
				const meal = await testHelper.createMeal(true);
				// eslint-disable-next-line @typescript-eslint/ban-ts-comment
				// @ts-ignore
				meal.description = '';
				validate_input({ meal: meal }, schema_meal.addMeal);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_description);
				}
			}
		});

		it('should throw error when description invalid: boolean', async () => {
			expect.assertions(2);
			try {
				const meal = await testHelper.createMeal(true);
				// eslint-disable-next-line @typescript-eslint/ban-ts-comment
				// @ts-ignore
				meal.description = testHelper.randomBoolean();
				validate_input({ meal: meal }, schema_meal.addMeal);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_description);
				}
			}
		});

		it('should throw error when description invalid: number', async () => {
			expect.assertions(2);
			try {
				const meal = await testHelper.createMeal(true);
				// eslint-disable-next-line @typescript-eslint/ban-ts-comment
				// @ts-ignore
				meal.description = testHelper.randomNumber();
				validate_input({ meal: meal }, schema_meal.addMeal);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_description);
				}
			}
		});
		
		it('should throw error when category not present', async () => {
			expect.assertions(2);
			try {
				const meal = await testHelper.createMeal(true);
				// eslint-disable-next-line @typescript-eslint/ban-ts-comment
				// @ts-ignore
				delete meal.category;
				validate_input({ meal: meal }, schema_meal.addMeal);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_category);
				}
			}
		});

		it('should throw error when category invalid: empty string', async () => {
			expect.assertions(2);
			try {
				const meal = await testHelper.createMeal(true);
				// eslint-disable-next-line @typescript-eslint/ban-ts-comment
				// @ts-ignore
				meal.category = '';
				validate_input({ meal: meal }, schema_meal.addMeal);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_category);
				}
			}
		});

		it('should throw error when category invalid: boolean', async () => {
			expect.assertions(2);
			try {
				const meal = await testHelper.createMeal(true);
				// eslint-disable-next-line @typescript-eslint/ban-ts-comment
				// @ts-ignore
				meal.category = testHelper.randomBoolean();
				validate_input({ meal: meal }, schema_meal.addMeal);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_category);
				}
			}
		});

		it('should throw error when category invalid: number', async () => {
			expect.assertions(2);
			try {
				const meal = await testHelper.createMeal(true);
				// eslint-disable-next-line @typescript-eslint/ban-ts-comment
				// @ts-ignore
				meal.category = testHelper.randomNumber();
				validate_input({ meal: meal }, schema_meal.addMeal);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_category);
				}
			}
		});
			
		it('should throw error when restaurant invalid: empty string', async () => {
			expect.assertions(2);
			try {
				const meal = await testHelper.createMeal(true);
				// eslint-disable-next-line @typescript-eslint/ban-ts-comment
				// @ts-ignore
				meal.restaurant = '';
				validate_input({ meal: meal }, schema_meal.addMeal);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_restaurant);
				}
			}
		});

		it('should throw error when restaurant invalid: string', async () => {
			expect.assertions(2);
			try {
				const meal = await testHelper.createMeal(true);
				// eslint-disable-next-line @typescript-eslint/ban-ts-comment
				// @ts-ignore
				meal.restaurant = await testHelper.randomHex(8);
				validate_input({ meal: meal }, schema_meal.addMeal);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_restaurant);
				}
			}
		});

		it('should throw error when restaurant invalid: number', async () => {
			expect.assertions(2);
			try {
				const meal = await testHelper.createMeal(true);
				// eslint-disable-next-line @typescript-eslint/ban-ts-comment
				// @ts-ignore
				meal.restaurant = testHelper.randomNumber();
				validate_input({ meal: meal }, schema_meal.addMeal);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_restaurant);
				}
			}
		});

		it('should throw error when takeaway invalid: empty string', async () => {
			expect.assertions(2);
			try {
				const meal = await testHelper.createMeal(true);
				// eslint-disable-next-line @typescript-eslint/ban-ts-comment
				// @ts-ignore
				meal.takeaway = '';
				validate_input({ meal: meal }, schema_meal.addMeal);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_takeaway);
				}
			}
		});

		it('should throw error when takeaway invalid: string', async () => {
			expect.assertions(2);
			try {
				const meal = await testHelper.createMeal(true);
				// eslint-disable-next-line @typescript-eslint/ban-ts-comment
				// @ts-ignore
				meal.takeaway = await testHelper.randomHex(4);
				validate_input({ meal: meal }, schema_meal.addMeal);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_takeaway);
				}
			}
		});

		it('should throw error when takeaway invalid: number', async () => {
			expect.assertions(2);
			try {
				const meal = await testHelper.createMeal(true);
				// eslint-disable-next-line @typescript-eslint/ban-ts-comment
				// @ts-ignore
				meal.takeaway = testHelper.randomNumber();
				validate_input({ meal: meal }, schema_meal.addMeal);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_takeaway);
				}
			}
		});

		it('should throw error when vegetarian invalid: empty string', async () => {
			expect.assertions(2);
			try {
				const meal = await testHelper.createMeal(true);
				// eslint-disable-next-line @typescript-eslint/ban-ts-comment
				// @ts-ignore
				meal.vegetarian = '';
				validate_input({ meal: meal }, schema_meal.addMeal);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_vegetarian);
				}
			}
		});

		it('should throw error when vegetarian invalid: string', async () => {
			expect.assertions(2);
			try {
				const meal = await testHelper.createMeal(true);
				// eslint-disable-next-line @typescript-eslint/ban-ts-comment
				// @ts-ignore
				meal.vegetarian = await testHelper.randomHex(8);
				validate_input({ meal: meal }, schema_meal.addMeal);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_vegetarian);
				}
			}
		});

		it('should throw error when vegetarian invalid: number', async () => {
			expect.assertions(2);
			try {
				const meal = await testHelper.createMeal(true);
				// eslint-disable-next-line @typescript-eslint/ban-ts-comment
				// @ts-ignore
				meal.vegetarian = testHelper.randomNumber();
				validate_input({ meal: meal }, schema_meal.addMeal);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_vegetarian);
				}
			}
		});

		it('should throw error when photoNameConverted invalid: number', async () => {
			expect.assertions(2);
			try {
				const meal = await testHelper.createMeal(true);
				// eslint-disable-next-line @typescript-eslint/ban-ts-comment
				// @ts-ignore
				meal.photoNameConverted = testHelper.randomNumber();
				validate_input({ meal: meal }, schema_meal.addMeal);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_photo_converted);
				}
			}
		});

		it('should throw error when photoNameOriginal invalid: number', async () => {
			expect.assertions(2);
			try {
				const meal = await testHelper.createMeal(true);
				// eslint-disable-next-line @typescript-eslint/ban-ts-comment
				// @ts-ignore
				meal.photoNameOriginal = testHelper.randomNumber();
				validate_input({ meal: meal }, schema_meal.addMeal);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_photo_original);
				}
			}
		});

	});

	describe(`schema_meal.singleMeal`, () => {

		it('should resolve when singleMeal object valid', async () => {
			expect.assertions(1);
			let result = false;
			try {
				validate_input({ date: testHelper.randomDate(), person: testHelper.randomPerson() }, schema_meal.singleMeal);
			} catch (e) {
				result = true;
			}
			expect(result).toBeFalsy();
		});

		it('should throw error when date not presented', async () => {
			expect.assertions(2);
			try {
				validate_input({ person: testHelper.randomPerson() }, schema_meal.singleMeal);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_date);
				}
			}
		});

		it('should throw error when date not valid', async () => {
			expect.assertions(2);
			try {
				validate_input({ date: testHelper.randomDate().substring(1,), person: testHelper.randomPerson() }, schema_meal.singleMeal);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_date);
				}
			}
		});

		it('should throw error when date not valid: random string', async () => {
			expect.assertions(2);
			try {
				validate_input({ date: testHelper.randomNumberAsString(), person: testHelper.randomPerson() }, schema_meal.singleMeal);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_date);
				}
			}
		});

		it('should throw error when date not valid: random number', async () => {
			expect.assertions(2);
			try {
				validate_input({ date: testHelper.randomNumber(), person: testHelper.randomPerson() }, schema_meal.singleMeal);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_date);
				}
			}
		});

		it('should throw error when person not presented', async () => {
			expect.assertions(2);
			try {
				validate_input({ date: testHelper.randomDate() }, schema_meal.singleMeal);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_person);
				}
			}
		});

		it('should throw error when person not valid: random string', async () => {
			expect.assertions(2);
			try {
				validate_input({ date: testHelper.randomDate(), person: await testHelper.randomHex(8) }, schema_meal.singleMeal);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_person);
				}
			}
		});

		it('should throw error when person not valid: random lowercase', async () => {
			expect.assertions(2);
			try {
				validate_input({ date: testHelper.randomDate(), person: testHelper.randomPerson().toLowerCase() }, schema_meal.singleMeal);
			} catch (e) {
				if (e instanceof Error) {
					expect(e).toBeInstanceOf(TypeError);
					expect(e.message).toEqual(testHelper.schema_error_person);
				}
			}
		});
		
	});

});