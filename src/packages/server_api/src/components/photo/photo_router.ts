import { LOCATION_TEMP } from '../../config/env';
import { photo_delete, photo_post } from './photo_controller';
import { Router } from 'express';
import { wrap } from '../../lib/wrap';
import FileUpload from 'express-fileupload';
import { RoutesPhoto } from '../../types/enum_routes';

const photo_router = Router({ mergeParams: true });

photo_router.use(FileUpload({
	useTempFiles: true,
	preserveExtension: true,
	tempFileDir: LOCATION_TEMP,
}));

// Upload + Remove images - for new/edit meals
photo_router.route(RoutesPhoto.BASE)
	// Get all meal data, ids in place of categories
	.delete(wrap(photo_delete))
	// Upload a new photo, return 2 photos names, original and converted
	.post(wrap(photo_post));

export { photo_router };