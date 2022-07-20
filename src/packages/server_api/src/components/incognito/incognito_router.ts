import { forgot_post, online_get, register_post, resetPassword_get, resetPassword_patch, signin_body_validator, verify_get } from '../../components/incognito/incognito_controller';
import { passport } from '../../config/passport';
import { Router } from 'express';
import { signin_post } from '../../components/user/user_controller';
import { wrap } from '../../lib/wrap';
import { RoutesIncognito } from '../../types/enum_routes';

const incognito = Router({ mergeParams: true });

incognito.post(RoutesIncognito.SIGNIN,
	wrap(signin_body_validator),
	wrap(passport.authenticate('local', { failWithError: true })),
	wrap(signin_post)
);

// basic check that the server is online
incognito.route(RoutesIncognito.ONLINE)
	.get(wrap(online_get));

// User register
incognito.route(RoutesIncognito.REGISTER)
	.post(wrap(register_post));

// Verify new account
incognito.route(RoutesIncognito.VERIFY_PARAM_VERIFYSTRING)
	.get(wrap(verify_get));

// Forgot password, send email to user with url to visit
incognito.route(RoutesIncognito.RESET)
	.post(wrap(forgot_post));

// url to visit to reset password, as made above
incognito.route(RoutesIncognito.RESET_PARAM_RESETSTRING)
	.get(wrap(resetPassword_get))
	.patch(wrap(resetPassword_patch));

export { incognito };