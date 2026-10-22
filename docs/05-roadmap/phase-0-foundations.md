# Phase 0 — Foundations

[Docs](../README.md) › [Roadmap](README.md)

**Goal:** a working skeleton. It installs as a `.deb` and `deecure status` shows real data.
**Rough time:** 3–4 weeks.

## Progress

| Task | Status |
| --- | --- |
| 1. Blocking decisions | Name ✅, Rust ✅. License and app ID still open. |
| 2. Repo setup | git ✅, CI ✅. Branch protection: turn on in GitHub settings. |
| 3. Cargo workspace | ✅ Done |
| 4. Shared types | ✅ Done. Wire format in [IPC protocol](../02-architecture/ipc-protocol.md). |
| 5. Agent skeleton | ✅ Done. Socket activation, same-user check, `version` and `status`. Idle: 2.4 MB RAM, 0% CPU. |
| 6. Storage | ✅ Done. SQLite schema v1, migrations, hourly cleanup; `status` reads findings from it. |
| 7. First 5 collectors | ✅ Done. CPU, memory, filesystems, battery, services; new `metrics` and `snapshot` calls. |
| 8. Rules engine | ✅ Done. YAML rules, condition language, finding lifecycle; 3 rules shipped. |
| 9. CLI | ✅ Done. `deecure status` (plain English) and `--json`; friendly message when the agent is not running. |
| 10. Fixture tests | ✅ Done. Capture script; 3 machines (1 real, 2 made up); exact expected findings. More real machines welcome. |
| 11. First `.deb` | ✅ Done. `make deb` (lintian clean), `make deb-test` (install, use, purge in a container); CI builds and tests it and keeps the file. |
| 12. Performance check | ✅ Done. `make perf` and CI job. On a Dell Latitude 3450: first result 0.1 s, idle CPU 0.02%, peak RAM 8.2 MB, database 1 MB. |

## Tasks (in order)

1. **Close the blocking decisions:** name, license, GitHub org, app ID, Rust vs Go.
   → [Open decisions](../01-product/open-decisions.md)
2. **Set up the repo:** `git`, branch protection on `main`, CI that runs format, lint, and tests on every pull request.
   → [CONTRIBUTING.md](../../CONTRIBUTING.md), [Testing](../04-development/testing.md)
3. **Create the Cargo workspace** with `deecure-core`, `deecure-agent`, and `deecure-cli`. Add `rust-toolchain.toml` and the `Makefile`.
   → [Project structure](../04-development/project-structure.md)
4. **Shared types in `deecure-core`:** finding, severity, metric, event, snapshot, and the request and response messages.
   → [Architecture overview](../02-architecture/overview.md)
5. **Agent skeleton:** systemd user socket and service, JSON-RPC over the Unix socket, caller user ID check.
   → [How the parts talk](../02-architecture/overview.md#how-the-parts-talk)
6. **Storage:** SQLite in WAL mode, the six tables, migrations, and the retention job.
   → [Storage](../02-architecture/storage.md)
7. **First 5 collectors** with the Ubuntu adapter: CPU, memory, filesystems, battery, systemd services.
   → [Collectors](../02-architecture/collectors.md)
8. **Rules engine v0:** load YAML rules, evaluate them, save findings. Ship 3 rules, starting with `storage.root_almost_full`.
   → [Rules engine](../02-architecture/rules-engine.md), [Writing rules](../04-development/writing-rules-and-kb.md)
9. **CLI:** `deecure status` and `deecure status --json`.
   → [CLI](../02-architecture/cli.md)
10. **Fixture tests:** a capture script and fixtures from at least 3 real machines.
    → [Testing](../04-development/testing.md)
11. **First `.deb`:** CI builds an installable package with the agent, CLI, and units. `lintian` passes.
    → [Debian package](../06-packaging/debian-package.md)
12. **Performance check in CI:** agent idle CPU and RAM stay inside the budget.
    → [Performance budget](../02-architecture/performance-budget.md)

## Done when

- [x] On a clean Ubuntu 24.04 system, `sudo apt install ./deecure_*.deb` works, then `deecure status` shows real data. *Checked in a clean container (`make deb-test`) and in CI. Still to try: a full desktop VM, where systemd starts the agent at login.*
- [x] `sudo apt purge deecure` removes everything it installed. *Checked by `make deb-test` and CI.*
- [x] CI is green: format, lint, unit tests, fixture tests, `lintian`, performance check. *All four CI jobs pass (run 39850817850, 2026-10-23).*

---

[Index](README.md) · Next: [Phase 1 — MVP](phase-1-mvp.md) →
