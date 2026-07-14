# Example: local-s3

This example shows how to run `kamu-api-server` locally with an S3-compatible repository.

### Preparation

```shell
cd ./examples/local-s3
```

### Starting environment

We'll need to run the environment in two different terminals:
1) Start `rustfs` first:
```shell
make rustfs-start
```
2) (Optional) Create a dump of the bucket data from the test environment of interest:
```shell
aws-sso exec
aws s3 sync s3://TEST_ENV_HOST ./aws-datasets-bucket
```
During a later startup, the saved data will be synchronized into rustfs.

3) Initialize `rustfs` buckets and sync data:
```shell
make rustfs-sync
```

4) You can now start `kamu-api-server`:

- 3.1) (Simpler) SQLite database startup option:
  ```shell
  RUST_LOG=info make run-sqlite
  ```
- 3.2) (More complex) PostgreSQL database startup option:
  - Start the Database:
    ```shell
    make postgres-start
    ```
  - Download the database `./dump.sql` using the [provided instructions](https://github.com/kamu-data/kamu-deploy/blob/master/DEVELOPER.md#make-a-database-backup).
  - Apply the dump to the local database:
    ```shell
    # ⚠️ Please note: when applied, Outbox tables will be cleared.
    make postgres-restore-dump
    ```
  - Run the server:
  ```shell
  RUST_LOG=info make run-postgres
  ```

5) Clean up when you're done:
```shell
make rustfs-stop postgres-stop clean-sqlite-data clean-aws-datasets-bucket

# or simply
make clean
```

### Useful extras

- To view S3 metrics, open http://127.0.0.1:8080/system/metrics.
