#!/usr/bin/env bash

set -euo pipefail

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
