import { RabbitMessage } from './enum_rabbitMessage';

type Branded<K, T> = K & { __brand: T }
type Id<T> = Branded<string, T>

export type AppStatusId = Id<'AppStatusId'>
export type AppNameId = Id<'AppNameId'>

export type TArgonValidate = { [K in 'attempt' | 'known_password_hash' ]: string }

export type TLoggerColors = { readonly [index in TLogLevels]: string };
export type TLogLevels = 'debug' | 'error' | 'verbose' | 'warn'

export type TErrorLog = { [ K in 'error_log_id' | 'message' | 'stack' | 'uuid'] : string } & { timestamp: Date, level: TLogLevels, http_code: number}

export type TMessage = RabbitMessage.PING | RabbitMessage.ARGON_CREATE_HASH | RabbitMessage.ARGON_VALIDATE_HASH
export type TValidateHash = { message_name: RabbitMessage.ARGON_VALIDATE_HASH, data: TArgonValidate }
export type TCreateHash = { message_name: RabbitMessage.ARGON_CREATE_HASH, data: { password: string} }
export type TPing = { message_name: RabbitMessage.PING }