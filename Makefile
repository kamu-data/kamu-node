
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
	cargo set-version --workspace --bump patch

.PHONY: release-minor
release-minor:
	cargo set-version --workspace --bump minor

.PHONY: release-major
release-major:
	cargo set-version --workspace --bump major
