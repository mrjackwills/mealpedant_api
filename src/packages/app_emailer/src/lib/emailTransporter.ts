import { createEmailTemplate } from '../config/emailTemplate';
import { createTransport } from 'nodemailer';
import { customTypeError } from '../config/customError';
import { EMAIL_ADDRESS, EMAIL_HOST, EMAIL_NAME, EMAIL_PASSWORD, EMAIL_PORT, MODE_ENV_PRODUCTION, MODE_ENV_DEV, LOCATION_TMP, NULL_ROUTE } from '../config/env';
import { queries } from './queries';
import { TSendEmail } from '../types';
import { log } from '../config/log';
import { Options } from 'nodemailer/lib/smtp-transport';
import { promises as fs } from 'fs';

const emailOptions: Options = {
	host: EMAIL_HOST,
	port: EMAIL_PORT,
	secure: true,
	auth: {
		user: EMAIL_ADDRESS,
		pass: EMAIL_PASSWORD
	}
};

const transporter = createTransport(emailOptions);

export const emailTransporter = async ({ email, rawBody, ipId, userAgentId, title, userId=undefined }: TSendEmail): Promise<void> => {
	try {
		if (NULL_ROUTE) return;

		if (!email) throw customTypeError('sendEmail(): !email');
		if (!title) throw customTypeError('sendEmail(): !title');
		if (!rawBody) throw customTypeError('sendEmail(): !rawBody');
		if (!userAgentId) throw customTypeError('sendEmail(): !userAgentId');
		if (!ipId) throw customTypeError('sendEmail(): !ipId');

		const html = createEmailTemplate ({
			title,
			...rawBody
		});

		const mailOption = {
			from: `${EMAIL_NAME} <${EMAIL_ADDRESS}>`,
			to: email,
			subject: `Meal Pedant - ${title}`,
			html,
		};
		if (MODE_ENV_PRODUCTION) {
			await transporter.sendMail(mailOption);
		} else if (MODE_ENV_DEV) {
			const fileName = `${LOCATION_TMP}/email${Date.now()}.html`;
			await fs.writeFile(fileName, html, 'utf-8');
			log.verbose(`dev email sent, see : ${fileName}`);
		}
		await queries.insert_emailLog ({ userId, title, rawBody, ipId, userAgentId, email });
	} catch (e) {
		log.error(e);
	}
};