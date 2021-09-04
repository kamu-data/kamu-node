ARG KAMU_VERSION
FROM kamudata/kamu-base:$KAMU_VERSION


# API Server
COPY ./bunyan /usr/local/bin/bunyan
COPY ./kamu-api-server /opt/kamu/api-server

ENTRYPOINT ["bash", "-c"]
CMD ["/opt/kamu/api-server --metadata-repo file:///opt/kamu/workspace run --address 0.0.0.0 | bunyan"]
EXPOSE 8080/tcp
