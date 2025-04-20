TARGET=x86_64-unknown-linux-gnu

KAMU_VERSION=$(shell cargo metadata --format-version 1 | jq -r '.packages[] | select( .name == "kamu-core") | .version')
NODE_VERSION=$(shell cargo metadata --format-version 1 | jq -r '.packages[] | select( .name == "kamu-api-server") | .version')
BRANCH_NAME=$(shell git rev-parse --abbrev-ref HEAD | sed 's/\//-/g')
SHA_SHORT=$(shell git rev-parse --short HEAD)

IMAGE_REPO=ghcr.io/kamu-data
IMAGE_NAME=kamu-dev-api-server
IMAGE_TAG=$(NODE_VERSION)-$(BRANCH_NAME)-$(SHA_SHORT)


################################################################################

.PHONY: build
build:
	# cargo binstall cross
	cd ../../ && RUSTFLAGS="" cross build -p kamu-api-server --release --target=$(TARGET)


.PHONY: image
image:
	mkdir -p ./tmp
	cp ../../target/$(TARGET)/release/kamu-api-server tmp/kamu-api-server
	docker build \
		--build-arg KAMU_VERSION=$(KAMU_VERSION) \
		-t $(IMAGE_REPO)/$(IMAGE_NAME):$(IMAGE_TAG) \
		.
	rm -rf ./tmp


.PHONY: image-push
image-push:
	docker push $(IMAGE_REPO)/$(IMAGE_NAME):$(IMAGE_TAG)

################################################################################
