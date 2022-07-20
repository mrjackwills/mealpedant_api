export enum RedisKey {
	CACHE_ALL_MEAL = 'cache:allMeal',
	CACHE_ALL_CATEGORY = 'cache:allCategory',
	CACHE_LAST_ID = 'cache:lastMealEditId',
	LIMITER_SET = 'limiter:set',
	LIMITER = 'limiter:',
	SESSION_SET = 'set:session:',
	TWO_FA_SETUP = '2fa:setup:',
	VERIFY_EMAIL = `verify:email:`,
	VERIFY_STRING = `verify:string:`,
	SESSION = `session:`
}