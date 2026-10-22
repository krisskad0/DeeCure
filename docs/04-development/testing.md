# Testing

[Docs](../README.md) › [Development](README.md)

1. **Fixture tests:** capture real `/proc`, `/sys`, journal, and command outputs from many machines, then run collectors and rules against them offline.
2. **VM matrix:** run the full app on supported Ubuntu LTS releases, desktop and server, in VMs on every release.
3. **Hardware lab:** test on a few real laptops (Intel, AMD, NVIDIA, ThinkPad, budget laptop) because sensors and batteries differ.
4. **Fix safety tests:** each fix must pass apply → verify → undo in a VM before it ships.
5. **Beginner tests:** 5 non-technical users try three tasks ("free up space", "why is it slow", "what changed"). Measure success without help.

## Tools

| Test type | Tool | Runs |
| --- | --- | --- |
| Rust unit and integration tests | `cargo test` | Every pull request |
| UI unit tests | Vitest + React Testing Library | Every pull request |
| Lint and format | `cargo fmt`, `cargo clippy -D warnings`, ESLint, Prettier, `tsc` | Every pull request |
| Fixture tests | `cargo test` against `tests/fixtures/` | Every pull request |
| Dependency checks | `cargo-deny`, `cargo-audit`, `npm audit` | Every pull request and nightly |
| End-to-end UI tests | WebDriver with `tauri-driver` | Nightly and before release |
| Package checks | `lintian` (package rules), `piuparts` (install → remove → purge leaves nothing behind), `autopkgtest` (smoke tests on the installed package) | Every release build |
| VM matrix | LXD or Multipass VMs: Ubuntu 24.04 and 26.04, desktop and server | Before every release |
| Performance budget | A script that measures agent CPU and RAM against the [budget](../02-architecture/performance-budget.md) | Every pull request |

## Where tests live

| Path | What |
| --- | --- |
| `crates/<crate>/src/**` | Rust unit tests, next to the code |
| `crates/<crate>/tests/` | Rust integration tests |
| `app/src/**/*.test.tsx` | UI component tests |
| `tests/fixtures/<machine>/` | Captured system data from one real machine, plus expected findings |
| `tests/vm/` | Scripts that build VMs, install the package, and run checks |
| `packaging/debian/tests/` | `autopkgtest` smoke tests |

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
