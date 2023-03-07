#############
## Builder ##
#############

FROM rust:slim as BUILDER

WORKDIR /usr/src

# Create blank project
RUN cargo new mealpedant

# We want dependencies cached, so copy those first
COPY Cargo.* /usr/src/mealpedant/

# Set the working directory
WORKDIR /usr/src/mealpedant

# This is a dummy build to get the dependencies cached - probably not needed - as run via a github action
RUN cargo build --release

# Now copy in the rest of the sources
COPY src /usr/src/mealpedant/src/

## Touch main.rs to prevent cached release build
RUN touch /usr/src/mealpedant/src/main.rs

# This is the actual application build
RUN cargo build --release

RUN cp /usr/src/mealpedant/target/release/mealpedant /

#############
## Runtime ##
#############

FROM ubuntu:22.04 AS RUNTIME

ARG DOCKER_GUID=1000 \
	DOCKER_UID=1000 \
	DOCKER_TIME_CONT=America \
	DOCKER_TIME_CITY=New_York \
	DOCKER_APP_USER=app_user \
	DOCKER_APP_GROUP=app_group

ENV TZ=${DOCKER_TIME_CONT}/${DOCKER_TIME_CITY}

RUN apt-get update \
	&& apt-get install -y ca-certificates wget age gnupg \
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

COPY --from=BUILDER /usr/src/mealpedant/target/release/mealpedant /app/

USER ${DOCKER_APP_USER}

CMD ["/app/mealpedant"]