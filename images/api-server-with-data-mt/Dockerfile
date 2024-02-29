ARG KAMU_VERSION
FROM ghcr.io/kamu-data/kamu-base:${KAMU_VERSION}
ARG API_SERVER_VERSION

WORKDIR /opt/kamu

RUN wget -q https://github.com/kamu-data/kamu-platform/releases/download/v${API_SERVER_VERSION}/kamu-api-server-x86_64-unknown-linux-gnu.tar.gz && \
    tar -xf kamu-api-server-x86_64-unknown-linux-gnu.tar.gz && \
    chmod +x kamu-api-server-x86_64-unknown-linux-gnu/kamu-api-server && \
    mv kamu-api-server-x86_64-unknown-linux-gnu/kamu-api-server /opt/kamu/ && \
    rm -rf kamu-api-server-x86_64-unknown-linux-gnu*

COPY config.yaml /opt/kamu/config.yaml

ENTRYPOINT ["/usr/bin/tini", "--"]

CMD [ "/opt/kamu/kamu-api-server", "--config=/opt/kamu/config.yaml", "--repo-url=file:///opt/kamu/workspace/.kamu/datasets", "--multi-tenant", "run", "--address=0.0.0.0", "--http-port=8080" ]

EXPOSE 8080/tcp
