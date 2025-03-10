# Example: local-minio

This example shows how to run `kamu-api-server` locally with the S3 repository located in minio.

## Preparation

```shell
cd ./examples/minio
```

### Starting environment

We'll need to run the environment in two different terminals:
1) Run minio first:
```shell
# or docker
podman run --rm \
  -p 9000:9000 \
  -p 9001:9001 \
  -e "MINIO_ROOT_USER=minio" \
  -e "MINIO_ROOT_PASSWORD=minio123" \
  quay.io/minio/minio server /data --console-address ":9001"
```
2) After running `kamu-api-server` via the script:
```shell
./start-kamu-api-server.sh
```

To view S3 metrics open http://127.0.0.1:8080/system/metrics.
