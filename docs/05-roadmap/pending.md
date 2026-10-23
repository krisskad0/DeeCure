# Pending work

[Docs](../README.md) › [Roadmap](README.md)

Everything that is not done yet, in one place. **Read this first when you continue the work.** When an item is done, delete it here and note it in the phase's Progress table and in `CHANGELOG.md`.

Last updated: 2026-10-23, after Phase 3 and the first pre-release.

## Recommended order

1. **First version complete** (owner and developer, no internet needed): the owner reviews every page and names what looks empty or "dummy"; fix each one (1a). Real-desktop checks: a test notification, the tray switch (5).
2. **Heavy checks** with a fast connection: `make deb-test`, `make helper-test`, `make install-test`, and one real install on clean Ubuntu 24.04 and 26.04 (2, 2b).
3. **Owner decisions** before a stable release: contact emails (#11), app ID (#6), signing key (#12, also turns on the APT repository and signed commits and tags), supported systems (#9), health points (#15); optional: domain, Launchpad, trademark, Best Practices badge (2d, 4).
4. **People and machines:** the 22 fixes in real VMs, 5-10 beginner testers, a second maintainer and a full helper security review (3, 5).
5. **Public beta:** fix the findings, release `v0.1.0-beta.1`, then `v0.1.0`; when contributors arrive, admin bypass to "pull requests only" and CodeQL and Dependencies as required checks (2c, 2d).
6. **Next features, on the owner's word:** Phase 4 (assistant), "optimize everything", Phase 5, coverage-guided fuzzing (1, 2d).

## 1. Next step: wait for the owner

| Item | Detail |
| --- | --- |
| Phase 4: Ask DeeCure | Not started. Start only when the owner says so. → [Phase 4](phase-4-assistant.md). The Assistant page and Settings › Assistant are hidden until then: remove them from `LATER_PAGES` and `LATER_TABS` in `app/src/visible.ts`. |
| Phase 5: many PCs | The machines menu shows only this computer; add "Add a machine" back with the multi-PC view. |
| After Phase 4 | The owner asked to "optimize everything": speed, size, and code cleanup across the app. |

## 1a. Make the first version complete (before any release work)

The owner's view (2026-10-23): the app is not ready as a first version; too much looks unfinished. Every data source returns real data (checked on a real laptop), but these parts of the UI are fake, stale, or missing. Fix them first, then review every page with the owner.

| Where | Problem |
| --- | --- |
| Every page | Review with the owner which panels look empty or "dummy" on a real computer, and fix each one. |

## 2. Heavy checks that need a fast internet connection

These download hundreds of MB, so they wait until the main connection works. All run in throwaway containers and never touch the host.

| Check | Command | State |
| --- | --- | --- |
| Install, use, and purge both packages | `make deb-test` | ✅ Passed on 2026-10-23 |
| Admin helper with real D-Bus and polkit | `make helper-test` | ✅ Passed on 2026-10-23 |
| Install script against a signed test repository, and from a GitHub Release | `make install-test` | ✅ Passed on 2026-10-23 |
| Real one-command install from GitHub on clean 24.04 (curl) and 26.04 (wget) | `make clean-install-test` | ✅ Passed on 2026-10-23 (containers). Still to do: once on a real desktop with someone signed in, to see the agent start at once (the fix for "agent unreachable" right after installing). |
| CI on GitHub | `gh run list` (gh is logged in as `krisskad0`) | Green on 2026-10-23 (CI, CodeQL, Dependencies, Scorecard). Check the newest push after each change; poll at most once a minute. |

## 2b. Releases

- **v0.1.0-alpha.2** is published (pre-release, 2026-10-23): `curl -fsSL https://raw.githubusercontent.com/krisskad0/DeeCure/main/packaging/install.sh | sudo sh` installs it. Checked with `--dry-run` against the real release.
- Still to do: one real install on a clean Ubuntu 24.04 and 26.04 machine or VM (not the development laptop).
- The tag `v0.1.0-alpha.1` has no release (its build failed on a timing problem, since fixed). It can be deleted from GitHub, or left.
- Next releases: `scripts/bump-version.sh <version>`, CHANGELOG, `dch`, commit, wait for green CI, then tag. Pre-release tags publish at once.

## 2c. Repository protection

Done on 2026-10-23 through the GitHub API (as `krisskad0`): the 5 rulesets are active, squash merge only, branches deleted after merge, Discussions on, wiki off, Actions approval for all outside contributors, read-only default workflow permissions, private vulnerability reporting, Dependabot alerts and security updates, secret scanning with push protection, and the 27 labels. Still the owner's choice later: change the admin bypass on `main` to "pull requests only" when contributors arrive ([Repository settings](../08-contributing/06-repository-settings.md)).

## 2d. Security practices still open

Done on 2026-10-23: actions pinned to commits, least-privilege release workflow, signed build provenance and SBOM for releases, cargo-deny and npm audit, CodeQL, OpenSSF Scorecard, randomized tests of the helper's input checks.

| Open item | Needs |
| --- | --- |
| Contact emails in `SECURITY.md` and `CODE_OF_CONDUCT.md` (still "TODO") | Owner (decision #11) |
| Signed commits and tags | The owner's SSH or GPG signing key (`git config commit.gpgsign true`, `git tag -s`) |
| A second maintainer, so two-person review of the helper can really happen | People |
| OpenSSF Best Practices badge | Owner registers at bestpractices.dev |
| Coverage-guided fuzzing (`cargo-fuzz`, nightly CI job) | Developer time and a fast connection |
| Make **Dependencies** and **CodeQL** required checks, once they have been green for a few weeks | Owner (ruleset) |

## 3. Phase 2 items that need people, machines, or the owner

→ [Phase 2](phase-2-fix-safely.md)

| Item | Detail |
| --- | --- |
| Fixes in a real VM | Apply → verify → undo each of the 22 fixes in a full Ubuntu VM (24.04 and 26.04). Unit tests and the container helper test already pass. |
| Security review of the helper | Two human reviewers, using the [review checklist](../02-architecture/security-model.md#helper-review-checklist). |
| Go live with the APT repository | Create the signing key and add it as the `APT_SIGNING_KEY` secret, turn on GitHub Pages for `gh-pages`, then push a `v*` tag. → [APT repository](../06-packaging/apt-repository.md) |
| Phase 2 "Done when" | 80% of beginners finish "free up space" alone; 95% of fixes verify in the VM matrix; `piuparts` passes; one-command install works on clean 24.04 and 26.04 VMs. |

## 4. Owner decisions

→ [Open decisions](../01-product/open-decisions.md)

| # | Decision | Default in use now |
| --- | --- | --- |
| 6 | Final app ID | `io.github.krisskad0.DeeCure` (`deecure_core::APP_ID`). It must never change after the first release. |
| 9 | Supported systems | Ubuntu 24.04 and 26.04 on amd64 first |
| 11 | Contact emails | Placeholder `krisskad0@users.noreply.github.com` |
| 12 | APT signing key owner | Not chosen |
| 15 | Health score points | −15 critical, −5 warning (the plan) |
| 1 | Before launch | Create the Launchpad team `deecure`, register `deecure.com`, run a legal trademark search |

## 5. Phase 3 items that need people or other desktops

→ [Phase 3](phase-3-history.md)

| Item | Detail |
| --- | --- |
| Beginner test | Do the timeline, the "new kernel" tip, and the notifications make sense to beginners? |
| Tray on other desktops | Tested on Ubuntu's GNOME logic only (needs a tray host, `org.kde.StatusNotifierWatcher`). Try KDE, and GNOME without the AppIndicator extension (the switch must say "no tray area"). |
| Real desktop notification | The D-Bus call was tested on a private bus. Press **Send a test notification** in Settings › Notifications once on a real desktop. |

## How to work (owner's rules)

- Build in small steps. After each working step: run the checks, commit, and push to `origin main`.
- Never add AI or assistant names to commits, files, or GitHub.
- Never change the host system: no service restarts, no fixes, no root commands. Test write paths only in containers or fixtures.
- Write files that contain shell commands with an editor tool, never through a shell heredoc.
- Keep docs small: one topic per file, in the numbered folders.
