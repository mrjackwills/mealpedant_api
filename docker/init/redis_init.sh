#!/bin/sh
set -e

sed -i "s|requirepass replace_me|requirepass ${DOCKER_REDIS_PASSWORD}|" /init/redis.conf

exec "$@"