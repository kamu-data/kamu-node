IMAGE_REPO = ghcr.io/kamu-data
KAMU_VERSION = $(shell cargo metadata --format-version 1 | jq -r '.packages[] | select( .name == "kamu-core") | .version')
API_SERVER_VERSION = $(shell cargo metadata --format-version 1 | jq -r '.packages[] | select( .name == "kamu-api-server") | .version')


#########################################################################################

.PHONY: api-server
api-server:
	docker build \
		--build-arg KAMU_VERSION=$(KAMU_VERSION) \
		--build-arg API_SERVER_VERSION=$(API_SERVER_VERSION) \
		-t $(IMAGE_REPO)/kamu-api-server:$(API_SERVER_VERSION) \
		api-server/


.PHONY: api-server-push
api-server-push:
	docker push $(IMAGE_REPO)/kamu-api-server:$(API_SERVER_VERSION)
	docker tag $(IMAGE_REPO)/kamu-api-server:$(API_SERVER_VERSION) $(IMAGE_REPO)/kamu-api-server:latest
	docker push $(IMAGE_REPO)/kamu-api-server:latest


#########################################################################################

.PHONY: api-server-with-data
api-server-with-data:
	docker build \
		--build-arg KAMU_VERSION=latest-with-data \
		--build-arg API_SERVER_VERSION=$(API_SERVER_VERSION) \
		-t $(IMAGE_REPO)/kamu-api-server:latest-with-data \
		api-server/


.PHONY: api-server-with-data-push
api-server-with-data-push:
	docker push $(IMAGE_REPO)/kamu-api-server:latest-with-data


#########################################################################################

.PHONY: api-server-with-data-mt
api-server-with-data-mt:
	docker build \
		--build-arg KAMU_VERSION=latest-with-data-mt \
		--build-arg API_SERVER_VERSION=$(API_SERVER_VERSION) \
		-t $(IMAGE_REPO)/kamu-api-server:latest-with-data-mt \
		api-server-with-data-mt/


.PHONY: api-server-with-data-mt-push
api-server-with-data-mt-push:
	docker push $(IMAGE_REPO)/kamu-api-server:latest-with-data-mt
