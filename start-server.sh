#!/usr/bin/env bash

# Set DOCKER_HOST if needed
#DOCKER_HOST=ssh://remote_host

docker-compose down
docker-compose rm -v
docker-compose pull
docker-compose up -d --build
