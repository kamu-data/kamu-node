KAMU_VERSION=$(shell cargo metadata --format-version 1 | jq -r '.packages[] | select( .name == "kamu-core") | .version')
IMAGE_REPO=ghcr.io/kamu-data
TAG=dev-$(shell git rev-parse --abbrev-ref HEAD | sed 's/\//-/g')


################################################################################

.PHONY: build
build:
	cd ../../ && cargo build -p kamu-api-server --release


.PHONY: image
image: build
	mkdir -p ./tmp
	cp ../../target/release/kamu-api-server tmp/kamu-api-server
	docker build \
		--build-arg KAMU_VERSION=$(KAMU_VERSION) \
		-t $(IMAGE_REPO)/kamu-api-server:$(TAG) \
		.
	rm -rf ./tmp


.PHONY: image-push
image-push:
	docker push $(IMAGE_REPO)/kamu-api-server:$(TAG)

################################################################################
