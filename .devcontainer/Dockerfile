FROM mcr.microsoft.com/devcontainers/base:ubuntu

RUN printf "alias cls='clear'\nalias ll='ls -l --human-readable --color=auto --group-directories-first --classify --time-style=long-iso -all'" >> /etc/bash.bashrc

RUN sh -c 'echo "deb https://apt.postgresql.org/pub/repos/apt $(lsb_release -cs)-pgdg main" > /etc/apt/sources.list.d/pgdg.list'
RUN wget --quiet -O - https://www.postgresql.org/media/keys/ACCC4CF8.asc | sudo apt-key add -

ENV PATH="/home/vscode/.cargo/bin:${PATH}"

RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
	&& apt-get -y install --no-install-recommends postgresql-client-17 build-essential pkg-config libssl-dev

USER vscode
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y