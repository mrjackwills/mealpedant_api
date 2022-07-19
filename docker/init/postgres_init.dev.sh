#!/bin/sh -x
set -e

# Should really use wait-for here
# sleep 1
DEV_DB=dev_${DB_NAME}


create_mealpedant_user() {
	echo "create_mealpedant_user"
	# ON_ERROR_STOP=1
	psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "postgres" <<-EOSQL
CREATE ROLE $DB_USER WITH LOGIN PASSWORD '$DB_PASSWORD';
CREATE DATABASE $DB_NAME;
EOSQL
}

restore_pg_dump() {
	echo "pg_restore"
	# Should use .sql instead, and then restore from backup? That way can make sure tables are setup correctly, and can also handle any migrations in an .sql file
	pg_restore -U "$POSTGRES_USER" -O --exit-on-error --single-transaction -d "$DB_NAME" -v /docker-entrypoint-initdb.d/pg_dump.tar
	psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$DB_NAME" <<-EOSQL
GRANT SELECT ON ALL SEQUENCES IN SCHEMA public TO $DB_NAME;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO $DB_NAME;
EOSQL
}

add_dev_mealpedant() {
	echo "creating dev_mealpedant"
	createdb -U "$POSTGRES_USER" -O "$POSTGRES_USER" -T "$DB_NAME" "$DEV_DB"

	echo "granting access on mealpedant to mealpedant"
	psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$DB_NAME" <<-EOSQL
GRANT SELECT ON ALL SEQUENCES IN SCHEMA public TO $DB_NAME;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO $DB_NAME;
EOSQL

	echo "granting access on dev_mealpedant to mealpedant"
	psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$DEV_DB" <<-EOSQL
GRANT SELECT ON ALL SEQUENCES IN SCHEMA public TO $DB_NAME;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO $DB_NAME;
EOSQL
}

banned_domains () {
	psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$DB_NAME" <<-EOSQL
DELETE FROM banned_email_domain;
COPY banned_email_domain (domain) FROM '/docker-entrypoint-initdb.d/banned_domains.txt';
EOSQL
}


main () {
	echo "IN MAIN"
	# DB_EXIST=$(psql -U "$POSTGRES_USER" -lu | grep "$DB_NAME")
	# if [ ! "$DB_EXIST" ]
	# then
		create_mealpedant_user
		restore_pg_dump
		banned_domains
		add_dev_mealpedant
	# fi
}

main



