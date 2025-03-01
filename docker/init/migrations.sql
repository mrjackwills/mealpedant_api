\echo "Migrations"

\echo "meal_photo originals to .jpg"
UPDATE meal_photo
SET photo_original = REPLACE(photo_original, '.jpeg', '.jpg')
WHERE photo_original LIKE '%.jpeg';

\echo "meal_photo converted to .jpg"
UPDATE meal_photo
SET photo_converted = REPLACE(photo_converted, '.jpeg', '.jpg')
WHERE photo_converted LIKE '%.jpeg';

\echo "meal_photo original uppercase"
UPDATE meal_photo
SET photo_original = UPPER(SUBSTRING(photo_original FROM 1 FOR LENGTH(photo_original) - 4)) || RIGHT(photo_original, 4)
WHERE photo_original LIKE '%.%';

\echo "meal_photo converted uppercase"
UPDATE meal_photo
SET photo_converted = UPPER(SUBSTRING(photo_converted FROM 1 FOR LENGTH(photo_converted) - 4)) || RIGHT(photo_converted, 4)
WHERE photo_converted LIKE '%.%';

\echo "meal_photo remove unused photos"
DELETE FROM meal_photo mp
WHERE NOT EXISTS (
	SELECT *
	FROM individual_meal im
	WHERE im.meal_photo_id= mp.meal_photo_id
);