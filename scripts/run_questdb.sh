#!/usr/bin/env bash
set -eo pipefail

# if a QuestDB container is running, print instructions to kill it and exit
RUNNING_CONTAINER=$(docker ps --filter 'name=questdb' --format '{{.ID}}')
if [[ -n $RUNNING_CONTAINER ]]; then
  echo >&2 "there is a QuestDB instance already running, kill it with"
  echo >&2 "    docker kill ${RUNNING_CONTAINER}"
  exit 1
fi

# Launch QuestDB using Docker
docker run \
    -p 9000:9000 -p 9009:9009 -p 8812:8812 -p 9003:9003\
    -d \
    --name "questdb_$(date '+%s')" \
    questdb/questdb:7.0.1

>&2 echo "QuestDB is ready to go!"
