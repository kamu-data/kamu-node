###############################################################################
# Minio
###############################################################################

.PHONY: minio-start
minio-start:
	podman run \
		--name kamu-minio \
		--rm -d \
		-p 9000:9000 \
		-p 9001:9001 \
		-v "./minio-data:/data:Z" \
		-e "MINIO_ROOT_USER=minio" \
		-e "MINIO_ROOT_PASSWORD=minio123" \
		quay.io/minio/minio server /data --console-address ":9001"


.PHONY: minio-stop
minio-stop:
	podman kill kamu-minio || true


.PHONY: minio-sync
minio-sync:
	./minio-sync.sh


###############################################################################
# Postgres
###############################################################################

.PHONY: postgres-start
postgres-start:
	make -C ../../ sqlx-local-setup-postgres


.PHONY: postgres-stop
postgres-stop:
	make -C ../../ sqlx-local-clean-postgres


.PHONY: postgres-restore-dump
postgres-restore-dump:
	psql -U root -h 127.0.0.1 -p 5433 -d kamu -f ./dump.sql
	sqlx migrate run --source ../../../kamu-cli/migrations/postgres --database-url postgres://root:root@localhost:5433/kamu


###############################################################################
# API Server
###############################################################################

.PHONY: run-sqlite
run-sqlite:
	AWS_ACCESS_KEY_ID=minio \
	AWS_SECRET_ACCESS_KEY=minio123 \
	AWS_ENDPOINT_URL=http://localhost:9000 \
	AWS_SESSION_TOKEN= \
		cargo run -p kamu-api-server -- --config config_sqlite.yaml run --address=127.0.0.1 --http-port=8080 --flightsql-port=50050


.PHONY: run-postgres
run-postgres:
	AWS_ACCESS_KEY_ID=minio \
	AWS_SECRET_ACCESS_KEY=minio123 \
	AWS_ENDPOINT_URL=http://localhost:9000 \
	AWS_SESSION_TOKEN= \
		cargo run -p kamu-api-server -- --config config_postgres.yaml run --address=127.0.0.1 --http-port=8080 --flightsql-port=50050


###############################################################################
# Cleanup
###############################################################################

.PHONY: clean-minio-data
clean-minio-data:
	rm -rf minio-data/
	git checkout minio-data/.gitkeep


.PHONY: clean-aws-datasets-bucket
clean-aws-datasets-bucket:
	rm -rf aws-datasets-bucket/
	git checkout aws-datasets-bucket/.gitkeep


.PHONY: clean
clean: minio-stop postgres-stop clean-minio-data clean-aws-datasets-bucket
