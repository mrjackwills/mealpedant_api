import { food_router } from '../components/food/food_router';
import { isAuthenticated } from '../lib/checkAuthentication';
import { Router } from 'express';
import { user_router } from '../components/user/user_router';
import { RoutesRoot } from '../types/enum_routes';

const AuthenticatedRoutes = Router({ mergeParams: true });

// Apply isAuthenticated to all routes
AuthenticatedRoutes.use(isAuthenticated);

AuthenticatedRoutes.use(RoutesRoot.USER, user_router);
AuthenticatedRoutes.use(RoutesRoot.FOOD, food_router);

export { AuthenticatedRoutes };