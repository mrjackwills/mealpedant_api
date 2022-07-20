#!/bin/sh
set -e

####################
# SET UP VARIABLES #
####################

# current time, for file naming
TIMESTAMP=$(date +%F_%H.%M.%S)

# temp folder name is random uuid
UUID_DIR_NAME=$(cat /proc/sys/kernel/random/uuid)
FILE_SUFFIX=$(echo "$UUID_DIR_NAME" | head -c 8)
TEMP_DIR_NAME="/tmp/${FILE_SUFFIX}"

# # Final filename
FINAL_OUTPUT_NAME="mealpedant_${TIMESTAMP}_LOGS_PHOTOS_REDIS_SQL_${FILE_SUFFIX}.tar.gpg"

# Move into temp directory
cd "$LOCATION_BACKUPS" || exit 1

# Create tmp dir using random string
mkdir "$TEMP_DIR_NAME"

# Tar gz all .log files in log directory logs into ./random_name/logs.tar.gz
tar -C "$LOCATION_ALL_LOGS" -cf "$TEMP_DIR_NAME/logs.tar" ./
tar -C "$LOCATION_REDIS" -cf "$TEMP_DIR_NAME/redis.tar" ./
tar -C "$LOCATION_STATIC" -cf "$TEMP_DIR_NAME/static.tar" ./


# Dump mealpedant database into a tar in tmp folder
pg_dump -U "$PG_USER" -d "$PG_DATABASE" -h "$PG_HOST" -p 5434 --no-owner --format=t > "$TEMP_DIR_NAME/pg_dump.tar"

# Create photo tars, no need to gzip

# Put logs.tar.gz, pg_dump.tar.gz, and photos.tar into combined.tar
tar -C "$TEMP_DIR_NAME" -cf "$TEMP_DIR_NAME/combined.tar" logs.tar pg_dump.tar redis.tar

# gzip here pointless?
gzip "$TEMP_DIR_NAME/combined.tar"

tar -C "$TEMP_DIR_NAME" -cf "$TEMP_DIR_NAME/all.tar" combined.tar.gz static.tar

# Encrypt data, -z 0 means no compression, pointless for the photos
gpg --output "$LOCATION_BACKUPS/$FINAL_OUTPUT_NAME" --batch -z 0 --passphrase "$GPG_PASSWORD" -c "$TEMP_DIR_NAME/all.tar"
chmod 440 "$LOCATION_BACKUPS/$FINAL_OUTPUT_NAME"

# Remove tmp dir
rm -rf "$TEMP_DIR_NAME"

# remove backup files that are older than 6 days
find "$LOCATION_BACKUPS" -type f -name '*.gpg' -mtime +6 -delete

exit 0