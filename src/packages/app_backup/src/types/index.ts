import { RabbitMessage } from './enum_rabbitMessage';

type Branded<K, T> = K & { __brand: T }
type Id<T> = Branded<string, T>

type GenIO<I, O> = (i: I) => O
export type PGenIO<I, O> = GenIO<I, Promise<O>>

export type AppStatusId = Id<'AppStatusId'>
export type AppNameId = Id<'AppNameId'>

export type TLoggerColors = { readonly [index in TLogLevels]: string };
export type TLogLevels = 'debug' | 'error' | 'verbose' | 'warn'

export type TErrorLog = { [ K in 'error_log_id' | 'message' | 'stack' | 'uuid'] : string } & { timestamp: Date, level: TLogLevels, http_code: number}

export type TScriptName = 'FULL' | 'SQL_ONLY'

export type TMessageName = RabbitMessage.PING | RabbitMessage.BACKUP_FULL_BACKUP | RabbitMessage.BACKUP_SQL_BACKUP
export type TFull = { message_name: RabbitMessage.BACKUP_FULL_BACKUP}
export type TSQLOnly = { message_name: RabbitMessage.BACKUP_SQL_BACKUP}
export type TPing = { message_name: RabbitMessage.PING }