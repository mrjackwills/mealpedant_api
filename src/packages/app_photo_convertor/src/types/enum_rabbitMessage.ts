export enum RabbitMessage {
	ERROR = 'error',
	PING = 'ping',
	PONG = 'pong',

	ARGON_CREATE_HASH = 'argon::create_hash',
	ARGON_VALIDATE_HASH = 'argon::validate_hash',
	
	BACKUP_SQL_BACKUP = 'backup::sql_backup',
	BACKUP_FULL_BACKUP = 'backup::full_backup',
	
	EMAIL_CHANGE_PASSWORD = 'email::change_password',
	EMAIL_CUSTOM_ADMIN = 'email::custom_admin',
	EMAIL_LOGIN_ATTEMPT = 'email::login_attempt',
	EMAIL_RESET = 'email::reset',
	EMAIL_TWO_FA = 'email::twofa',
	EMAIL_TWO_FA_BACKUP = 'email::twofa_backup',
	EMAIL_VERIFY = 'email::verify',

	PHOTO_CONVERT = 'photo::convert',

}
