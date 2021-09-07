FROM ubuntu:focal

ARG NODE_VERSION=16.8.0

# NodeJS
RUN apt-get update && \
    apt-get -y install curl && \
    rm -rf /var/lib/apt/lists/*

ENV NVM_DIR=/root/.nvm

RUN curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.38.0/install.sh | bash

RUN . "$NVM_DIR/nvm.sh" && \
    nvm install $NODE_VERSION && \
    nvm use v$NODE_VERSION && \
    nvm alias default v$NODE_VERSION

ENV PATH="/root/.nvm/versions/node/v${NODE_VERSION}/bin/:${PATH}"


# Web-UI
COPY web-ui /opt/kamu/web-ui
WORKDIR /opt/kamu/web-ui 
RUN npm install

# TODO: Properly deploy app and get rid of dev mode websocket port
EXPOSE 80/tcp 24678/tcp
CMD ["npx", "svelte-kit", "dev", "--host=0.0.0.0", "--port=80"]
