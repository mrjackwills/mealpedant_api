-- Remove first_name, last_name, with full_name
ALTER TABLE
	registered_user
ADD
	COLUMN full_name TEXT;

DO $ $ DECLARE temp_user_row record;

BEGIN for temp_user_row IN
SELECT
	*
FROM
	registered_user LOOP
UPDATE
	registered_user
SET
	full_name = concat(
		temp_user_row.first_name,
		' ',
		temp_user_row.last_name
	)
WHERE
	registered_user_id = temp_user_row.registered_user_id;

END LOOP;

END;

$ $;

ALTER TABLE
	registered_user
ALTER column
	full_name
SET
	NOT NULL;

ALTER TABLE
	registered_user DROP column first_name;

ALTER TABLE
	registered_user DROP column last_name;