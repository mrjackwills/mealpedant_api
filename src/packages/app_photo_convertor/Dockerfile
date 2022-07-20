FROM node:16-alpine3.16

ARG DOCKER_GUID=1000 \
	DOCKER_UID=1000 \
	DOCKER_TIME_CONT=America \
	DOCKER_TIME_CITY=New_York \
	DOCKER_APP_USER=app_user \
	DOCKER_APP_GROUP=app_group

ENV TZ=${DOCKER_TIME_CONT}/${DOCKER_TIME_CITY}

RUN deluser --remove-home node \
	&& addgroup -g ${DOCKER_GUID} -S ${DOCKER_APP_GROUP} \
	&& adduser -u ${DOCKER_UID} -S -G ${DOCKER_APP_GROUP} ${DOCKER_APP_USER} \
	&& apk --no-cache add tzdata \
	&& cp /usr/share/zoneinfo/${TZ} /etc/localtime \
	&& echo ${TZ} > /etc/timezone

RUN npm install -g npm@latest

WORKDIR /app

RUN mkdir /logs /photo_converted /photo_original \
 && chown ${DOCKER_APP_USER}:${DOCKER_APP_GROUP} /app /logs /photo_converted /photo_original

USER ${DOCKER_APP_USER}

COPY --chown=${DOCKER_APP_USER}:${DOCKER_APP_GROUP} images /images

COPY --chown=${DOCKER_APP_USER}:${DOCKER_APP_GROUP} package*.json tsconfig*.json .eslintignore .eslintrc.js ./

RUN npm install

COPY --chown=${DOCKER_APP_USER}:${DOCKER_APP_GROUP} src /app/src

RUN npm run build \
	&& npm prune --production \
	&& npm cache clean --force

CMD [ "node", "dist/index.js"]