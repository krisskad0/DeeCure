# Short names for common developer commands. Run `make help` to list them.

CARGO ?= cargo

.PHONY: help build test lint fmt agent status clean

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

clean: ## Remove build output
	$(CARGO) clean
