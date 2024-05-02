FROM docker.io/library/ubuntu:24.04
ARG NODE_VERSION

WORKDIR /opt/kamu

RUN wget -q https://github.com/kamu-data/kamu-node/releases/download/v${NODE_VERSION}/kamu-oracle-provider-x86_64-unknown-linux-gnu.tar.gz && \
    tar -xf kamu-oracle-provider-x86_64-unknown-linux-gnu.tar.gz && \
    chmod +x kamu-oracle-provider-x86_64-unknown-linux-gnu/kamu-oracle-provider && \
    mv kamu-oracle-provider-x86_64-unknown-linux-gnu/kamu-oracle-provider /opt/kamu/ && \
    rm -rf kamu-oracle-provider-x86_64-unknown-linux-gnu*


ENTRYPOINT ["/usr/bin/tini", "--"]

CMD ["/opt/kamu/kamu-oracle-provider"]

EXPOSE 8080/tcp
