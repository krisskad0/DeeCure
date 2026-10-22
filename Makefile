# Short names for common developer commands. Run `make help` to list them.

CARGO ?= cargo

.PHONY: help build test lint fmt agent status perf deb deb-test clean

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

agent: ## Run the agent in the foreground (Ctrl+C to stop)
	$(CARGO) run -p deecure-agent -- --rules rules

status: ## Show what the running agent found (deecure status)
	$(CARGO) run -q -p deecure-cli -- status

perf: ## Check the agent against the performance budget (takes about 80 s)
	scripts/perf-check.sh

deb: ## Build the .deb package into target/deb/ and check it with lintian
	scripts/build-deb.sh

deb-test: deb ## Install, use, and purge the .deb in a clean Ubuntu 24.04 container (needs Docker)
	docker run --rm -v "$(CURDIR):/src:ro" ubuntu:24.04 sh -c '/src/scripts/test-deb.sh --throwaway /src/target/deb/deecure_*.deb'

clean: ## Remove build output
	$(CARGO) clean
