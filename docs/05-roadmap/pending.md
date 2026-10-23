# Pending work

[Docs](../README.md) › [Roadmap](README.md)

Everything that is not done yet, in one place. **Read this first when you continue the work.** When an item is done, delete it here and note it in the phase's Progress table and in `CHANGELOG.md`.

Last updated: 2026-10-23, after Phase 3.

## 1. Next step: wait for the owner

| Item | Detail |
| --- | --- |
| Phase 4 — Ask DeeCure | Not started. Start only when the owner says so. → [Phase 4](phase-4-assistant.md) |
| After Phase 4 | The owner asked to "optimize everything": speed, size, and code cleanup across the app. |

## 1a. Make the first version complete (before any release work)

The owner's view (2026-10-23): the app is not ready as a first version; too much looks unfinished. Every data source returns real data (checked on a real laptop), but these parts of the UI are fake, stale, or missing. Fix them first, then review every page with the owner.

| Where | Problem |
| --- | --- |
| Settings › General | The spec's "Refresh interval" and "Temperature unit" are missing. |
| Settings › Assistant, Assistant page | Placeholders until Phase 4. |
| Top bar › Machines | "Add a machine" is planned for Phase 5. |
| Every page | Review with the owner which panels look empty or "dummy" on a real computer, and fix each one. |

## 1b. Areas with no checks yet

Overview › Subsystems shows these as **Not checked**. Each needs a cheap, reliable fact and a rule ([Writing rules](../04-development/writing-rules-and-kb.md)).

| Area | Idea for a first rule |
| --- | --- |
| Scheduled Jobs | A timer whose last run failed (needs timer results as a metric or snapshot) |
| Containers | A container port open to every network (needs Docker state when Docker runs) |
| Privacy | Apps allowed to use the camera or microphone (needs a light, read-only count) |

## 2. Heavy checks that need a fast internet connection

These download hundreds of MB, so they wait until the main connection works. All run in throwaway containers and never touch the host.

| Check | Command | State |
| --- | --- | --- |
| Install, use, and purge both packages | `make deb-test` | Agent part passed on 2026-10-23. The desktop part was stopped while it downloaded its libraries (slow network). Run it again: the desktop app now uses the tray library (`libayatana-appindicator3-1`, a Recommends). |
| Admin helper with real D-Bus and polkit | `make helper-test` | Not run since Phase 3. Run it once. |
| Install script against a signed test repository | `make install-test` | Not run since Phase 3. Run it once. |
| CI on GitHub | The Actions page of the repository | Check that the run for the newest push is green (all 4 jobs, including the new `make db-size` step). Poll the GitHub API at most once a minute. |

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
