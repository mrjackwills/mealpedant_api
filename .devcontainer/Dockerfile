# See here for image contents: https://github.com/microsoft/vscode-dev-containers/tree/v0.202.1/containers/typescript-node/.devcontainer/base.Dockerfile

ARG VARIANT="buster"
FROM mcr.microsoft.com/vscode/devcontainers/rust:0-${VARIANT}

RUN printf "alias cls='clear'\nalias ll='ls -l --human-readable --color=auto --group-directories-first --classify --time-style=long-iso -all'" >> /etc/bash.bashrc

RUN sh -c 'echo "deb http://apt.postgresql.org/pub/repos/apt $(lsb_release -cs)-pgdg main" > /etc/apt/sources.list.d/pgdg.list'
RUN wget --quiet -O - https://www.postgresql.org/media/keys/ACCC4CF8.asc | sudo apt-key add -
RUN apt-get update && apt-get -y install postgresql-client-14 && apt-get upgrade -y

# COPY --chown=vscode:vscode ../docker/env/.pgpass /home/vscode/.pgpass

# RUN chmod 600 /home/vscode/.pgpass

# Instal pg_dump
