# Testing

[Docs](../README.md) › [Development](README.md)

1. **Fixture tests:** capture real `/proc`, `/sys`, journal, and command outputs from many machines, then run collectors and rules against them offline.
2. **VM matrix:** run the full app on supported Ubuntu LTS releases, desktop and server, in VMs on every release.
3. **Hardware lab:** test on a few real laptops (Intel, AMD, NVIDIA, ThinkPad, budget laptop) because sensors and batteries differ.
4. **Fix safety tests:** each fix must pass apply → verify → undo in a VM before it ships.
5. **Beginner tests:** 5 non-technical users try three tasks ("free up space", "why is it slow", "what changed"). Measure success without help.

## Tools

| Test type | Tool | Runs | State |
| --- | --- | --- | --- |
| Rust unit and integration tests | `cargo test` (`make test`) | Every push (CI) | ✅ About 270 tests |
| Real-machine fixtures | `cargo test` against `tests/fixtures/` | Every push | ✅ 3 machines |
| UI tests | Vitest + React Testing Library (`make web`) | Every push | ✅ About 190 tests |
| Every page, Expert level, no agent | `app/src/allpages.test.tsx` (part of `make web`) | Every push | ✅ All 78 tabs show their help, nothing crashes, no React error or warning |
| Every page with this computer's real answers, and with an older agent | `make ui-real` (records a throwaway agent's answers to a private file in `/tmp`, never the repository) | Before a release, on a real desktop | ✅ All 78 tabs and the row drawers |
| Lint and format | `cargo fmt`, `cargo clippy -D warnings`, `tsc` | Every push | ✅ |
| Generated UI code matches the spec | `scripts/gen-ui.py --check` | Every push | ✅ |
| Performance budget | `make perf` | Every push | ✅ |
| 2 years of history under 200 MB | `make db-size` | Every push | ✅ 124 MB |
| Package build and lintian | `make deb` | Every push | ✅ |
| Install, use, purge in a clean container | `make deb-test` | Every push | ✅ |
| Root helper with real D-Bus and polkit | `make helper-test` | Every push | ✅ |
| Install script against a signed test repository | `make install-test` | Before a release | ✅ |
| The real one-command install from GitHub, on clean Ubuntu 24.04 (curl) and 26.04 (wget) | `make clean-install-test` | After each release | ✅ Passed with v0.1.0-alpha.2 on 2026-10-23 |
| Desktop notifications | A fake notification server on a private session bus (`dbus-run-session`), never the real desktop | By hand | ✅ |
| Dependency checks | `cargo-deny` (known vulnerabilities, licenses, sources; `deny.toml`) and `npm audit` | Every push, pull request, and weekly | ✅ |
| Code scanning | CodeQL for Rust, TypeScript, and the workflows | Every push, pull request, and weekly | ✅ |
| Security practices | OpenSSF Scorecard | Every push to `main` and weekly | ✅ |
| Randomized input tests of the root helper | 50,000 hostile inputs against every input check, and 20,000 calls with a system that says yes to everything (`cargo test`) | Every push | ✅ |
| Fuzzing with coverage | `cargo-fuzz` on the helper's input checks | Nightly | Planned |
| End-to-end UI tests | WebDriver with `tauri-driver` | Before release | Planned |
| `piuparts`, `autopkgtest` | Package install → upgrade → purge leaves nothing; smoke tests | Release builds | Planned |
| VM matrix and fix safety | Ubuntu 24.04 and 26.04 VMs: each fix passes apply → verify → undo | Before every release | Pending ([Pending work](../05-roadmap/pending.md)) |
| Beginner tests | 5 non-technical users, three tasks | Before the beta | Pending |

### When each check runs (no wasted runs)

| Workflow | Runs on | Skips |
| --- | --- | --- |
| CI | Every push to `main` and every pull request | The heavy jobs, when only docs changed (the `What changed` job decides; anything unclear runs everything) |
| Pull request (sign-off and title) | Every pull request event, including description edits | Nothing: it takes 10 seconds, and a skipped required check would count as passed |
| Dependencies | Changes to `Cargo.lock`, `Cargo.toml`, `package*.json`, or `deny.toml`, and every Monday | Everything else |
| CodeQL | Code changes, and every Tuesday | Docs-only changes; the design prototype, tests, and fixtures are not scanned |
| Scorecard | Every Wednesday, and when protection rules change | Normal pushes |
| Dependabot | Monthly: minor and patch updates grouped, each major version alone; security fixes at once | |
| Release | `v*` tags only | |

## Where tests live

| Path | What |
| --- | --- |
| `crates/<crate>/src/**` | Rust unit tests, next to the code |
| `crates/<crate>/tests/` | Rust integration tests |
| `app/src/**/*.test.tsx` | UI component tests |
| `tests/fixtures/<machine>/` | Captured system data from one real machine, plus expected findings |
| `scripts/test-deb.sh`, `test-helper.sh`, `test-install.sh` | Container tests of the packages, the helper, and the installer |
| `tests/vm/` | VM test scripts (planned) |

## Fixtures

A fixture is a saved copy of what the collectors read on one machine. `crates/deecure-agent/tests/fixtures.rs` replays every fixture through the real collectors and rules, and checks that DeeCure reports **exactly** the expected findings.

```
tests/fixtures/<machine>/
├── fixture.yaml             # description, source (real or synthetic), machine, OS, kernel
├── proc/                    # stat, loadavg, meminfo, mounts
├── sys/class/power_supply/  # battery and charger files (missing on desktops)
├── statvfs.json             # disk space per mount point
├── services.json            # systemd services and their states
└── expected-findings.yaml   # the exact findings DeeCure must report
```

| Fixture | Source | Why it is there |
| --- | --- | --- |
| `dell-latitude-3450-ubuntu-24.04` | Real | LVM disk, separate `/boot`, USB-C power supply that must be ignored, worn battery |
| `riya-laptop-synthetic` | Made up, from the UI prototype | Disk 94% full, failed Bluetooth service, worn battery, btrfs subvolumes, snap images |
| `desktop-no-battery-synthetic` | Made up | No battery (no battery finding allowed), network share that must be skipped, no swap |

More real machines make DeeCure safer. Captures from other laptops and desktops are very welcome.

## Adding a fixture

1. On the machine, run `scripts/capture-fixture.py <name> --description "…"`. It copies only what the collectors read. It replaces your user name and host name, blanks disk UUIDs, and skips battery serial numbers, models, and makers.
2. Read every file it wrote and remove anything private that is left, such as a personal disk label.
3. Run `cargo test -p deecure-agent --test fixtures`. The new fixture fails at first and prints what DeeCure found.
4. Check that the result is right for that machine, then copy it into `expected-findings.yaml`.
5. Commit the folder.
