IMAGE_REPO = kamudata
KAMU_VERSION = 0.62.2
API_SERVER_VERSION = 0.0.1


.PHONY: api-server
api-server:
	cd .. \
		&& cross build --target x86_64-unknown-linux-gnu --release \
		&& cp target/x86_64-unknown-linux-gnu/release/kamu-api-server images/api-server/

	# Cheating here while there is no deb package for bunyan-rs
	cp ../../../opensource/bunyan/target/x86_64-unknown-linux-gnu/release/bunyan api-server/bunyan
	
	docker build \
		--build-arg KAMU_VERSION=$(KAMU_VERSION) \
		--build-arg API_SERVER_VERSION=$(API_SERVER_VERSION) \
		-t $(IMAGE_REPO)/api-server:$(API_SERVER_VERSION) \
		api-server/

	rm api-server/kamu-api-server api-server/bunyan


.PHONY: api-server-push
api-server-push:
	docker push $(IMAGE_REPO)/api-server:$(KAMU_VERSION)



.PHONY: api-server-with-data
api-server-with-data:
	cd .. \
		&& cross build --target x86_64-unknown-linux-gnu --release \
		&& cp target/x86_64-unknown-linux-gnu/release/kamu-api-server images/api-server/
	
	# Cheating here while there is no deb package for bunyan-rs
	cp ../../../opensource/bunyan/target/x86_64-unknown-linux-gnu/release/bunyan api-server/bunyan
	
	docker build \
		--build-arg KAMU_VERSION=latest-with-data \
		--build-arg API_SERVER_VERSION=$(API_SERVER_VERSION) \
		-t $(IMAGE_REPO)/api-server:latest-with-data \
		api-server/
	
	rm api-server/kamu-api-server api-server/bunyan


.PHONY: api-server-with-data-push
api-server-with-data-push:
	docker push $(IMAGE_REPO)/api-server:latest-with-data
