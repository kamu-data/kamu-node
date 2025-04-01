# Observability Stack
Allows you to spawn a local observability stack that includes Loki, Tempo, and Grafana so you could test logging and tracing locally.

Run the stack:
```sh
docker-compose up
```

Run the server:
```sh
KAMU_OTEL_OTLP_ENDPOINT=http://localhost:4317 <your normal run command>
```

## Issues
Currently we run server outside of `docker-compose` so its logs go into `stderr` and won't make it into Loki. Only OTLP traces will. We need to figure out how this can be fixed without sacrificing the convenience of local testing.
