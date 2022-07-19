#!/bin/bash
set -e

create_mealpedant_user_and_db() {
	echo "create_mealpedant_user_and_db"
	psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "postgres" <<-EOSQL
CREATE ROLE $DB_NAME WITH LOGIN PASSWORD '$DB_PASSWORD';
CREATE DATABASE $DB_NAME;
EOSQL
}

restore_pg_dump() {
	echo "restore_pg_dump"
	# --single-transaction?
	pg_restore -U "$POSTGRES_USER" -O --exit-on-error --single-transaction -d "$DB_NAME" -v /docker-entrypoint-initdb.d/pg_dump.tar
	psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$DB_NAME" <<-EOSQL
GRANT SELECT ON ALL SEQUENCES IN SCHEMA public TO $DB_NAME;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO $DB_NAME;
EOSQL
}

update_banned_domains () {
	echo "update_banned_domains"
	psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$DB_NAME" <<-EOSQL
COPY banned_email_domain (domain) FROM '/docker-entrypoint-initdb.d/banned_domains.txt';
DELETE FROM banned_email_domain;
GRANT USAGE, SELECT ON SEQUENCE banned_email_domain_banned_email_domain_id_seq TO $DB_NAME;
GRANT ALL ON banned_email_domain TO $DB_NAME;
GRANT SELECT ON ALL SEQUENCES IN SCHEMA public TO $DB_NAME;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO $DB_NAME;
EOSQL
}

main () {
	create_mealpedant_user_and_db
	restore_pg_dump
	update_banned_domains
}

main