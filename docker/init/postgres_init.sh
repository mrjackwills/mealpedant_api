#!/bin/bash
set -e

create_mealpedant_user_and_db() {
	printf "\ncreate_mealpedant_user_and_db\n\n"
	psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "postgres" <<-EOSQL
CREATE ROLE $DB_NAME WITH LOGIN PASSWORD '$DB_PASSWORD';
CREATE DATABASE $DB_NAME;
EOSQL
}

restore_pg_dump() {
	printf "\nrestore_pg_dump\n\n"
	# --single-transaction?
	pg_restore -U "$POSTGRES_USER" -O --exit-on-error --single-transaction -d "$DB_NAME" -v /docker-entrypoint-initdb.d/pg_dump.tar
	psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$DB_NAME" <<-EOSQL
GRANT SELECT ON ALL SEQUENCES IN SCHEMA public TO $DB_NAME;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO $DB_NAME;
EOSQL
}

update_banned_domains () {
	printf "\nupdate_banned_domains\n\n"
	psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$DB_NAME" <<-EOSQL
DELETE FROM banned_email_domain;
COPY banned_email_domain (domain) FROM '/docker-entrypoint-initdb.d/banned_domains.txt';
GRANT USAGE, SELECT ON SEQUENCE banned_domain_banned_domain_id_seq TO $DB_NAME;
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