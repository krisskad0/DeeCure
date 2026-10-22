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

## Adding a fixture

1. Run the capture script on a real machine: `scripts/capture-fixture.sh` *(planned)*. It copies the needed `/proc`, `/sys`, journal, and command outputs.
2. Check the output for private data. The script removes user names, host names, IPs, and serial numbers, but look again.
3. Save it as `tests/fixtures/<short-machine-name>/`.
4. Add `expected-findings.yaml` with the findings this machine should produce.
