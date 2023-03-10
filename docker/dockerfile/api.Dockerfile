FROM ubuntu:22.04

ARG DOCKER_GUID=1000 \
	DOCKER_UID=1000 \
	DOCKER_TIME_CONT=America \
	DOCKER_TIME_CITY=New_York \
	DOCKER_APP_USER=app_user \
	DOCKER_APP_GROUP=app_group

ENV TZ=${DOCKER_TIME_CONT}/${DOCKER_TIME_CITY}

RUN apt-get update \
	&& apt-get install -y apt-utils ca-certificates wget age gnupg \
	&& update-ca-certificates \
	&& sh -c 'echo "deb http://apt.postgresql.org/pub/repos/apt jammy-pgdg main" > /etc/apt/sources.list.d/pgdg.list' \
	&& wget --quiet -O - https://www.postgresql.org/media/keys/ACCC4CF8.asc | apt-key add - \
 	&& apt-get update \
	&& apt-get -y install postgresql-client-15 \
	&& groupadd --gid ${DOCKER_GUID} ${DOCKER_APP_GROUP} \
	&& useradd --create-home --no-log-init --uid ${DOCKER_UID} --gid ${DOCKER_GUID} ${DOCKER_APP_USER} \
	&& mkdir /backups /logs /static /photo_original /photo_converted \
	&& chown ${DOCKER_APP_USER}:${DOCKER_APP_GROUP} /backups /logs /static /photo_original /photo_converted
	
WORKDIR /app

COPY --chown=${DOCKER_APP_USER}:${DOCKER_APP_GROUP} ./docker/healthcheck/health_api.sh /healthcheck/
RUN chmod +x /healthcheck/health_api.sh

COPY --chown=${DOCKER_APP_USER}:${DOCKER_APP_GROUP} ./docker/data/watermark.png /app

# Download latest release from github
RUN wget https://github.com/mrjackwills/mealpedant_api/releases/download/v1.3.1/mealpedant_linux_x86_64.tar.gz \
	&& tar xzvf mealpedant_linux_x86_64.tar.gz mealpedant \
	&& rm mealpedant_linux_x86_64.tar.gz \
	&& chown ${DOCKER_APP_USER}:${DOCKER_APP_GROUP} mealpedant

USER ${DOCKER_APP_USER}

CMD ["/app/mealpedant"]