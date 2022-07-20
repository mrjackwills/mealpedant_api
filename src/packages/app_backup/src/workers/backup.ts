import { customTypeError, customError } from '../config/customError';
import { ErrorMessages } from '../types/enum_error';
import { execFile } from 'child_process';
import { expose } from 'threads/worker';
import { fileExists } from '../lib/helpers';
import { HttpCode } from '../types/enum_httpCode';
import { LOCATION_SCRIPTS } from '../config/env';
import { TScriptName } from 'types';

const runScript = (fileName: string): Promise<string> => new Promise((resolve, reject) => {

	const backupBash = execFile(fileName);

	backupBash.stderr?.on('error', (error)=> {
		reject(error);
	});

	backupBash.on('exit', (code) => {
		if (code === 0) resolve('resolve');
		else reject(customError(HttpCode.INTERNAL_SERVER_ERROR));
	});

	backupBash.on('error', (e) => {
		const message = e instanceof Error? e.message: ErrorMessages.WORKER;
		reject(customTypeError(message));
	});

});

const backup = async (scriptName: TScriptName): Promise<boolean> => {
	// Check backup script exists
	const fileName = `${LOCATION_SCRIPTS}/${scriptName}.sh`;
	const valid = await fileExists(fileName);
	if (!valid) throw customTypeError('script not found');

	await runScript(fileName);
	return true;
};

expose(backup);