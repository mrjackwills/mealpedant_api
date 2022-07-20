export enum RoutesRoot {
	ADMIN = '/admin',
	AUTHENTICATED = '/authenticated',
	BASE ='/',
	CATCH_ALL = '*',
	FOOD ='/food',
	INCOGNITO = '/incognito',
	MEAL ='/meal',
	PHOTO ='/photo',
	USER = '/user',
}

export enum RoutesAdmin {
	BACKUP = '/backup',
	BACKUP_PARAM_FILENAME = '/backup/:fileName',
	BASE = '/',
	EMAIL = '/email',
	ERROR ='/error',
	LIMIT = '/limit',
	MEMORY = '/memory',
	RESTART = '/restart',
	SESSION = '/session',
	SESSION_PARAM_EMAIL ='/session/:email',
	USER = '/user',
}

export enum RoutesFood {
	ALL = '/all',
	CACHE = '/cache',
	CATEGORY = '/category',
	LAST = '/last',
}

export enum RoutesIncognito {
	SIGNIN = '/signin',
	ONLINE = '/online',
	REGISTER = '/register',
	VERIFY_PARAM_VERIFYSTRING = '/verify/:verifyString',
	RESET = '/reset-password',
	RESET_PARAM_RESETSTRING = '/reset-password/:resetString',
}

export enum RoutesMeal {
	BASE = '/',
	BASE_PARAM_DATE_PARAM_PERSON = '/:date/:person',
	MISSING ='/missing'
}

export enum RoutesPhoto {
	BASE ='/'
}

export enum RoutesUser {
	BASE ='/',
	SIGNOUT = '/signout',
	PASSWORD = '/password',
	SETUP_TWO_FA = '/setup/twofa',
	TWO_FA = '/twofa'
}