#!/bin/bash

create_mealpedant_database() {
	echo "create_mealpedant_database"
	psql -v ON_ERROR_STOP=0 -U "$POSTGRES_USER" -d "$POSTGRES_USER" <<-EOSQL
		CREATE DATABASE ${DB_NAME};
	EOSQL
}

create_mealpedant_user() {
	echo "create_mealpedant_user"
	psql -v ON_ERROR_STOP=0 -U "$POSTGRES_USER" -d "$POSTGRES_USER" <<-EOSQL
		CREATE ROLE ${DB_USER} WITH LOGIN PASSWORD '$DB_PASSWORD';
	EOSQL
}

restore_pg_dump() {
	echo "restore_pg_dump"
	pg_restore -U "$POSTGRES_USER" -O --exit-on-error --single-transaction -d "$DB_NAME" -v /init/pg_dump.tar
	psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$DB_NAME" <<-EOSQL
		GRANT SELECT ON ALL SEQUENCES IN SCHEMA public TO $DB_NAME;
		GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO $DB_NAME;
	EOSQL
}


update_banned_domains() {
	echo "update_banned_domains"
	psql -v ON_ERROR_STOP=0 --username "$POSTGRES_USER" --dbname "$DB_NAME"  <<-EOSQL
		DELETE FROM banned_email_domain;
		COPY banned_email_domain (domain) FROM '/init/banned_domains.txt';
		GRANT USAGE, SELECT ON SEQUENCE banned_domain_banned_domain_id_seq TO $DB_NAME;
		GRANT ALL ON banned_email_domain TO $DB_NAME;
		GRANT SELECT ON ALL SEQUENCES IN SCHEMA public TO $DB_NAME;
		GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO $DB_NAME;
	EOSQL
}

# Run any & all migrations
run_migrations() {
	if ! psql -v ON_ERROR_STOP=0 -U "$POSTGRES_USER" -d "$DB_NAME" --port "${DOCKER_PG_PORT}" -f "/init/migrations.sql"; then
		echo "Error: Failed to run migrations.sql" >&2
		exit 1
	fi
}

# Create db from .sql file, requires other data (*.csv etc) to read to build
bootstrap_from_sql_file() {
	psql -U "${POSTGRES_USER}" -d "${POSTGRES_USER}" -f /init/init_db.sql
}

from_pg_dump() {
	create_mealpedant_user
	create_mealpedant_database
	restore_pg_dump
}

from_scratch() {
	create_mealpedant_user
	bootstrap_from_sql_file
}

create_tables() {
	if [ -f "/init/pg_dump.tar" ]; then
		from_pg_dump
	else
		from_scratch
	fi
}

main() {
	if [ "$1" == "migrations" ]; then
		run_migrations
	else
		create_tables
	fi
	update_banned_domains
}

main "$1"
