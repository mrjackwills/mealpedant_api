# #############
# ## Builder ##
# #############

FROM rust:slim AS builder

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

#############
## Runtime ##
#############

FROM ubuntu:22.04 AS runtime

ARG DOCKER_GUID=1000 \
	DOCKER_UID=1000 \
	DOCKER_APP_USER=app_user \
	DOCKER_APP_GROUP=app_group

RUN apt-get update \
	&& apt-get install -y apt-utils ca-certificates wget age gnupg \
	&& update-ca-certificates \
	&& sh -c 'echo "deb https://apt.postgresql.org/pub/repos/apt jammy-pgdg main" > /etc/apt/sources.list.d/pgdg.list' \
	&& wget --quiet -O - https://www.postgresql.org/media/keys/ACCC4CF8.asc | apt-key add - \
 	&& apt-get update \
	&& apt-get -y install postgresql-client-16 \
	&& groupadd --gid ${DOCKER_GUID} ${DOCKER_APP_GROUP} \
	&& useradd --create-home --no-log-init --uid ${DOCKER_UID} --gid ${DOCKER_GUID} ${DOCKER_APP_USER} \
	&& mkdir /backups /logs /static /photo_original /photo_converted \
	&& chown ${DOCKER_APP_USER}:${DOCKER_APP_GROUP} /backups /logs /static /photo_original /photo_converted
	
WORKDIR /app

COPY --chown=${DOCKER_APP_USER}:${DOCKER_APP_GROUP} ./docker/healthcheck/health_api.sh /healthcheck/
RUN chmod +x /healthcheck/health_api.sh

COPY --chown=${DOCKER_APP_USER}:${DOCKER_APP_GROUP} ./docker/data/watermark.png /app

COPY --from=builder /usr/src/mealpedant/target/release/mealpedant /app/

# Copy from host filesystem - used when debugging
# COPY --chown=${DOCKER_APP_USER}:${DOCKER_APP_GROUP} target/release/mealpedant /app

USER ${DOCKER_APP_USER}

CMD ["/app/mealpedant"]