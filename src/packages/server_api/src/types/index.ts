import { PoolClient } from 'pg';
import { ErrorMessages } from './enum_error';
import { HttpCode } from './enum_httpCode';
import { Response, Request, NextFunction } from 'express';
import { ResponseMessages } from './enum_response';
import { RabbitMessage } from './enum_rabbitMessage';

type Branded<K, T> = K & { __brand: T }
export type UserId = Branded<string, 'UserId'>
export type mealId = Branded<string, 'MealId'>
export type backupId = Branded<string, 'BackupId'>
export type lastId = Branded<string, 'lastId'>
export type mealDateId = Branded<string, 'mealDateId'>
export type mealCategoryId = Branded<string, 'mealCategoryId'>
export type mealDescriptionId = Branded<string, 'mealDescriptionId'>
export type mealPhotoId = Branded<string, 'mealPhotoId'>
export type passwordResetId = Branded<string, 'passwordResetId'>
export type ipId = Branded<string, 'ipId'>
export type userAgentId = Branded<string, 'userAgentId'>
export type sessionId = Branded<string, 'sessionId'>
export type personId = Branded<string, 'personId'>

type TUserId = { userId: UserId }
type TAdminEmail = { [ k in 'firstName' | 'lineOne' | 'title'] : string} & { [ k in 'buttonLink' | 'lineTwo' | 'buttonText']?: string} & TBaseEmail & TUserId
type TApiVersion = { api_version: string }
type TBackup = { [ K in 'filename' | 'filesize' ]: string }
type TBaseEmail = TIpUserAgent & { [ K in 'email' | 'firstName']: string}
type TBaseInsert = TIpUserAgent & TUserId
type TBaseMeal = { [ K in 'date' | 'category' | 'description'] : string } & { [K in 'restaurant' | 'takeaway' | 'vegetarian']: boolean } & { person: TPerson }
type TBaserUserEmail = TBaseEmail & TUserId
type TCreateBackups = TIpUserAgent & { user: TPassportDeserializedUser };
type TEmailReset = TBaseEmail & TUserId & { resetString: string }
type TEmailTFA = TBaserUserEmail & { enabled: boolean }
type TMealDatePersonObj = { meal?: TMealDatePerson }
type TPhoto = { [K in 'o' |'c']: string }
type TPoolClient = { Client: PoolClient }
type TSecret = { secret: string }
type TTFAStatus = { [ K in 'two_fa_backup' | 'two_fa_active' | 'two_fa_always_required']: boolean } & { two_fa_count?: number }
type TTwoFABackup = { twoFABackup: boolean }
type TUser = { [K in 'first_name' | 'email' | 'password_hash']: string }
type TFailedLogin = TIpUserAgent & TUserId & { errorString: ErrorMessages }
type TUserLimits = { limits: Array<TLimitedClient> }
type TResBackups = { backups: Array<string> }
type TResEmails = { emails: Array<string> }
type TResLastId = { lastId: lastId }
type TRabbitUuid = { rabbit_uuid: string }

type TBaseUser = TUser & {
	active: boolean;
	admin?: boolean;
	registered_user_id: UserId;
	two_fa_backup: string
	two_fa_enabled?: string;
}

type TSend = {
	res: Response,
	status?: HttpCode;
	response?: Array<TErrorLog>
	| Array<TAdminSession>
	| Array<TAllMealVue>
	| Array<TAllUsers>
	| Array<TBackup>
	| Array<TCategories>
	| TUserLimits
	| ErrorMessages
	| IAdminUptime
	| TMissingMealsResponse
	| TResBackups
	| TResEmails
	| TResLastId
	| ResponseMessages
	| TApiVersion
	| TMealDatePersonObj
	| TPhoto
	| TSecret
	| TTFAStatus
	| TTwoFABackup
	| TPasswordResetSelectGet
}

export type TIndividualMeal = {
	meal_date_id: mealDateId
	meal_category_id: mealCategoryId
	meal_description_id: mealDescriptionId
	meal_photo_id: mealPhotoId
}

export type TAdminPasswordReset = {
	registered_user_id: UserId
	email: string,
	password_reset_id: passwordResetId
	timestamp: Date
}

export type TAllMealVue = { ds: string } & { [ K in 'D' | 'J' ]?: TBaseMealVue }

export type TInsertBackup = TBaseInsert & { backupArray: Array<string> }
export type TInsertMeal = TBaseMeal & { [K in 'photoNameOriginal' | 'photoNameConverted']?: string|null }
export type TVerifyPassword = { [K in 'attempt' | 'known_password_hash' ]: string }

export type TDeleteSession = { [K in 'sessionName' | 'currentSession']: string }
export type TInsertPasswordReset = TIpUserAgent & TUserId & { resetString: string }
export type TInsertTFASecret = TBaseInsert & { secret: string }
export type TMissingMealsResponse = { missingMeals: Array<TMissingMeals> }
export type TNameUserId = { first_name: string, registered_user_id: UserId }
export type TPasswordResetInsert = TUserId & { password_hash: string, stringId: passwordResetId }
export type TPasswordResetSelectGet = { [ k in 'two_fa_backup' |'two_fa_active']: boolean }
export type TPasswordResetSelectPatch = { two_fa_backup: boolean, two_fa_secret: string, email: string, first_name: string, registered_user_id: UserId, password_reset_id: passwordResetId }
export type TSelectInsertCategory = TPoolClient & TUserId & { category: string }
export type TSelectInsertDate = TPoolClient & TUserId & { date: string }
export type TSelectInsertDescription = TPoolClient & TUserId & { description: string }
export type TSelectInsertPerson = { person: TPerson, userId: UserId, Client: PoolClient }
export type TSelectInsertPhoto = TPoolClient & TUserId & { [ K in 'original' |'converted']: string }
export type TSelectSession = TUserId & { currentSession : string }
export type TTwoFaBackup = { two_fa_backup_code: string, two_fa_backup_id: backupId }
export type TUpdateMeal = TUserId & { mealId: mealId, newMeal:TMealDatePerson }
export type TUpdatePasswordHash = TUserId & { passwordHash: string }

// TODO need to add 2fa backup count to query
export type TAllUsers = TUserLogin
	& { password_reset_id: string}
	& { [ K in 'passwordResetDate' | 'login_date' | 'passwordResetDate']: Date }
	& { [ K in 'login_ip' | 'loginSuccess' | 'passwordResetConsumed' | 'tfaSecret']: boolean }
	& { [ K in 'lastName' | 'password_creation_ip' | 'passwordResetCreationIp' | 'reset_string' | 'user_agent_string' | 'user_creation_ip']: string }
	
export type TAdminSession = { [ K in 'currentSession'| 'httpOnly'| 'sameSite'| 'secure']: boolean }
	& { expires: Date, originalMaxAge: number}
	& { [ K in 'domain'| 'ip' | 'path'| 'sessionKey'| 'userAgent']: string }
	
export type TMissingMeals = {
	dates_series: Date
	person: TPerson
}

export type TPersonDate = {
	person: TPerson;
	date: string;
}

export type TLoginHistory = TIpUserAgent & TUserId & {
	success?: boolean;
	sessionId?: sessionId;
}

export type IAdminUptime = { [K in 'rss' | 'heapUsed' | 'heapTotal' | 'external']: string } & {[K in 'nodeUptime' | 'serverUptime' ]: number }
export type TCategories = { [ K in 'id' | 'c' | 'n']: string }
export type TErrorLog = { [ K in 'error_log_id' | 'message' | 'stack' | 'uuid'] : string } & { timestamp: Date, level: TLogLevels, http_code: number}
export type TIpUserAgent = { ipId: ipId, userAgentId: userAgentId}
export type TLoggerColors = { readonly [index in TLogLevels]: string };
export type TLogLevels = 'debug' | 'error' | 'verbose' | 'warn'
export type TMealDatePerson = TInsertMeal & { [K in 'id' | 'meal_photo_id']: string }
export type TMealRow = TBaseMeal & { [ K in 'photo_original' | 'photo_converted']: string}
export type TNewUser = TUser & TIpUserAgent & { last_name: string };
export type TPassportDeserializedUser = TBaseUser & { two_fa_always_required: boolean }
export type TPerson = 'Dave' | 'Jack';
export type TUserLogin = TBaseUser & { login_attempt_number: string }

export type TVuexObject = TTFAStatus & {
	email: string
	admin?: boolean;
}

export type TBaseMealVue = { [ K in 'md' | 'c']: string } &
	{ [ K in 'v'|'t'|'r']?: boolean } &
	{ p?: TPhoto }

export type TLimitedClient = {
	p: number
	u: string,
	b: boolean,
	m?: number,
}

type GenIO<I, O> = (i: I) => O
export type PGenIO<I, O> = GenIO<I, P<O>>
export type GenI<I> = (i: I) => I
export type P<T> = Promise<T>
export type PV = P<void>

export type TRequest = Request

export type CheckPasswordAndToken = PGenIO<Request, void>
export type CreateBackupArray = PGenIO<Array<string>, Array<string>>
export type CreateNewBackupCodes = PGenIO<TCreateBackups, Array<string>>
export type DestroySession = (req: Request, res: Response) => PV
export type FailedLogin = PGenIO<TFailedLogin, never>
export type FileExists = PGenIO<string, boolean>

export type ListOfBackups = PGenIO<void, Array<TBackup>>
export type RandomHex = PGenIO<number, string>
export type ReqString = GenIO<Request, string>
export type RequestMethod = (req: Request, res: Response, next: NextFunction) => PV
export type Send = PGenIO<TSend, void>

export type TwoFAStatus = PGenIO<TPassportDeserializedUser, TTFAStatus>
export type UserObject = PGenIO<TPassportDeserializedUser, TVuexObject>

export type TRabbitConvert = { originalFileName: string} & TRabbitUuid

export type TCreateHash = { password: string }

export type TRabbitCreateHash = TCreateHash & TRabbitUuid
export type TRabbitValidateHash = TVerifyPassword & TRabbitUuid

type TResponseMessageConverted = RabbitMessage.PHOTO_CONVERT
type TResponseMessageCreateHash = RabbitMessage.ARGON_CREATE_HASH
type TResponseMessageValidateHash = RabbitMessage.ARGON_VALIDATE_HASH
type TResponseMessageBackupSql = RabbitMessage.BACKUP_SQL_BACKUP
type TResponseMessageBackupFull = RabbitMessage.BACKUP_FULL_BACKUP
type TResponseMessagePing = RabbitMessage.PING
type TRabbitMessageName = TResponseMessageConverted | TResponseMessageCreateHash | TResponseMessageValidateHash | TResponseMessagePing | TResponseMessageBackupSql | TResponseMessageBackupFull

export type TRabbitData = { message_name: TRabbitMessageName, data: TRabbitConvert | TRabbitCreateHash | TRabbitValidateHash | TRabbitUuid }

export type TResponseError = { message_name: RabbitMessage.ERROR, data: { error: string } }
export type TResponseCreateHash = { message_name: TResponseMessageCreateHash, data: { response: string } }
export type TResponseValidateHash = { message_name: TResponseMessageValidateHash, data: { response: boolean } }
export type TResponseBackup = { message_name: TResponseMessageBackupFull|TResponseMessageBackupSql, data: { response: boolean } }
export type TResponseConverted = { message_name: TResponseMessageConverted, data: { response: string } }
export type TResponseMessage = TResponseMessageConverted | RabbitMessage.ERROR

type TRabbitEmailChangePassword = { message_name: RabbitMessage.EMAIL_CHANGE_PASSWORD, data: TBaserUserEmail }
type TRabbitEmailLoginAttempt = { message_name: RabbitMessage.EMAIL_LOGIN_ATTEMPT, data: TBaserUserEmail }
type TRabbitEmailReset = { message_name: RabbitMessage.EMAIL_RESET, data: TEmailReset }
type TRabbitEmailTwoFABackups = { message_name: RabbitMessage.EMAIL_TWO_FA_BACKUP, data: TEmailTFA }
type TRabbitEmailTwoFA = { message_name: RabbitMessage.EMAIL_TWO_FA, data: TEmailTFA }
type TRabbitEmailVerify = { message_name: RabbitMessage.EMAIL_VERIFY, data: { verifyString: string } & TBaseEmail }
type TRabbitEmailCustom = { message_name: RabbitMessage.EMAIL_CUSTOM_ADMIN, data: TAdminEmail }

export type TRabbitEmail =
| TRabbitEmailReset
| TRabbitEmailVerify
| TRabbitEmailChangePassword
| TRabbitEmailLoginAttempt
| TRabbitEmailTwoFA
| TRabbitEmailTwoFABackups
| TRabbitEmailCustom