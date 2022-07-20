import { spawn, Thread, Worker, FunctionThread } from 'threads';
import { TScriptName, PGenIO } from '../types';

class BackupSpawner {

	async create (scriptName: TScriptName): Promise<boolean> {
		let backupScript: FunctionThread|undefined;
		try {
			backupScript = await spawn<PGenIO<TScriptName, boolean>>(new Worker(`../../dist/workers/backup.js`));
			const backupCreated = await backupScript(scriptName);
			return backupCreated;
		} finally {
			if (backupScript) await Thread.terminate(backupScript);
		}
	}
}

export const backupSpawner = new BackupSpawner();