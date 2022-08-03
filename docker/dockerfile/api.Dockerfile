# use ubuntu slim?
FROM alpine:3.16

ARG DOCKER_GUID=1000 \
	DOCKER_UID=1000 \
	DOCKER_TIME_CONT=America \
	DOCKER_TIME_CITY=New_York \
	DOCKER_APP_USER=app_user \
	DOCKER_APP_GROUP=app_group

ENV TZ=${DOCKER_TIME_CONT}/${DOCKER_TIME_CITY}

RUN apk add --update --no-cache tzdata gnupg postgresql-client \
	&& cp /usr/share/zoneinfo/${TZ} /etc/localtime \
	&& echo ${TZ} > /etc/timezone \
	&& addgroup -g ${DOCKER_GUID} -S ${DOCKER_APP_GROUP} \
	&& adduser -u ${DOCKER_UID} -S -G ${DOCKER_APP_GROUP} ${DOCKER_APP_USER} \
	&& mkdir /backups /logs /static /photo_original /photo_converted \
	&& chown ${DOCKER_APP_USER}:${DOCKER_APP_GROUP} /backups /logs /static /photo_original /photo_converted
	
WORKDIR /app

COPY --chown=${DOCKER_APP_USER}:${DOCKER_APP_GROUP} ./docker/healthcheck/health_api.sh /healthcheck/
RUN chmod +x /healthcheck/health_api.sh

COPY --chown=${DOCKER_APP_USER}:${DOCKER_APP_GROUP} ./docker/data/watermark.png /app

# Download latest release from github
RUN wget https://github.com/mrjackwills/mealpedant_api/releases/download/v1.0.2/mealpedant_linux_x86_64_musl.tar.gz \
	&& tar xzvf mealpedant_linux_x86_64_musl.tar.gz mealpedant \
	&& rm mealpedant_linux_x86_64_musl.tar.gz \
	&& chown ${DOCKER_APP_USER}:${DOCKER_APP_GROUP} mealpedant

USER ${DOCKER_APP_USER}

CMD ["/app/mealpedant"]