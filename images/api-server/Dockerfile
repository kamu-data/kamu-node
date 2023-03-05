ARG KAMU_VERSION
FROM kamudata/kamu-base:${KAMU_VERSION}

ARG BUNYAN_RS_VERSION=0.1.9
ARG API_SERVER_VERSION

WORKDIR /opt/kamu

# Bunyan
RUN wget -q https://github.com/LukeMathWalker/bunyan/releases/download/v${BUNYAN_RS_VERSION}/bunyan-v${BUNYAN_RS_VERSION}-x86_64-unknown-linux-gnu.tar.gz && \
    tar -xf bunyan-v${BUNYAN_RS_VERSION}-x86_64-unknown-linux-gnu.tar.gz && \
    chmod +x bunyan && \
    mv bunyan /usr/local/bin/ && \
    rm -f bunyan-v${BUNYAN_RS_VERSION}-x86_64-unknown-linux-gnu.tar.gz


# API Server
RUN wget -q https://github.com/kamu-data/kamu-platform/releases/download/v${API_SERVER_VERSION}/kamu-api-server-x86_64-unknown-linux-gnu.tar.gz && \
    tar -xf kamu-api-server-x86_64-unknown-linux-gnu.tar.gz && \
    chmod +x kamu-api-server-x86_64-unknown-linux-gnu/kamu-api-server && \
    mv kamu-api-server-x86_64-unknown-linux-gnu/kamu-api-server /opt/kamu/ && \
    rm -rf kamu-api-server-x86_64-unknown-linux-gnu*


ENTRYPOINT ["/usr/bin/tini", "--"]
CMD ["/opt/kamu/kamu-api-server --local-repo /opt/kamu/workspace run --address 0.0.0.0 --http-port 80 | bunyan"]
EXPOSE 80/tcp
