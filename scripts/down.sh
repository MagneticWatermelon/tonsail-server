#!/usr/bin/env bash
#

# if a MySQL container is running, kill it
MYSQL_INSTANCE=$(docker ps --filter 'name=mysql' --format '{{.ID}}')
if [[ -n $MYSQL_INSTANCE ]]; then
  echo >&2 "MySQL instance ${MYSQL_INSTANCE} found, killing it"
  docker kill ${MYSQL_INSTANCE}
fi

# if a QuestDB container is running, kill it
QUESTDB_INSTANCE=$(docker ps --filter 'name=questdb' --format '{{.ID}}')
if [[ -n $QUESTDB_INSTANCE ]]; then
  echo >&2 "QuestDB instance ${QUESTDB_INSTANCE} found, killing it"
  docker kill ${QUESTDB_INSTANCE}
fi

# if a Redis container is running, kill it
REDIS_INSTANCE=$(docker ps --filter 'name=redis' --format '{{.ID}}')
if [[ -n $REDIS_INSTANCE ]]; then
  echo >&2 "MySQL instance ${REDIS_INSTANCE} found, killing it"
  docker kill ${REDIS_INSTANCE}
fi
