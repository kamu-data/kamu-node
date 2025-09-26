#!/usr/bin/env bash

set -euo pipefail

show_usage() {
    echo "Usage: $0 {sqlite,postgres}"
    exit 1
}

if [ $# -lt 1 ]; then
    show_usage
fi

case "$1" in
    sqlite)
        CONFIG_FILE="config_sqlite.yaml"
        ;;
    postgres)
        CONFIG_FILE="config_postgres.yaml"
        ;;
    *)
        show_usage
        ;;
esac

export AWS_ACCESS_KEY_ID=minio
export AWS_SECRET_ACCESS_KEY=minio123
export AWS_ENDPOINT_URL=http://localhost:9000
export AWS_SESSION_TOKEN=

BUCKETS=("datasets" "upload")

for i in "${!BUCKETS[@]}"; do
    BUCKET="${BUCKETS[$i]}"

    if ! aws s3api head-bucket --bucket "${BUCKET}" 2>/dev/null; then
        aws s3 mb "s3://${BUCKET}"
    fi
done

aws s3 sync ./aws-datasets-bucket s3://datasets

cargo run -p kamu-api-server -- --config "${CONFIG_FILE}" run --address=127.0.0.1 --http-port=8080 --flightsql-port=50050
