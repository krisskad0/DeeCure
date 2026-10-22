# Short names for common developer commands. Run `make help` to list them.

CARGO ?= cargo

.PHONY: help build test lint fmt clean

help: ## List the available commands
	@grep -E '^[a-z-]+:.*## ' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*## "} {printf "  make %-8s %s\n", $$1, $$2}'

build: ## Build every crate
	$(CARGO) build --workspace --locked

test: ## Run every test
	$(CARGO) test --workspace --locked

lint: ## Check formatting and lints (CI runs this)
	$(CARGO) fmt --all --check
	$(CARGO) clippy --workspace --all-targets --locked -- -D warnings

fmt: ## Format all Rust code
	$(CARGO) fmt --all

clean: ## Remove build output
	$(CARGO) clean
