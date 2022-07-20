import { RabbitMessage } from './enum_rabbitMessage';

type Branded<K, T> = K & { __brand: T }
type Id<T> = Branded<string, T>

export type AppStatusId = Id<'AppStatusId'>
export type AppNameId = Id<'AppNameId'>

export type TLoggerColors = { readonly [index in TLogLevels]: string };
export type TLogLevels = 'debug' | 'error' | 'verbose' | 'warn'

export type TErrorLog = { [ K in 'error_log_id' | 'message' | 'stack' | 'uuid'] : string } & { timestamp: Date, level: TLogLevels, http_code: number}

export type TMessage = RabbitMessage.PING | RabbitMessage.PHOTO_CONVERT
export type TConvertPhoto = { message_name: RabbitMessage.PHOTO_CONVERT, data: { originalFileName: string} }
export type TPing = { message_name: RabbitMessage.PING }