KAMU_VERSION=$(shell cargo metadata --format-version 1 | jq -r '.packages[] | select( .name == "kamu-core") | .version')
IMAGE_REPO=ghcr.io/kamu-data
IMAGE_NAME=kamu-dev-api-server
TARGET=x86_64-unknown-linux-gnu
# Branch name
TAG=$(shell git rev-parse --abbrev-ref HEAD | sed 's/\//-/g')


################################################################################

.PHONY: build
build:
	# cargo binstall cross
	cd ../../ && RUSTFLAGS="" cross build -p kamu-api-server --release --target=$(TARGET)


.PHONY: image
image: build
	mkdir -p ./tmp
	cp ../../target/$(TARGET)/release/kamu-api-server tmp/kamu-api-server
	docker build \
		--build-arg KAMU_VERSION=$(KAMU_VERSION) \
		-t $(IMAGE_REPO)/$(IMAGE_NAME):$(TAG) \
		.
	rm -rf ./tmp


.PHONY: image-push
image-push:
	docker push $(IMAGE_REPO)/$(IMAGE_NAME):$(TAG)

################################################################################
