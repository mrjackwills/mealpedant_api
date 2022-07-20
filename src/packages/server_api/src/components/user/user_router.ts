import { Router } from 'express';
import { wrap } from '../../lib/wrap';
import {
	changePassword_patch,
	setupTwoFA_delete,
	setupTwoFA_get,
	setupTwoFA_patch,
	setupTwoFA_post,
	signout_post,
	twoFA_delete,
	twoFA_get,
	twoFA_patch,
	twoFA_post,
	twoFA_put,
	user_get,
} from './user_controller';

import { RoutesUser } from '../../types/enum_routes';

const user_router = Router({ mergeParams: true });

// Check user is authenticated and return {email,admin}
user_router.route(RoutesUser.BASE)
	.get(wrap(user_get));

// User sign out route
user_router.route(RoutesUser.SIGNOUT)
	.post(wrap(signout_post));

// Change password via settings page
user_router.route(RoutesUser.PASSWORD)
	.patch(wrap(changePassword_patch));

// Two Factor Authentication setup
user_router.route(RoutesUser.SETUP_TWO_FA)
	// Cancel 2fa setup - just delete key from redis
	.delete(wrap(setupTwoFA_delete))
	// 2fa setup, retreive secret token (create qr code with it client side)
	.get(wrap(setupTwoFA_get))
	// Enabled twoFA on all password required dialogs
	.patch(wrap(setupTwoFA_patch))
	// Post 6 digit token created with qr-token, to make sure it all works
	.post(wrap(setupTwoFA_post));

// Two Factor Authentication details
user_router.route(RoutesUser.TWO_FA)
	// Remove user 2fa completely
	.delete(wrap(twoFA_delete))
	// Check status of twofa enabled & backup
	.get(wrap(twoFA_get))
	// Create backup codes - will remove if any already there
	.post(wrap(twoFA_post))
	// re_generate backup codes backup codes - will remove if any already there
	.patch(wrap(twoFA_patch))
	// delete 2fa backup codes
	.put(wrap(twoFA_put));

export { user_router };