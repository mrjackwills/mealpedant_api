import { schema_shared } from '../shared/shared_schema';
import * as joi from 'types-joi';

class Photo {
	readonly delete = joi.object({
		o: schema_shared.imageNameOriginal.required(),
		c: schema_shared.imageNameConverted.required()
	}).required();
}

export const schema_photo = new Photo();
export type TSchemaPhotoDelete = joi.InterfaceFrom<typeof schema_photo.delete>
