import { EmailButtonText, EmailLineOne, EmailLineTwo, EmailTitle } from './enum_email';
import { RabbitMessage } from './enum_rabbitMessage';

type Branded<K, T> = K & { __brand: T }
type Id<T> = Branded<string, T>
export type userId = Branded<string, 'UserId'>

export type AppStatusId = Id<'AppStatusId'>
export type AppNameId = Id<'AppNameId'>
export type EmailId = Id<'EmailId'>
export type UserId = Id<'UserId'>
export type DeviceId = Id<'DeviceId'>
export type IpId = Id<'IpId'>
export type UserAgentId = Id<'UserAgentId'>

export type TEmailTransporter = (x: TSendEmail) => Promise<void>

export type TLoggerColors = { readonly [index in TLogLevels]: string };
export type TLogLevels = 'debug' | 'error' | 'verbose' | 'warn'
export type TIpUserAgent = { ipId: IpId, userAgentId?: UserAgentId }

export type TErrorLog = { [ K in 'error_log_id' | 'message' | 'stack' | 'uuid'] : string } & { timestamp: Date, level: TLogLevels, http_code: number}

export type TRawBody = {
	firstName: string;
	lineOne: EmailLineOne;
	lineTwo?: EmailLineTwo
	buttonText?: EmailButtonText,
	buttonLink?: string
}

type TEmail = TRawBody & { title: EmailTitle }
export type TFEmailTemplate = (x: TEmail) => string

type TUserId = { userId: userId }

export type TBaseSendEmail = {
	title: EmailTitle
	rawBody: TRawBody
	ipId: IpId,
	securityEmail?: boolean,
	userId?: UserId
	userAgentId?: UserAgentId
	deviceId?: DeviceId
}

export type TSendEmail = TIpUserAgent & Partial<TUserId> & {
	email: string;
	rawBody: TRawBody;
	title: EmailTitle;
}

type TBaseEmail = TIpUserAgent & { [ K in 'email' | 'firstName']: string}
export type TEmailInterface = TRawBody & { title: EmailTitle }

export type TDataBaseUser = TBaseEmail & { userId : UserId }

export type TAdminEmail = TEmailInterface & TBaseEmail & TUserId

export type TDataVerify = { verifyString: string} & TBaseEmail

export type TDataReset = TBaseEmail & TUserId & { resetString: string }

export type TEmailTFA = TDataBaseUser & { enabled: boolean }

type TData2FA = TDataBaseUser & { enabled: boolean }

export type TMessage2FA = {message_name: RabbitMessage.EMAIL_TWO_FA, data: TData2FA }
export type TMessage2FABackups = {message_name: RabbitMessage.EMAIL_TWO_FA_BACKUP, data: TData2FA }
export type TMessageChangePassword = { message_name: RabbitMessage.EMAIL_CHANGE_PASSWORD, data: TDataBaseUser }
export type TMessageLoginAttempt = {message_name: RabbitMessage.EMAIL_LOGIN_ATTEMPT, data: TDataBaseUser }
export type TMessageReset = { message_name: RabbitMessage.EMAIL_RESET, data: TDataReset }
export type TMessageVerify = { message_name: RabbitMessage.EMAIL_VERIFY, data: TDataVerify }
type TRabbitEmailCustom = { message_name: RabbitMessage.EMAIL_CUSTOM_ADMIN, data: TAdminEmail }

export type TMessageName = RabbitMessage.EMAIL_TWO_FA
	| RabbitMessage.EMAIL_TWO_FA_BACKUP
	| RabbitMessage.EMAIL_CHANGE_PASSWORD
	| RabbitMessage.EMAIL_LOGIN_ATTEMPT
	| RabbitMessage.EMAIL_RESET
	| RabbitMessage.EMAIL_VERIFY
	| RabbitMessage.EMAIL_CUSTOM_ADMIN

export type TMessage = TMessageReset
| TMessageVerify
| TMessageChangePassword
| TMessageLoginAttempt
| TMessage2FA
| TMessage2FABackups
| TRabbitEmailCustom