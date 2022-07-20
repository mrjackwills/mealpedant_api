
export const isString = (input: unknown): input is string => !!input && typeof input === 'string' && input.length > 0;