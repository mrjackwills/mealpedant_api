import { TPerson } from './index';

export const isId = <T>(input: unknown): input is T => !isNaN(Number(input)) && Number(input) > 0;

export const isPerson = (input: string): input is TPerson => input === 'Jack' || input === 'Dave';

export const isString = (input: unknown): input is string => !!input && typeof input === 'string' && input.length > 0;