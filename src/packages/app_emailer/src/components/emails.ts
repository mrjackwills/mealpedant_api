import { customTypeError } from '../config/customError';
import { WWW_DOMAIN } from '../config/env';
import { TRawBody, TDataReset, TEmailTransporter, TDataBaseUser, TAdminEmail, TDataVerify, TEmailTFA } from 'types';
import { EmailButtonText, EmailLineOne, EmailLineTwo, EmailTitle } from '../types/enum_email';
import { emailTransporter } from '../lib/emailTransporter';

class Emailer {

	#emailTransporter!: TEmailTransporter;
	constructor (emailTransporter: TEmailTransporter) {
		this.#emailTransporter = emailTransporter;
	}

	async verifyAccount ({ email, firstName, ipId, userAgentId, verifyString }:TDataVerify): Promise<void> {
		if (!email|| !firstName || !verifyString || !ipId || !userAgentId) {
			throw customTypeError('verifyAccount_email(): !email|| !firstName || !verifyString || !ipId || !userAgentId ');
		}
		const title = EmailTitle.VERIFY;
		const rawBody: TRawBody = {
			firstName,
			lineOne: EmailLineOne.VERIFY,
			buttonText: EmailButtonText.VERIFY,
			buttonLink: `https://${WWW_DOMAIN}/user/verify/${verifyString}`,
		};
		await this.#emailTransporter({ title, rawBody, email, ipId, userAgentId });
	}

	async changePassword ({ email, firstName, userId, ipId, userAgentId }: TDataBaseUser): Promise<void> {
		if (!firstName || !email || !userId || !ipId || !userAgentId) {
			throw customTypeError('email_changePassword: !firstName || !email || !userId || !ipId || !userAgentId');
		}
		const title = EmailTitle.PASSWORD_CHANGED;
		const rawBody: TRawBody = {
			firstName,
			lineOne: EmailLineOne.PASSWORD_CHANGE,
			lineTwo: EmailLineTwo.DIDNT_PASSWORD
		};
		await this.#emailTransporter({ title, rawBody, email, ipId, userAgentId, userId });
	}

	async loginAttempt ({ email, firstName, ipId, userAgentId, userId }: TDataBaseUser): Promise<void> {
		if (!email || !firstName || !userId || !ipId || !userAgentId) {
			throw customTypeError('email_loginAttempt: !email || !firstName || !userId || !ipId || !userAgentId');
		}
		const title = EmailTitle.SECURITY;
		const rawBody: TRawBody = {
			firstName,
			lineOne: EmailLineOne.FAILED_LOGIN,
			lineTwo: EmailLineTwo.DIDNT_ATTEMPT
		};
		await this.#emailTransporter({ title, rawBody, email, ipId, userAgentId, userId });
	}
		
	async resetPassword ({ email, firstName, resetString, ipId, userAgentId, userId }: TDataReset): Promise<void> {
		if (!email || !firstName || !resetString || !ipId || !userAgentId || !userId) {
			throw customTypeError('email_resetPassword(): !email|| !firstName || !resetString || !ipId || !userAgentId || !userId');
		}
		
		const title = EmailTitle.PASSWORD_RESET;
		const rawBody: TRawBody = {
			firstName,
			lineOne: EmailLineOne.PASSWORD_RESET,
			lineTwo: EmailLineTwo.DIDNT_REQUEST,
			buttonText: EmailButtonText.RESET,
			buttonLink: `https://${WWW_DOMAIN}/user/reset/${resetString}`
		};
		await this.#emailTransporter({ title, rawBody, email, ipId, userAgentId, userId });
	}

	async customAdminEmail (data: TAdminEmail): Promise<void> {
			
		if (!data.email|| !data.firstName || !data.ipId || !data.userAgentId || !data.title || !data.lineOne || !data.userId) {
			throw customTypeError('email_customEmail(): !email|| !firstName || !userId || !verifyString || !ipId || !userAgentId');
		}
		if (data.buttonText && !data.buttonLink || data.buttonLink && !data.buttonText) {
			throw customTypeError('email_customEmail(): !(buttonLink && buttontext)');
		}
		
		const rawBody: TRawBody = {
			firstName: data.firstName,
			lineOne: data.lineOne,
			lineTwo: data.lineTwo,
			buttonText: data.buttonText,
			buttonLink: data.buttonLink
		};
			
		await this.#emailTransporter({ title: data.title, rawBody, email: data.email, ipId: data.ipId, userAgentId: data.userAgentId, userId: data.userId });
	}

	async twoFA ({ email, enabled, firstName, ipId, userAgentId, userId }: TEmailTFA): Promise<void> {
		if (!email || !firstName || !userId || !ipId || !userAgentId) throw customTypeError('email_twoFA: !email || !firstName || !userId || !ipId || !userAgentId');
		const title = enabled ? EmailTitle.TWO_FA_ENABLE : EmailTitle.TWO_FA_DISABLE;
		const lineOne = enabled ? EmailLineOne.TWO_FA_ENABLE : EmailLineOne.TWO_FA_DISABLE;
		const rawBody: TRawBody = {
			firstName,
			lineOne,
			lineTwo: EmailLineTwo.DIDNT_ENABLE,
			buttonText: enabled? EmailButtonText.GENERATE_BACKUPS: undefined,
			buttonLink: enabled? `https://${WWW_DOMAIN}/settings` : undefined,
		};
		await this.#emailTransporter({ title, rawBody, email, ipId, userAgentId, userId });
	}
		
	async twoFABackup ({ email, firstName, ipId, userAgentId, userId, enabled }: TEmailTFA): Promise<void> {
		if (!email || !firstName || !ipId || !userAgentId|| !userId) throw customTypeError('email_twoFABackup: !email || !firstName || !userId || !ipId ||!userAgentId');
		const title = enabled ? EmailTitle.TWO_FA_BACKUP_CREATED : EmailTitle.TWO_FA_BACKUP_REMOVED;
		const lineOne = enabled ? EmailLineOne.TWO_FA_BACKUP_CREATED : EmailLineOne.TWO_FA_BACKUP_REMOVED;
		const rawBody: TRawBody = {
			firstName,
			lineOne,
			lineTwo: EmailLineTwo.DIDNT_ENABLE,
			buttonText: ! enabled ? EmailButtonText.GENERATE_BACKUPS: undefined,
			buttonLink: ! enabled ? `https://${WWW_DOMAIN}/settings` : undefined,
		};
		await this.#emailTransporter({ title, rawBody, email, ipId, userAgentId, userId });
	}
		
}

export const emailer = new Emailer(emailTransporter);