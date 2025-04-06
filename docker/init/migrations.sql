\echo "Migrations"

\echo "meal_photo originals to .jpg"
UPDATE meal_photo
SET photo_original = REPLACE(photo_original, '.jpeg', '.jpg')
WHERE photo_original LIKE '%.jpeg';

\echo "meal_photo converted to .jpg"
UPDATE meal_photo
SET photo_converted = REPLACE(photo_converted, '.jpeg', '.jpg')
WHERE photo_converted LIKE '%.jpeg';

\echo "meal_photo remove unused photos"
DELETE FROM meal_photo mp
WHERE NOT EXISTS (
	SELECT *
	FROM individual_meal im
	WHERE im.meal_photo_id= mp.meal_photo_id
);

\echo "login_attempt registered_user_id NOT NULL"
ALTER TABLE login_attempt 
ALTER COLUMN registered_user_id SET NOT NULL;

\echo "login_attempt registered_user_id NOT NULL"
ALTER TABLE login_attempt 
ALTER COLUMN login_attempt_number SET NOT NULL;

\echo "registered_user active NOT NULL"
ALTER TABLE registered_user 
ALTER COLUMN active SET NOT NULL;