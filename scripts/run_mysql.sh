#!/usr/bin/env bash
set -x
set -eo pipefail

# if a redis container is running, print instructions to kill it and exit
RUNNING_CONTAINER=$(docker ps --filter 'name=mysql' --format '{{.ID}}')
if [[ -n $RUNNING_CONTAINER ]]; then
  echo >&2 "there is a MySQL instance already running, kill it with"
  echo >&2 "    docker kill ${RUNNING_CONTAINER}"
  exit 1
fi

# Launch MySQL using Docker
docker run \
    -p 3306:3306 \
    -d \
    -e MYSQL_DATABASE=tonsail -e MYSQL_ROOT_PASSWORD=root \
    --name "mysql_$(date '+%s')" \
    mysql:8

>&2 echo "MySQL is ready to go!"
