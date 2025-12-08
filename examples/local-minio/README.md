# Example: local-minio

This example shows how to run `kamu-api-server` locally with the S3 repository located in minio.

### Preparation

```shell
cd ./examples/local-minio
```

### Starting environment

We'll need to run the environment in two different terminals:
1) Run minio first:
```shell
# or docker
podman run --rm -d \
  -p 9000:9000 \
  -p 9001:9001 \
  -v "./minio-data:/data:Z" \
  -e "MINIO_ROOT_USER=minio" \
  -e "MINIO_ROOT_PASSWORD=minio123" \
  quay.io/minio/minio server /data --console-address ":9001"
```
2) (Optional) Create a dump of the bucket data from the test environment of interest:
```shell
aws-sso exec
aws s3 sync s3://TEST_ENV_HOST ./aws-datasets-bucket
```
During a later startup, the saved data will be synchronized into minio.

3) After running `kamu-api-server` via the script:

- 3.1) (Simpler) SQLite database startup option:
  ```shell
  RUST_LOG=info ./start-kamu-api-server.sh sqlite
  ```
- 3.2) (More complex) PostgreSQL database startup option:
  - Start the Database. As an option, you can use the script from the repo root directory:
    ```shell
    make sqlx-local-setup-postgres
    ```
  - Download the database dump using the [provided instructions](https://github.com/kamu-data/kamu-deploy/blob/master/DEVELOPER.md#make-a-database-backup).
  - Apply the dump to the local database:
    ```shell
    psql -U root -h 127.0.0.1 -p 5433 -d kamu -f DUMP.sql
    ```
  - Apply latest migrations, if necessary:
    ```shell
    sqlx migrate run --source ../../../kamu-cli/migrations/postgres --database-url postgres://root:root@localhost:5433/kamu
    ```
  - Run the server:
  ```shell
  RUST_LOG=info ./start-kamu-api-server.sh postgres
  ```

### Useful extras

- To view S3 metrics, open http://127.0.0.1:8080/system/metrics.
