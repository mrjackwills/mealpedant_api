import { admin_router } from '../components/admin/admin_router';
import { isAdmin } from '../lib/checkAuthentication';
import { meal_router } from '../components/meal/meal_router';
import { photo_router } from '../components/photo/photo_router';
import { Router } from 'express';

import { RoutesRoot } from '../types/enum_routes';

const AdminRoutes = Router({ mergeParams: true });

// Apply isAdmin protection to following routes
AdminRoutes.use(isAdmin);

AdminRoutes.use(RoutesRoot.PHOTO, photo_router);
AdminRoutes.use(RoutesRoot.MEAL, meal_router);

// more refactoring can be done here, /backup, /server, /users, etc
AdminRoutes.use(RoutesRoot.BASE, admin_router);

export { AdminRoutes };