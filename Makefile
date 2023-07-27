
###############################################################################
# Lint
###############################################################################

.PHONY: lint
lint:
	cargo fmt --check
	cargo test -p kamu-repo-tools
	cargo deny check


###############################################################################
# Test
###############################################################################

.PHONY: test
test:
	cargo test


###############################################################################
# Release
###############################################################################

.PHONY: release-patch
release-patch:
	cargo run -p kamu-repo-tools --bin release -- --patch

.PHONY: release-minor
release-minor:
	cargo run -p kamu-repo-tools --bin release -- --minor

.PHONY: release-major
release-major:
	cargo run -p kamu-repo-tools --bin release -- --major
