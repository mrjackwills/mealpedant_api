FROM postgres:17-alpine3.21

ARG DOCKER_GUID=1000 \
	DOCKER_UID=1000 \
	DOCKER_APP_USER=app_user \
	DOCKER_APP_GROUP=app_group

RUN addgroup -g ${DOCKER_GUID} -S ${DOCKER_APP_GROUP} \
	&& adduser -u ${DOCKER_UID} -S -G ${DOCKER_APP_GROUP} ${DOCKER_APP_USER} \
	&& mkdir /pg_data /init /healthcheck \
	&& chown -R ${DOCKER_APP_USER}:postgres /pg_data \
	&& chown -R ${DOCKER_APP_USER}:${DOCKER_APP_GROUP} /init /healthcheck

COPY --chown=${DOCKER_APP_USER}:${DOCKER_APP_GROUP} docker/data/pg_dump.tar* docker/init/migrations.sql docker/data/banned_domains.txt /init/

COPY --chown=${DOCKER_APP_USER}:${DOCKER_APP_GROUP} docker/healthcheck/health_postgres.sh /healthcheck/

COPY --chown=${DOCKER_APP_USER}:${DOCKER_APP_GROUP} docker/init/init_postgres.sh /docker-entrypoint-initdb.d/

RUN chmod +x /healthcheck/health_postgres.sh /docker-entrypoint-initdb.d/init_postgres.sh

USER ${DOCKER_APP_USER}