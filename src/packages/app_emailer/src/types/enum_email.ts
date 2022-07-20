export enum EmailTitle {
	PASSWORD_CHANGED = 'Password changed',
	PASSWORD_RESET = 'Password reset requested',
	SECURITY =`Security Alert`,
	TWO_FA_BACKUP_CREATED = `Two Factor backups created`,
	TWO_FA_BACKUP_REMOVED = `Two Factor backups removed`,
	TWO_FA_DISABLE = `Two Factor Disabled`,
	TWO_FA_ENABLE = `Two Factor Enabled`,
	VERIFY = 'Verify email address',
}

export enum EmailButtonText {
	GENERATE_BACKUPS = 'GENERATE BACKUP CODES',
	RESET = 'RESET PASSWORD',
	VERIFY = 'VERIFY EMAIL ADDRESS',
}

export enum EmailLineOne {
	FAILED_LOGIN = `We're just letting you know that you've had 5 invalid login attempts to your Meal Pedant account.`,
	PASSWORD_CHANGE = 'The password for your Meal Pedant account has been changed.',
	PASSWORD_RESET = 'This password reset link will only be valid for one hour.',
	TWO_FA_BACKUP_CREATED = 'You have created Two Factor Authentication backup codes for your Meal Pedant account. The codes should be stored somewhere secure.',
	TWO_FA_BACKUP_REMOVED = `You have removed the Two Factor Authentication backup codes for your Meal Pedant account. New backup codes can be created at any time from the user settings page.`,
	TWO_FA_DISABLE = 'You have disabled Two-Factor Authentication for your Meal Pedant account.',
	TWO_FA_ENABLE =`You have enabled Two-Factor Authentication for your Meal Pedant account, it is recommended to create and save backup codes, these can be generated in the user settings area.`,
	VERIFY = 'Welcome to Meal Pedant, before you start we just need you to verify this email address',
}

export enum EmailLineTwo {
	DIDNT_ENABLE = `If you did not enable this setting, please contact support as soon as possible`,
	DIDNT_ATTEMPT = `If you did not make these attempts, please contact support immediately.`,
	DIDNT_PASSWORD = 'If you did not make this change then please contact support immediately.',
	DIDNT_REQUEST = 'If you did not request a password reset then please ignore this email',
}