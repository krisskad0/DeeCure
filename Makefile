# Short names for common developer commands. Run `make help` to list them.

CARGO ?= cargo

# The newest tag, and the one before it (after a release: the new and the old version).
# versionsort.suffix puts 0.2.0-rc.1 before 0.2.0, as releases are ordered.
NEWEST = $(patsubst v%,%,$(shell git describe --tags --abbrev=0))
PREVIOUS = $(patsubst v%,%,$(shell git -c versionsort.suffix=- tag --sort=-v:refname | sed -n 2p))

.PHONY: help build test lint fmt agent status app web ui-real perf db-size deb deb-test helper-test upgrade-test install-test clean-install-test update-systemd-test release-check after-release-test clean

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

app: ## Run the desktop app with live reload (start the agent first: make agent)
	cd app && ([ -d node_modules ] || npm ci) && npm run tauri dev

web: ## Test, type-check, and build the web UI
	cd app && ([ -d node_modules ] || npm ci) && npm test && npm run build

ui-real: ## Render every page at the Expert level with this computer's real agent answers (about 2 min; private data stays in /tmp)
	sh scripts/ui-real.sh

perf: ## Check the agent against the performance budget (takes about 80 s)
	scripts/perf-check.sh

db-size: ## Check that 2 years of history stay under 200 MB (takes about 1 min)
	$(CARGO) test --release --locked -p deecure-agent --lib two_years -- --ignored --nocapture

deb: ## Build the .deb package into target/deb/ and check it with lintian
	scripts/build-deb.sh

deb-test: deb ## Install, use, and purge the .deb in a clean Ubuntu 24.04 container (needs Docker)
	docker run --rm -v "$(CURDIR):/src:ro" ubuntu:24.04 sh -c '/src/scripts/test-deb.sh --throwaway /src/target/deb/deecure_*.deb /src/target/deb/deecure-desktop_*.deb'

helper-test: deb ## Test the admin helper with real D-Bus and polkit in a clean Ubuntu 24.04 container (needs Docker)
	docker run --rm -v "$(CURDIR):/src:ro" ubuntu:24.04 sh -c '/src/scripts/test-helper.sh --throwaway /src/target/deb/deecure_*.deb'

upgrade-test: deb ## Upgrade from the newest published release to target/deb/ as the in-app updater does, in a clean container (needs Docker and the network)
	docker run --rm -v "$(CURDIR):/src:ro" ubuntu:24.04 bash -c '/src/scripts/test-upgrade.sh --throwaway $(NEWEST) /src/target/deb/deecure_*.deb /src/target/deb/deecure-desktop_*.deb'

install-test: deb ## Build a signed APT repository and test install.sh against it in a clean container (needs Docker)
	docker run --rm -v "$(CURDIR):/src:ro" ubuntu:24.04 bash -c '/src/scripts/test-install.sh --throwaway /src/target/deb/deecure_*.deb /src/target/deb/deecure-desktop_*.deb'

update-systemd-test: ## After a release: the in-app update, automatic update, repair, and helper sandbox, with systemd running (needs Docker; BROKEN=, OLD=, NEW= versions)
	scripts/systemd-container.sh ubuntu:24.04 scripts/test-update-systemd.sh --throwaway $(or $(BROKEN),0.1.0-alpha.6) $(or $(OLD),0.1.0-alpha.7) $(or $(NEW),$(NEWEST))

clean-install-test: ## The real one-command install from GitHub on clean Ubuntu 24.04 (curl) and 26.04 (wget) containers, both at once (needs Docker)
	scripts/run-parallel.sh \
	  clean-install-24.04 "docker run --rm -v '$(CURDIR):/src:ro' ubuntu:24.04 bash /src/scripts/test-clean-install.sh --throwaway curl $(NEWEST)" \
	  clean-install-26.04 "docker run --rm -v '$(CURDIR):/src:ro' ubuntu:26.04 bash /src/scripts/test-clean-install.sh --throwaway wget $(NEWEST)"

release-check: deb ## Before tagging: build once, then the upgrade from the newest release and every page with real data, at once (CI and the release job run deb-test and helper-test)
	scripts/run-parallel.sh \
	  upgrade-test "docker run --rm -v '$(CURDIR):/src:ro' ubuntu:24.04 bash -c '/src/scripts/test-upgrade.sh --throwaway $(NEWEST) /src/target/deb/deecure_*.deb /src/target/deb/deecure-desktop_*.deb'" \
	  ui-real "sh scripts/ui-real.sh"

after-release-test: ## After a release is published: the one-command install (24.04, 26.04), the in-app update from the release before, and updates with systemd, all at once (needs Docker)
	scripts/run-parallel.sh \
	  clean-install-24.04 "docker run --rm -v '$(CURDIR):/src:ro' ubuntu:24.04 bash /src/scripts/test-clean-install.sh --throwaway curl $(NEWEST)" \
	  clean-install-26.04 "docker run --rm -v '$(CURDIR):/src:ro' ubuntu:26.04 bash /src/scripts/test-clean-install.sh --throwaway wget $(NEWEST)" \
	  self-update "docker run --rm -v '$(CURDIR):/src:ro' ubuntu:24.04 bash /src/scripts/test-self-update.sh --throwaway $(PREVIOUS) $(NEWEST)" \
	  update-systemd "scripts/systemd-container.sh ubuntu:24.04 scripts/test-update-systemd.sh --throwaway $(or $(BROKEN),0.1.0-alpha.6) $(or $(OLD),0.1.0-alpha.7) $(or $(NEW),$(NEWEST))"

clean: ## Remove build output
	$(CARGO) clean
