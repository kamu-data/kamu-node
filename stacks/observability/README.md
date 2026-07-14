# Observability Stack
Allows you to spawn a local observability stack that includes Loki, Tempo, and Grafana so you could test logging and tracing locally.

Run the stack:
```sh
docker-compose up
```

Run the server with:
```sh
KAMU_OTEL_OTLP_ENDPOINT=http://127.0.0.1:4317 <your normal run command>
```

E.g. when using `examples/local-s3`:
```sh
KAMU_OTEL_MODE=dev \
KAMU_OTEL_OTLP_ENDPOINT=http://127.0.0.1:4317 \
RUST_LOG='info' \
make run-postgres
```

You can also use `KAMU_OTEL_OTLP_ENDPOINT=http://127.0.0.1:4327` to send the traces directly to Tempo, bypassing the `otel-collector` component.


## Issues
Currently we run server outside of `docker-compose` so its logs go into `stderr` and won't make it into Loki. Only OTLP traces will. We need to figure out how this can be fixed without sacrificing the convenience of local testing.
