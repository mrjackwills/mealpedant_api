import { Router } from 'express';
import { wrap } from '../../lib/wrap';
import {
	admin_get,
	backup_delete,
	backup_get,
	backup_post,
	backupDownload_get,
	email_get,
	email_post,
	error_get,
	limit_get,
	limit_patch,
	memory_get,
	memory_put,
	restart_put,
	session_delete,
	session_get,
	user_get,
	user_patch,
} from './admin_controller';

import { RoutesAdmin } from '../../types/enum_routes';

const admin_router = Router({ mergeParams: true });

// Check user is admin, used on app load if veux.user.isAdmin is true
admin_router.route(RoutesAdmin.BASE)
	.get(wrap(admin_get));

admin_router.route(RoutesAdmin.BACKUP)
	// Delete selected db backup file
	.delete(wrap(backup_delete))
	// Get list of all db backup files
	.get(wrap(backup_get))
	// Create new db backup file
	.post(wrap(backup_post));

admin_router.route(RoutesAdmin.BACKUP_PARAM_FILENAME)
	// Download selected .tar.gpg backup file
	.get(wrap(backupDownload_get));

admin_router.route(RoutesAdmin.EMAIL)
	.get(wrap(email_get))
	.post(wrap(email_post));

admin_router.route(RoutesAdmin.ERROR)
	// get all sessions for a single user
	.get(wrap(error_get));

admin_router.route(RoutesAdmin.LIMIT)
	.get(wrap(limit_get))
	.patch(wrap(limit_patch));

admin_router.route(RoutesAdmin.MEMORY)
	.get(wrap(memory_get))
	.put(wrap(memory_put));

admin_router.route(RoutesAdmin.RESTART)
	.put(wrap(restart_put));

admin_router.route(RoutesAdmin.USER)
	// get all user info
	.get(wrap(user_get))
	.patch(wrap(user_patch));

admin_router.route(RoutesAdmin.SESSION)
	// delete single session, based on req,body,sessionKey
	.delete(wrap(session_delete));

admin_router.route(RoutesAdmin.SESSION_PARAM_EMAIL)
	// get all sessions for a single user
	.get(wrap(session_get));

export { admin_router };