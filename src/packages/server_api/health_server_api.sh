#!/bin/sh
set -e

wget -nv -t1 --spider "${API_HOSTNAME}:${API_PORT}/v0/incognito/online" || exit 1