ARG KAMU_VERSION
FROM ghcr.io/kamu-data/kamu-base:${KAMU_VERSION}

WORKDIR /opt/kamu

COPY ./tmp/kamu-api-server /opt/kamu/

ENTRYPOINT ["/usr/bin/tini", "--"]

CMD ["/opt/kamu/kamu-api-server", "run", "--address=0.0.0.0", "--http-port=8080"]

EXPOSE 8080/tcp
