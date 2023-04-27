ARG KAMU_VERSION
FROM ghcr.io/kamu-data/kamu-base:${KAMU_VERSION}

ARG API_SERVER_VERSION

WORKDIR /opt/kamu

# API Server
RUN wget -q https://github.com/kamu-data/kamu-platform/releases/download/v${API_SERVER_VERSION}/kamu-api-server-x86_64-unknown-linux-gnu.tar.gz && \
    tar -xf kamu-api-server-x86_64-unknown-linux-gnu.tar.gz && \
    chmod +x kamu-api-server-x86_64-unknown-linux-gnu/kamu-api-server && \
    mv kamu-api-server-x86_64-unknown-linux-gnu/kamu-api-server /opt/kamu/ && \
    rm -rf kamu-api-server-x86_64-unknown-linux-gnu*


ENTRYPOINT ["/usr/bin/tini", "--"]
# S3-based:
CMD ["/opt/kamu/kamu-api-server --repo-url s3+https://s3.us-west-2.amazonaws.com/datasets.kamu.dev/ run --address 0.0.0.0 --http-port 80"]
# Local repository:
# CMD ["/opt/kamu/kamu-api-server --local-repo /opt/kamu/workspace run --address 0.0.0.0 --http-port 80"]
EXPOSE 80/tcp
