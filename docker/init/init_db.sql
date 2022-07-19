
-- just make the db, don't create any tables !!!!
-- just make the db, don't create any tables !!!!
-- just make the db, don't create any tables !!!!
-- just make the db, don't create any tables !!!!
-- pg_restore -h localhost -p 5432 -U jack -c -d mealpedant -v ./pg_dump.tar

CREATE ROLE "$DB_USER" WITH LOGIN PASSWORD "$DB_PASSWORD";
CREATE DATABASE $DB_NAME;
GRANT ALL PRIVILEGES ON DATABASE $DB_NAME TO $DB_USER;
CREATE DATABASE mealpedant WITH OWNER jack;
GRANT ALL PRIVILEGES ON DATABASE mealpedant TO mealpedant;


CREATE DATABASE mealpedant;
GRANT ALL PRIVILEGES ON DATABASE mealpedant TO mealpedant;

\c mealpedant

CREATE TABLE IF NOT EXISTS ip_address (
	ip_id BIGINT GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY,
	timestamp TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
	ip INET UNIQUE NOT NULL
);

GRANT ALL ON ip_address TO mealpedant;
GRANT USAGE, SELECT ON SEQUENCE ip_address_ip_id_seq TO mealpedant;

CREATE TABLE IF NOT EXISTS user_agent (
	user_agent_id BIGINT GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY,
	timestamp TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
	user_agent_string TEXT UNIQUE NOT NULL
);

GRANT ALL ON user_agent TO mealpedant;
GRANT USAGE, SELECT ON SEQUENCE user_agent_user_agent_id_seq TO mealpedant;

CREATE TABLE IF NOT EXISTS registered_user (
	registered_user_id BIGINT GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY,
	first_name TEXT NOT NULL,
	last_name TEXT NOT NULL,
	email TEXT UNIQUE NOT NULL,
	active BOOLEAN DEFAULT false,
	password_hash TEXT NOT NULL,
	timestamp TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
	ip_id BIGINT REFERENCES ip_address(ip_id) NOT NULL,
	user_agent_id BIGINT REFERENCES user_agent(user_agent_id) NOT NULL
);

GRANT ALL ON registered_user TO mealpedant;
GRANT USAGE, SELECT ON SEQUENCE registered_user_registered_user_id_seq TO mealpedant;
CREATE INDEX email_index ON registered_user(email);

CREATE TABLE IF NOT EXISTS admin_user (
	admin_id BIGINT GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY ,
	timestamp TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
	registered_user_id BIGINT UNIQUE REFERENCES registered_user(registered_user_id),
	ip_id BIGINT REFERENCES ip_address(ip_id) NOT NULL,
	admin BOOLEAN DEFAULT false
);

GRANT ALL ON admin_user TO mealpedant;
GRANT USAGE, SELECT ON SEQUENCE admin_user_admin_id_seq TO mealpedant;

CREATE TABLE IF NOT EXISTS login_attempt (
	login_attempt_id BIGINT GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY ,
	registered_user_id BIGINT UNIQUE REFERENCES registered_user(registered_user_id) ON DELETE CASCADE,
	login_attempt_number BIGINT DEFAULT 0
);

GRANT ALL ON login_attempt TO mealpedant;
GRANT USAGE, SELECT ON SEQUENCE login_attempt_login_attempt_id_seq TO mealpedant;

CREATE TABLE IF NOT EXISTS password_reset (
	password_reset_id BIGINT GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY,
	timestamp TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
	registered_user_id BIGINT REFERENCES registered_user(registered_user_id) ON DELETE CASCADE,
	reset_string TEXT NOT NULL CHECK(LENGTH(reset_string)=256),
	ip_id BIGINT REFERENCES ip_address(ip_id) NOT NULL,
	user_agent_id BIGINT REFERENCES user_agent(user_agent_id) NOT NULL,
	consumed BOOLEAN DEFAULT false
);

GRANT ALL ON password_reset TO mealpedant;
GRANT USAGE, SELECT ON SEQUENCE password_reset_password_reset_id_seq TO mealpedant;
CREATE INDEX reset_string ON password_reset(reset_string);

CREATE TABLE IF NOT EXISTS meal_category (
	meal_category_id BIGINT GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY,
	category TEXT UNIQUE NOT NULL,
	registered_user_id BIGINT NOT NULL REFERENCES registered_user(registered_user_id),
	timestamp TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

GRANT ALL ON meal_category TO mealpedant;
GRANT USAGE, SELECT ON SEQUENCE meal_category_meal_category_id_seq TO mealpedant;

CREATE TABLE IF NOT EXISTS meal_description (
	meal_description_id BIGINT GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY,
	description TEXT UNIQUE,
	registered_user_id BIGINT NOT NULL REFERENCES registered_user(registered_user_id),
	timestamp TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

GRANT ALL ON meal_description TO mealpedant;
GRANT USAGE, SELECT ON SEQUENCE meal_description_meal_description_id_seq TO mealpedant;

CREATE TABLE IF NOT EXISTS meal_person (
	meal_person_id BIGINT GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY,
	-- registered_user_id BIGINT NOT NULL REFERENCES registered_user(registered_user_id),
	person TEXT UNIQUE NOT NULL CHECK (person IN ('Dave','Jack'))
);

GRANT ALL ON meal_person TO mealpedant;
GRANT USAGE, SELECT ON SEQUENCE meal_person_meal_person_id_seq TO mealpedant;

-- Create the two people entries here
INSERT INTO meal_person (person) VALUES('Dave'), ('Jack');

CREATE TABLE IF NOT EXISTS meal_photo (
	meal_photo_id BIGINT GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY,
	registered_user_id BIGINT NOT NULL REFERENCES registered_user(registered_user_id),
	timestamp TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
	photo_original TEXT NOT NULL UNIQUE,
	photo_converted TEXT NOT NULL UNIQUE
);

GRANT ALL ON meal_photo TO mealpedant;
GRANT USAGE, SELECT ON SEQUENCE meal_photo_meal_photo_id_seq TO mealpedant;

CREATE TABLE IF NOT EXISTS meal_date (
	meal_date_id BIGINT GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY,
	registered_user_id BIGINT NOT NULL REFERENCES registered_user(registered_user_id),
	date_of_meal DATE UNIQUE CHECK(date_of_meal >= DATE '2015-05-09'),
	timestamp TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

GRANT ALL ON meal_date TO mealpedant;
GRANT USAGE, SELECT ON SEQUENCE meal_date_meal_date_id_seq TO mealpedant;

CREATE TABLE IF NOT EXISTS individual_meal (
	individual_meal_id BIGINT GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY,
	meal_date_id BIGINT NOT NULL REFERENCES meal_date(meal_date_id),
	meal_person_id BIGINT NOT NULL REFERENCES meal_person(meal_person_id),
	meal_category_id BIGINT NOT NULL REFERENCES meal_category(meal_category_id),
	meal_description_id BIGINT NOT NULL REFERENCES meal_description(meal_description_id),
	restaurant BOOLEAN,
	takeaway BOOLEAN,
	vegetarian BOOLEAN,
	meal_photo_id BIGINT REFERENCES meal_photo(meal_photo_id),
	registered_user_id BIGINT NOT NULL REFERENCES registered_user(registered_user_id),
	timestamp TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
	UNIQUE (meal_date_id, meal_person_id)
);

GRANT ALL ON individual_meal TO mealpedant;
GRANT USAGE, SELECT ON SEQUENCE individual_meal_individual_meal_id_seq TO mealpedant;

CREATE TABLE IF NOT EXISTS login_history (
	login_history_id BIGINT GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY,
	registered_user_id BIGINT REFERENCES registered_user(registered_user_id) ON DELETE CASCADE,
	timestamp TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
	ip_id BIGINT REFERENCES ip_address(ip_id),
	user_agent_id BIGINT REFERENCES user_agent(user_agent_id),
	session_name TEXT,
	success BOOLEAN
);

GRANT ALL ON login_history TO mealpedant;
GRANT USAGE, SELECT ON SEQUENCE login_history_login_history_id_seq TO mealpedant;

CREATE TABLE IF NOT EXISTS two_fa_secret (
	two_fa_secret_id BIGINT GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY ,
	timestamp TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
	registered_user_id BIGINT UNIQUE REFERENCES registered_user(registered_user_id) ON DELETE CASCADE,
	ip_id BIGINT REFERENCES ip_address(ip_id) NOT NULL,
	user_agent_id BIGINT REFERENCES user_agent(user_agent_id) NOT NULL,
	two_fa_secret TEXT DEFAULT NULL,
	always_required BOOLEAN DEFAULT FALSE
);

GRANT ALL ON two_fa_secret TO mealpedant;
GRANT USAGE, SELECT ON SEQUENCE two_fa_secret_two_fa_secret_id_seq TO mealpedant;

-- * Check codes are 8 chars long, and that user+code is unique
CREATE TABLE IF NOT EXISTS two_fa_backup (
	two_fa_backup_id BIGINT GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY,
	timestamp TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
	registered_user_id BIGINT REFERENCES registered_user(registered_user_id) ON DELETE CASCADE,
	-- TODO regex this for hex
	-- Nope, these get inserted as argon, so ignore all of this
	two_fa_backup_code TEXT NOT NULL,
	ip_id BIGINT REFERENCES ip_address(ip_id) NOT NULL,
	user_agent_id BIGINT REFERENCES user_agent(user_agent_id) NOT NULL,
	UNIQUE (registered_user_id, two_fa_backup_code)
);

GRANT ALL ON two_fa_backup TO mealpedant;
GRANT USAGE, SELECT ON SEQUENCE two_fa_backup_two_fa_backup_id_seq TO mealpedant;

CREATE TABLE IF NOT EXISTS banned_email_domain (
	banned_email_domain_id BIGINT GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY,
	domain TEXT UNIQUE NOT NULL
);

GRANT ALL ON banned_email_domain TO mealpedant;
GRANT USAGE, SELECT ON SEQUENCE banned_email_domain_banned_email_domain_id_seq TO mealpedant;

CREATE TABLE IF NOT EXISTS email_log (
	email_log_id BIGINT GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY,
	timestamp TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
	ip_id BIGINT REFERENCES ip_address(ip_id) NOT NULL,
	user_agent_id BIGINT REFERENCES user_agent(user_agent_id) NOT NULL,
	-- TODO replace email with email_address_id when using a email table for registered_user_id as well
	-- email_address_id BIGINT REFERENCES email_address(email_address_id) NOT NULL,
	email TEXT NOT NULL,
	registered_user_id BIGINT REFERENCES registered_user(registered_user_id) ON DELETE SET NULL,
	email_title TEXT NOT NULL,
	email_body TEXT NOT NULL
);

GRANT ALL ON email_log TO mealpedant;
GRANT USAGE, SELECT ON SEQUENCE email_log_email_log_id_seq TO mealpedant;

CREATE TABLE IF NOT EXISTS error_log (
	error_log_id BIGINT GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY ,
	timestamp TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
	-- TODO these should be defined error codes
	http_code SMALLINT,
 	level TEXT NOT NULL CHECK (level IN ('warn', 'error','verbose', 'info', 'debug')),
 	message TEXT,
 	stack TEXT,
	uuid TEXT
);

GRANT ALL ON error_log TO mealpedant;
GRANT USAGE, SELECT ON SEQUENCE error_log_error_log_id_seq TO mealpedant;


-- * Function to minus jsonb from jsonb, works with nested jsons as well!
CREATE OR REPLACE FUNCTION jsonb_minus ( arg1 jsonb, arg2 jsonb ) RETURNS jsonb AS $$
SELECT
	COALESCE(json_object_agg(
	key,
	CASE
		WHEN jsonb_typeof(value) = 'object' AND arg2 -> key IS NOT NULL 
			THEN jsonb_minus(value, arg2 -> key)
			ELSE value
		END
	), '{}')::jsonb
FROM
	jsonb_each(arg1)
WHERE
	arg1 -> key <> arg2 -> key 
	OR arg2 -> key IS NULL
$$ LANGUAGE SQL;

	-- create the actual operator, using minux symbol, to use above function
CREATE OPERATOR - (
	PROCEDURE = jsonb_minus,
	LEFTARG = jsonb,
	RIGHTARG = jsonb 
);

CREATE TABLE IF NOT EXISTS registered_user_audit (
	user_audit_id BIGINT GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY,
	table_name TEXT NOT NULL,
	user_name TEXT,
	action_timestamp TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
	action TEXT NOT NULL CHECK (action IN ('i','d','u')),
	old_values jsonb,
	new_values jsonb,
	difference jsonb,
	query TEXT
);

GRANT ALL ON registered_user_audit TO mealpedant;
GRANT USAGE, SELECT ON SEQUENCE registered_user_audit_user_audit_id_seq TO mealpedant;

	-- * create function, to be executed by trigger, to insert into user_audit table
CREATE FUNCTION registered_user_modified_func() RETURNS TRIGGER AS $body$
BEGIN
IF tg_op = 'UPDATE' THEN
	INSERT into registered_user_audit (table_name,	user_name, action, old_values, new_values, difference, query)
	VALUES (tg_table_name::TEXT, current_user, 'u', to_jsonb(OLD), to_jsonb(NEW), to_jsonb(OLD) - to_jsonb(NEW), current_query());
	RETURN new;
ELSIF tg_op = 'DELETE' THEN
	INSERT into registered_user_audit ( table_name, user_name, action, old_values, query)
	VALUES (tg_table_name::TEXT, current_user, 'd', to_jsonb(OLD), current_query());
	RETURN old;
ELSIF tg_op = 'INSERT' THEN
	INSERT into registered_user_audit (table_name, user_name, action, new_values, query)
	VALUES (tg_table_name::TEXT, current_user, 'i', to_jsonb(NEW), current_query());
	RETURN new;
END IF;
END;
$body$
LANGUAGE plpgsql;

	-- * trigger on insert/update/delete on registered_user table, execute if_modified_func()
CREATE TRIGGER user_audit_trig
BEFORE INSERT OR UPDATE OR DELETE
ON registered_user
FOR EACH ROW
EXECUTE PROCEDURE registered_user_modified_func();

CREATE TABLE IF NOT EXISTS individual_meal_audit (
	individual_meal_audit_id BIGINT GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY,
	table_name TEXT NOT NULL,
	user_name TEXT,
	action_timestamp TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
	action TEXT NOT NULL CHECK (action IN ('i','d','u')),
	old_values jsonb,
	new_values jsonb,
	difference jsonb,
	query TEXT
);

GRANT ALL ON individual_meal_audit TO mealpedant;
GRANT USAGE, SELECT ON SEQUENCE individual_meal_individual_meal_id_seq TO mealpedant;

CREATE FUNCTION individual_meal_modifiy_func() RETURNS TRIGGER AS $body$
BEGIN
IF tg_op = 'UPDATE' THEN
	INSERT into individual_meal_audit (table_name,	user_name, action, old_values, new_values, difference, query)
	VALUES (tg_table_name::TEXT, current_user, 'u', to_jsonb(OLD), to_jsonb(NEW), to_jsonb(OLD) - to_jsonb(NEW), current_query());
	RETURN new;
ELSIF tg_op = 'DELETE' THEN
	INSERT into individual_meal_audit ( table_name, user_name, action, old_values, query)
	VALUES (tg_table_name::TEXT, current_user, 'd', to_jsonb(OLD), current_query());
	RETURN old;
ELSIF tg_op = 'INSERT' THEN
	INSERT into individual_meal_audit (table_name, user_name, action, new_values, query)
	VALUES (tg_table_name::TEXT, current_user, 'i', to_jsonb(NEW), current_query());
	RETURN new;
END IF;
END;
$body$
LANGUAGE plpgsql;

CREATE TRIGGER individual_meal_audit_trig
BEFORE INSERT OR UPDATE OR DELETE
ON individual_meal
FOR EACH ROW
EXECUTE PROCEDURE individual_meal_modifiy_func();

/**
** New tables
*/

CREATE TABLE IF NOT EXISTS meal_category_audit (
	meal_category_audit_id BIGINT GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY,
	table_name TEXT NOT NULL,
	user_name TEXT,
	action_timestamp TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
	action TEXT NOT NULL CHECK (action IN ('i','d','u')),
	old_values jsonb,
	new_values jsonb,
	difference jsonb,
	query TEXT
);

GRANT ALL ON meal_category_audit TO mealpedant;
GRANT USAGE, SELECT ON SEQUENCE meal_category_audit_meal_category_audit_id_seq TO mealpedant;

CREATE FUNCTION meal_category_modify_func() RETURNS TRIGGER AS $body$
BEGIN
IF tg_op = 'UPDATE' THEN
	INSERT into meal_category_audit (table_name,	user_name, action, old_values, new_values, difference, query)
	VALUES (tg_table_name::TEXT, current_user, 'u', to_jsonb(OLD), to_jsonb(NEW), to_jsonb(OLD) - to_jsonb(NEW), current_query());
	RETURN new;
ELSIF tg_op = 'DELETE' THEN
	INSERT into meal_category_audit ( table_name, user_name, action, old_values, query)
	VALUES (tg_table_name::TEXT, current_user, 'd', to_jsonb(OLD), current_query());
	RETURN old;
ELSIF tg_op = 'INSERT' THEN
	INSERT into meal_category_audit (table_name, user_name, action, new_values, query)
	VALUES (tg_table_name::TEXT, current_user, 'i', to_jsonb(NEW), current_query());
	RETURN new;
END IF;
END;
$body$
LANGUAGE plpgsql;

CREATE TRIGGER meal_category_audit_trig
BEFORE INSERT OR UPDATE OR DELETE
ON meal_category
FOR EACH ROW
EXECUTE PROCEDURE meal_category_modify_func();

-- meal date audit
CREATE TABLE IF NOT EXISTS meal_date_audit (
	meal_date_audit_id BIGINT GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY,
	table_name TEXT NOT NULL,
	user_name TEXT,
	action_timestamp TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
	action TEXT NOT NULL CHECK (action IN ('i','d','u')),
	old_values jsonb,
	new_values jsonb,
	difference jsonb,
	query TEXT
);
GRANT ALL ON meal_date_audit TO mealpedant;
GRANT USAGE, SELECT ON SEQUENCE meal_date_audit_meal_date_audit_id_seq TO mealpedant;

CREATE FUNCTION meal_date_modify_func() RETURNS TRIGGER AS $body$
BEGIN
IF tg_op = 'UPDATE' THEN
	INSERT into meal_date_audit (table_name,	user_name, action, old_values, new_values, difference, query)
	VALUES (tg_table_name::TEXT, current_user, 'u', to_jsonb(OLD), to_jsonb(NEW), to_jsonb(OLD) - to_jsonb(NEW), current_query());
	RETURN new;
ELSIF tg_op = 'DELETE' THEN
	INSERT into meal_date_audit ( table_name, user_name, action, old_values, query)
	VALUES (tg_table_name::TEXT, current_user, 'd', to_jsonb(OLD), current_query());
	RETURN old;
ELSIF tg_op = 'INSERT' THEN
	INSERT into meal_date_audit (table_name, user_name, action, new_values, query)
	VALUES (tg_table_name::TEXT, current_user, 'i', to_jsonb(NEW), current_query());
	RETURN new;
END IF;
END;
$body$
LANGUAGE plpgsql;

CREATE TRIGGER meal_date_audit_trig
BEFORE INSERT OR UPDATE OR DELETE
ON meal_date
FOR EACH ROW
EXECUTE PROCEDURE meal_date_modify_func();

-- description audit
CREATE TABLE IF NOT EXISTS meal_description_audit (
	meal_description_audit_id BIGINT GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY,
	table_name TEXT NOT NULL,
	user_name TEXT,
	action_timestamp TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
	action TEXT NOT NULL CHECK (action IN ('i','d','u')),
	old_values jsonb,
	new_values jsonb,
	difference jsonb,
	query TEXT
);
GRANT ALL ON meal_description_audit TO mealpedant;
GRANT USAGE, SELECT ON SEQUENCE meal_description_audit_meal_description_audit_id_seq TO mealpedant;

CREATE FUNCTION meal_description_modify_func() RETURNS TRIGGER AS $body$
BEGIN
IF tg_op = 'UPDATE' THEN
	INSERT into meal_description_audit (table_name,	user_name, action, old_values, new_values, difference, query)
	VALUES (tg_table_name::TEXT, current_user, 'u', to_jsonb(OLD), to_jsonb(NEW), to_jsonb(OLD) - to_jsonb(NEW), current_query());
	RETURN new;
ELSIF tg_op = 'DELETE' THEN
	INSERT into meal_description_audit ( table_name, user_name, action, old_values, query)
	VALUES (tg_table_name::TEXT, current_user, 'd', to_jsonb(OLD), current_query());
	RETURN old;
ELSIF tg_op = 'INSERT' THEN
	INSERT into meal_description_audit (table_name, user_name, action, new_values, query)
	VALUES (tg_table_name::TEXT, current_user, 'i', to_jsonb(NEW), current_query());
	RETURN new;
END IF;
END;
$body$
LANGUAGE plpgsql;

CREATE TRIGGER meal_description_audit_trig
BEFORE INSERT OR UPDATE OR DELETE
ON meal_description
FOR EACH ROW
EXECUTE PROCEDURE meal_description_modify_func();

-- meal photo audit 
CREATE TABLE IF NOT EXISTS meal_photo_audit (
	meal_photo_audit_id BIGINT GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY,
	table_name TEXT NOT NULL,
	user_name TEXT,
	action_timestamp TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
	action TEXT NOT NULL CHECK (action IN ('i','d','u')),
	old_values jsonb,
	new_values jsonb,
	difference jsonb,
	query TEXT
);
GRANT ALL ON meal_photo_audit TO mealpedant;
GRANT USAGE, SELECT ON SEQUENCE meal_photo_audit_meal_photo_audit_id_seq TO mealpedant;

CREATE FUNCTION meal_photo_modify_func() RETURNS TRIGGER AS $body$
BEGIN
IF tg_op = 'UPDATE' THEN
	INSERT into meal_photo_audit (table_name,	user_name, action, old_values, new_values, difference, query)
	VALUES (tg_table_name::TEXT, current_user, 'u', to_jsonb(OLD), to_jsonb(NEW), to_jsonb(OLD) - to_jsonb(NEW), current_query());
	RETURN new;
ELSIF tg_op = 'DELETE' THEN
	INSERT into meal_photo_audit ( table_name, user_name, action, old_values, query)
	VALUES (tg_table_name::TEXT, current_user, 'd', to_jsonb(OLD), current_query());
	RETURN old;
ELSIF tg_op = 'INSERT' THEN
	INSERT into meal_photo_audit (table_name, user_name, action, new_values, query)
	VALUES (tg_table_name::TEXT, current_user, 'i', to_jsonb(NEW), current_query());
	RETURN new;
END IF;
END;
$body$
LANGUAGE plpgsql;

CREATE TRIGGER meal_photo_audit_trig
BEFORE INSERT OR UPDATE OR DELETE
ON meal_photo
FOR EACH ROW
EXECUTE PROCEDURE meal_photo_modify_func();


CREATE DATABASE dev_mealpedant WITH TEMPLATE mealpedant OWNER jack;
GRANT ALL PRIVILEGES ON DATABASE dev_mealpedant TO mealpedant;