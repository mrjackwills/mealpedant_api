FROM redis:alpine

ARG DOCKER_GUID=1000 \
	DOCKER_UID=1000 \
	DOCKER_APP_USER=app_user \
	DOCKER_APP_GROUP=app_group

RUN deluser redis \
	&& addgroup -g ${DOCKER_GUID} -S ${DOCKER_APP_GROUP} \
	&& adduser -u ${DOCKER_UID} -S -G ${DOCKER_APP_GROUP} ${DOCKER_APP_USER} \
	&& mkdir /redis_logs /redis_data /healthcheck \
	&& touch /redis_logs/redis-server.log \
	&& chown -R ${DOCKER_APP_USER}:${DOCKER_APP_GROUP} /redis_logs /redis_data /healthcheck

WORKDIR /

USER ${DOCKER_APP_USER}

COPY --chown=${DOCKER_APP_USER}:${DOCKER_APP_GROUP} docker/healthcheck/health_redis.sh /healthcheck/

RUN chmod +x /healthcheck/health_redis.sh

ENTRYPOINT [ "sh", "-c", "exec redis-server --bind mealpedant_redis --port \"$DOCKER_REDIS_PORT\" --pidfile \"/var/run/redis_${DOCKER_REDIS_PORT}.pid\" --logfile /redis_logs/redis-server.log --save 60 1 --dir /redis_data --repl-diskless-sync no --requirepass \"$DOCKER_REDIS_PASSWORD\"" ]
