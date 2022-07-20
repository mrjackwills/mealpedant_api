import { customError, customTypeError } from '../../config/customError';
import { ErrorMessages } from '../../types/enum_error';
import { fileExists, randomHex, } from '../../lib/helpers';
import { HttpCode } from '../../types/enum_httpCode';
import { LOCATION_PHOTO_CONVERTED, LOCATION_PHOTO_ORIGINAL, } from '../../config/env';
import { promises as fs } from 'fs';
import { rabbit_photoConvertor } from '../../lib/rabbitRpc';
import { RequestMethod } from 'types';
import { schema_photo, TSchemaPhotoDelete } from './photo_schema';
import { send } from '../../lib/send';
import { UploadedFile } from 'express-fileupload';
import { validate_input } from '../../lib/validateinput';

// Image upload route, convert file and return filenames to user
export const photo_post: RequestMethod = async (req, res) => {
	if (!req.files) throw customTypeError(ErrorMessages.NO_PHOTO, HttpCode.BAD_REQUEST);
	const uploadedImage = req.files.image as UploadedFile;
	if (!uploadedImage || !uploadedImage.mimetype) throw customTypeError(ErrorMessages.NO_PHOTO, HttpCode.BAD_REQUEST);
	const mime = uploadedImage.mimetype.split('/');
	if (!mime || !mime[1]) throw customTypeError(ErrorMessages.NO_PHOTO, HttpCode.BAD_REQUEST);
	if (! [ 'JPEG', 'JPG', 'BMP', 'PNG' ].includes(mime[1].toUpperCase())) throw customTypeError(ErrorMessages.NOT_IMAGE, HttpCode.BAD_REQUEST);

	// Check to see if file is smaller than 10mb, also done client side
	if (uploadedImage.size > 10240000) throw customError(HttpCode.PAYLOAD_TOO_LARGE, ErrorMessages.FILE_SIZE);
	const original = uploadedImage;

	// Create new file name, timestamp plus random string
	const originalFileName = `${original.name.split('.')[0]}_O_${await randomHex(16)}.${original.mimetype.split('/')[1]}`;

	// Move the original image to original folder with new name, .mv is a function of the express-fileupload method
	await original.mv(`${LOCATION_PHOTO_ORIGINAL}/${originalFileName}`);

	const convertedFileName = await rabbit_photoConvertor(originalFileName);
	if (!convertedFileName) throw customError(HttpCode.INTERNAL_SERVER_ERROR);
	
	if (global.gc) global.gc();
	
	// Return the file names to the client
	send({ res, response: { o: originalFileName, c: convertedFileName } });
};

// Delete image from server
export const photo_delete: RequestMethod = async (req, res) => {

	const body = <TSchemaPhotoDelete>validate_input(req.body, schema_photo.delete);

	const original = `${LOCATION_PHOTO_ORIGINAL}/${body.o}`;
	const converted = `${LOCATION_PHOTO_CONVERTED}/${body.c}`;

	// use fileExists helper here
	const [ originalExists, convertedExists ] = await Promise.all([
		fileExists(original),
		fileExists(converted)
	]);
		// Throw error if either don't exists
	if (!originalExists || !convertedExists) throw customError(HttpCode.NOT_FOUND, ErrorMessages.FILE_NOT_FOUND);

	// async delete files
	await Promise.all([
		fs.unlink(original),
		fs.unlink(converted),
	]);
	send({ res });
};