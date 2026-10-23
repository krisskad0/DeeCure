# Pending work and handoff

[Docs](../README.md) › [Roadmap](README.md)

**Read this first when you continue the work.** It says where the project stands, what is left in order, and how to work here. When an item is done, delete it here and note it in the phase's Progress table and in `CHANGELOG.md`.

Last updated: 2026-10-23, after the release `v0.1.0-alpha.3`, the faster installer, and the light Beginner mode.

## 1. Where the project stands

| Area | State |
| --- | --- |
| Phases 0-3 | Built: agent, history, rules, desktop app with real data on every page, safe fixes with undo, root helper, notifications, weekly report, tray, CLI ([roadmap](README.md)) |
| Checks | 16 rules; all 13 areas in Overview › Subsystems are checked |
| Releases | `v0.1.0-alpha.3` is the newest (pre-release). `v0.1.0-alpha.2` before it. The tag `v0.1.0-alpha.1` has no release (its build failed; harmless). |
| Install | One command, from GitHub Releases: `curl -fsSL https://raw.githubusercontent.com/krisskad0/DeeCure/main/packaging/install.sh \| sudo sh` ([install guide](../07-user-guide/install.md)). `install.sh` is served from `main`, so installer fixes reach users without a release. The signed APT repository is built but not online (needs the signing key, decision #12). |
| Tests | All green on 2026-10-23: CI (lint, tests, web, perf, `.deb`), CodeQL, Dependencies, Scorecard; container tests `make deb-test`, `make helper-test`, `make install-test`, `make clean-install-test` (real one-command install on clean 24.04 with curl and 26.04 with wget) |
| Performance | First result 1.5 s, idle CPU 0.11%, peak RAM 18.9 MB, 2 years of history 124 MB (`make perf`, `make db-size`) |
| GitHub | Protected by 5 rulesets (applied through the API): pull requests with review and 6 required checks on `main` and `release/*`, squash only, tags locked; owner `krisskad0` has an admin bypass. Security features on. 27 labels. Dependabot monthly. |
| Docs | Current: architecture, UI, development, packaging, user guide (install, everyday use), contributing guide ([08-contributing](../08-contributing/README.md)) |

## 2. Recommended order of what is left

0. **Expert build (approved by the owner on 2026-10-23, in progress).** Build all of [View levels](../01-product/view-levels.md), [Expert coverage](../01-product/expert-coverage.md), and [Optimize](../01-product/optimize-catalog.md), in this order. Test each step once, commit, push, and tick it here:
   1. ✅ Three view levels (Beginner, Advanced, Expert): menus, top bar, Settings, first-run screens.
   2. ✅ Tab merges (89 to 77), with old links opening the merged tab at the right part.
   3. Expert depth without admin, area by area (✅ CPU: Performance › CPU, Hardware › Processor, Power › Sleep; ✅ memory: Performance › Memory, Hardware › Memory; ✅ drives and filesystems: Disk I/O, Storage Devices, Filesystems, PCIe links on Peripherals; next: graphics), in this order: memory, drives, filesystems, graphics, network, battery and sleep, temperature, processes drawer, services and boot, kernel limits, hardware and firmware, ports and USB-C, built-in devices, display and sound, hardware errors, software, security, containers.
   4. Power controls (charge limits and mode, wake sources, device power saving, turbo, keyboard backlight, radios, lid), each with undo.
   5. Optimize page, item by item, each on a measured trigger.
   6. Admin reads in the helper, one by one with its security review (BIOS settings, memory slots, power meters, NVMe details, self-tests, wake-up tracing, low-power residency, sudo rules, `sshd -T`); eBPF traffic counters last.
1. **First version complete** (owner and developer, no downloads): the owner reviews every page and names what looks empty or "dummy"; fix each one (section 4). Then the real-desktop checks in section 3.
2. **Owner decisions** before a stable release (section 5): contact emails (#11), app ID (#6), signing key (#12, also turns on the APT repository and signed commits and tags), supported systems (#9), health points (#15).
3. **People and machines** (section 6): the 22 fixes in real VMs, 5-10 beginner testers, a second maintainer, a full security review of the helper.
4. **Public beta:** fix the findings, release `v0.1.0-beta.1`, then `v0.1.0` ([how to release](#7-how-to-release)). When contributors arrive: admin bypass on `main` to "pull requests only", and CodeQL and Dependencies as required checks.
5. **Next features, on the owner's word:** Phase 4 (assistant; un-hide it in `app/src/visible.ts`), use per app (disk I/O, power, and network traffic; their panels are designed but hidden until a collector exists), "optimize everything", Phase 5 (bring back "Add a machine"), coverage-guided fuzzing.

## 3. Checks only a real desktop can do

Containers have nobody signed in, so these need a real Ubuntu desktop (not done yet):

| Check | How |
| --- | --- |
| The agent starts right after installing | Install with the one command while signed in, open DeeCure at once: no "agent unreachable" (fixed in alpha.3: the package starts the agent for everyone signed in, and the app starts it if needed) |
| A real notification | Settings › Notifications › **Send a test notification** |
| The tray | Settings › General › Tray icon, on Ubuntu's GNOME; then KDE, and GNOME without the AppIndicator extension (the switch must say "no tray area") |
| Install speed | Time the one command on a normal desktop (expected 40-50 MB of Ubuntu libraries; a bare system needs about 120 MB) |

## 4. First version: what the owner still has to review

All 91 tabs show real data and every data source was checked on a real laptop. Fake or stale parts found so far are fixed (access level, Settings › Data and General, hidden placeholders for the assistant and multi-PC). Beginner mode (the default) now shows only 8 pages and 22 sub-pages, so the app feels light for newcomers ([Beginner mode](../03-ui-design/02-shell/sidebar.md#beginner-mode)); the list is `BEGINNER` in `app/src/visible.ts`. What is left is the owner's eye: which panels look empty or unclear on a real computer, and whether the Beginner list is right. Fix each one, page by page.

## 5. Owner decisions

→ [Open decisions](../01-product/open-decisions.md)

| # | Decision | Default in use now |
| --- | --- | --- |
| 11 | Contact emails | `SECURITY.md` and `CODE_OF_CONDUCT.md` still say "TODO"; package maintainer `krisskad0@users.noreply.github.com` |
| 6 | Final app ID | `io.github.krisskad0.DeeCure` (`deecure_core::APP_ID`). Must never change after a stable release. |
| 12 | APT signing key | Not created. The owner creates it offline, adds the subkey as the `APT_SIGNING_KEY` secret, and turns on GitHub Pages for `gh-pages` ([APT repository](../06-packaging/apt-repository.md)). Also allows signed commits and tags. |
| 9 | Supported systems | Ubuntu 24.04 and 26.04 on amd64 |
| 15 | Health score points | −15 critical, −5 warning, at most 15 per area |
| 1 | Before launch | Register `deecure.com`, the Launchpad team `deecure`, a trademark search; optional OpenSSF Best Practices badge |
| - | Two GitHub accounts | The repository belongs to `krisskad0`; commits show `krisskad1` (the work email). If both are the owner's, add `krisskad1` as admin and to CODEOWNERS. |

## 6. Needs people or machines

| Item | Detail |
| --- | --- |
| Fixes in real VMs | Apply → verify → undo each of the 22 fixes in full Ubuntu 24.04 and 26.04 VMs (unit tests and the container helper test pass) |
| Security review of the helper | Two people, with the [review checklist](../02-architecture/security-model.md#helper-review-checklist) |
| Beginner tests | 5-10 people: can they free up space alone? Do the timeline, the "new kernel" tip, and the notifications make sense? |
| A second maintainer | So two-person review of the helper can really happen |
| Coverage-guided fuzzing | `cargo-fuzz` on the helper's input checks, as a nightly job (randomized tests already run in `cargo test`) |
| `piuparts`, `autopkgtest` | Install, upgrade, and purge checks for the packages |

## 7. How to release

1. `scripts/bump-version.sh <version>` (such as `0.1.0-beta.1`); it prints the next steps.
2. Move the `Unreleased` notes in `CHANGELOG.md` under the version, and give the AppStream entry it added in `packaging/desktop/*.metainfo.xml` a description.
3. In `packaging/`: `dch -v <debian version> -D noble "…"` with `DEBFULLNAME="DeeCure maintainers"` and `DEBEMAIL="krisskad0@users.noreply.github.com"` (the Debian version of `0.1.0-beta.1` is `0.1.0~beta.1-1`).
4. Commit (`git commit -s`), push, and wait until CI is green on that exact commit (`gh run list --commit <sha>`).
5. `git tag -a v<version> -m "…"` and `git push origin v<version>`. The release workflow builds, tests, signs provenance and the SBOM, and publishes. Pre-releases publish at once; stable releases are drafts to publish by hand.
6. Check: `gh release view v<version>`, then `make clean-install-test` (the real one-command install against the published release).

## 8. How to work here (owner's rules and practical notes)

- Build in small steps. After each working step: run the checks, commit with a sign-off (`git commit -s`), and push to `origin main`. Conventional Commits.
- Never add AI or assistant names to commits, files, or GitHub. Never use the long dash or the range dash in any text.
- Never change the host system: no service restarts, no fixes, no root commands. Test write paths only in containers or fixtures. Running the agent read-only is fine.
- Write files that contain shell commands with an editor tool, never through a shell heredoc (a nested heredoc once ran doc text as commands).
- Keep docs small: one topic per file, in the numbered folders. Update this file at the end of every session.
- **Tools:** Rust from `~/.cargo/bin`, Node from `~/.nvm/versions/node/v22.20.0/bin`. `gh` is logged in as `krisskad0` (repo admin): use it for CI status, logs, releases, and settings. Poll GitHub at most once a minute.
- **Dev agent:** `make agent` (uses `rules/` and `kb/` from the repo and the real database `~/.local/share/deecure/deecure.db`, schema 2). After changing agent code, stop it (Ctrl+C or `kill -INT`) and start it again, or the app talks to the old agent ("Agent needs a restart"). The app runs with `make app` (live reload).
- **Throwaway agent:** `./target/debug/deecure-agent --rules rules --db <scratch>/x.db --socket /tmp/<short>/a.sock`; socket paths must stay under about 100 characters.
- **Long tests** (`make deb`, container tests, `make perf`) take minutes: run them in the background and read their logs. Build `make deb` once, then run the container tests on `target/deb/` (each `make` test target would rebuild); never rebuild while a container test runs.
- **CI is lean on purpose:** docs-only changes skip the heavy jobs (`What changed`), CodeQL and Dependencies run only for code or dependency changes, Scorecard weekly, Dependabot monthly. Keep it that way: no redundant runs.
- **Before every commit:** `cargo fmt --all --check` (a skipped format once turned CI red), `git grep -n -P '\x{2013}|\x{2014}'` (the two dashes) and `git grep -n -i -E 'cl[a]ude|anthr[o]pic'` (assistant names) find nothing.
