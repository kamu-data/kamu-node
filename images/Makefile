IMAGE_REPO = ghcr.io/kamu-data
KAMU_VERSION = 0.223.0
# KAMU_VERSION = $(shell cargo metadata --format-version 1 | jq -r '.packages[] | select( .name == "kamu-core") | .version')
NODE_VERSION = $(shell cargo metadata --format-version 1 | jq -r '.packages[] | select( .name == "kamu-api-server") | .version')


#########################################################################################

.PHONY: api-server
api-server:
	docker build \
		--build-arg KAMU_VERSION=$(KAMU_VERSION) \
		--build-arg NODE_VERSION=$(NODE_VERSION) \
		-t $(IMAGE_REPO)/kamu-api-server:$(NODE_VERSION) \
		kamu-api-server/


.PHONY: api-server-push
api-server-push:
	docker push $(IMAGE_REPO)/kamu-api-server:$(NODE_VERSION)
	docker tag $(IMAGE_REPO)/kamu-api-server:$(NODE_VERSION) $(IMAGE_REPO)/kamu-api-server:latest
	docker push $(IMAGE_REPO)/kamu-api-server:latest


#########################################################################################

.PHONY: api-server-with-data
api-server-with-data:
	docker build \
		--build-arg KAMU_VERSION=latest-with-data \
		--build-arg NODE_VERSION=$(NODE_VERSION) \
		-t $(IMAGE_REPO)/kamu-api-server:latest-with-data \
		kamu-api-server/


.PHONY: api-server-with-data-push
api-server-with-data-push:
	docker push $(IMAGE_REPO)/kamu-api-server:latest-with-data


#########################################################################################

.PHONY: api-server-with-data-mt
api-server-with-data-mt:
	docker build \
		--build-arg KAMU_VERSION=latest-with-data-mt \
		--build-arg NODE_VERSION=$(NODE_VERSION) \
		-t $(IMAGE_REPO)/kamu-api-server:latest-with-data-mt \
		kamu-api-server-with-data-mt/


.PHONY: api-server-with-data-mt-push
api-server-with-data-mt-push:
	docker push $(IMAGE_REPO)/kamu-api-server:latest-with-data-mt


#########################################################################################

.PHONY: oracle-provider
oracle-provider:
	docker build \
		--build-arg KAMU_VERSION=$(KAMU_VERSION) \
		--build-arg NODE_VERSION=$(NODE_VERSION) \
		-t $(IMAGE_REPO)/kamu-oracle-provider:$(NODE_VERSION) \
		kamu-oracle-provider/


.PHONY: oracle-provider-push
oracle-provider-push:
	docker push $(IMAGE_REPO)/kamu-oracle-provider:$(NODE_VERSION)
	docker tag $(IMAGE_REPO)/kamu-oracle-provider:$(NODE_VERSION) $(IMAGE_REPO)/kamu-oracle-provider:latest
	docker push $(IMAGE_REPO)/kamu-oracle-provider:latest