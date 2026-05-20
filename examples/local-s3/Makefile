###############################################################################
# rustfs
###############################################################################

.PHONY: rustfs-start
rustfs-start:
	mkdir -p rustfs-data
	podman run \
		--name kamu-rustfs \
		--rm -d \
		-p 9000:9000 \
		-p 9001:9001 \
		-e "RUSTFS_ROOT_USER=rustfs" \
		-e "RUSTFS_ROOT_PASSWORD=rustfs123" \
		docker.io/rustfs/rustfs server /data --console-address ":9001"


.PHONY: rustfs-stop
rustfs-stop:
	podman kill kamu-rustfs || true


.PHONY: rustfs-sync
rustfs-sync:
	./rustfs-sync.sh


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
	PGPASSWORD=root psql -U root -h 127.0.0.1 -p 5433 -d kamu \
	  -c "CREATE ROLE postgres;" \
	  -c "CREATE ROLE kamu_api_server;" \
	  -f ./dump.sql \
	  -c "TRUNCATE public.outbox_messages, public.outbox_message_consumptions;"
	sqlx migrate run --source ../../../kamu-cli/migrations/postgres --database-url postgres://root:root@localhost:5433/kamu


.PHONY: postgres-refresh-dump
postgres-refresh-dump: postgres-stop postgres-start postgres-restore-dump


###############################################################################
# API Server
###############################################################################

.PHONY: run-sqlite
run-sqlite:
	AWS_ACCESS_KEY_ID=rustfs \
	AWS_SECRET_ACCESS_KEY=rustfs123 \
	AWS_ENDPOINT_URL=http://localhost:9000 \
	AWS_SESSION_TOKEN= \
		cargo run -p kamu-api-server -- --config config_sqlite.yaml run --address=127.0.0.1 --http-port=8080 --flightsql-port=50050


.PHONY: run-postgres
run-postgres:
	AWS_ACCESS_KEY_ID=rustfs \
	AWS_SECRET_ACCESS_KEY=rustfs123 \
	AWS_ENDPOINT_URL=http://localhost:9000 \
	AWS_SESSION_TOKEN= \
		cargo run -p kamu-api-server -- --config config_postgres.yaml run --address=127.0.0.1 --http-port=8080 --flightsql-port=50050


###############################################################################
# Cleanup
###############################################################################

.PHONY: clean-sqlite-data
clean-sqlite-data:
	rm kamu-api-server.sqlite*

.PHONY: clean-aws-datasets-bucket
clean-aws-datasets-bucket:
	rm -rf aws-datasets-bucket

.PHONY: clean
clean: rustfs-stop postgres-stop clean-sqlite-data clean-aws-datasets-bucket
