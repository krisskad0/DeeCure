# Phase 2: Fix safely (public beta)

[Docs](../README.md) › [Roadmap](README.md)

**Goal:** users can fix problems with a preview, one password prompt, and an undo button. Then DeeCure goes public.
**Rough time:** 6-8 weeks.

## Progress

| Task | Status |
| --- | --- |
| 1. Root helper | ✅ Done. 12 admin operations and 6 read-outs, strict checks, audit log, idle exit ([Helper](../02-architecture/helper.md)). |
| 2. System files | ✅ Done. polkit policy (one action per operation), D-Bus policy and activation, hardened `deecure-helper.service`; in the `.deb`, never enabled or started by it. |
| 3. Access level | ✅ Done. Settings › Access shows whether the helper is installed; onboarding explains Basic and Full. |
| 4. Admin insights | ✅ Done. SMART health, firewall rules, memory modules, and the serial number through the helper, behind a "asks for your password" button. All users' crontabs and account status are available as helper read-outs. |
| 5. Fix engine | ✅ Done. Plan → preview → apply → verify, with undo calls and run history ([Fix engine](../02-architecture/fix-engine.md)). |
| 6. Fix drawer UI | ✅ Done. Untick steps, undo and admin badges, live progress, a verify result; Fix buttons on issues, cleanup, packages, updates, firewall, services, startup apps, processes, power, kernels, and Docker images. |
| 7. First 20 fixes | Built: 22. Tested: unit tests, and the helper path end to end in a container with real D-Bus and polkit. Still to do: apply → verify → undo of each fix in a full VM. |
| 8. Undo screens | ✅ Done. Activity › DeeCure Actions and Backups › Restore Points list every run with Undo; `deecure history` and `deecure undo`. |
| 9. Security review | Needs two human reviewers. The [review checklist](../02-architecture/security-model.md#helper-review-checklist) is met by the code and its tests. |
| 10. Distribution | ✅ Published: pre-releases `v0.1.0-alpha.2`, `v0.1.0-alpha.3`, and `v0.1.0-alpha.4` on GitHub Releases, with `SHA256SUMS`, SBOM, and signed provenance. One-command install from GitHub Releases, tested on clean 24.04 and 26.04 (`make clean-install-test`). The signed APT repository is built and tested but not online: it needs the signing key as the `APT_SIGNING_KEY` secret and GitHub Pages for `gh-pages` (decision #12). |
| 11. User docs | ✅ Done. [Install, update, and uninstall](../07-user-guide/install.md). |
| 12. Open decisions | Need the owner: contact emails (#11), the signing key owner (#12), the final app ID (#6), supported systems (#9), and health score points (#15). |

## Tasks (in order)

1. **Root helper:** D-Bus service, fixed action list, input checks, audit log, exit after 2 idle minutes.
   → [Helper](../02-architecture/helper.md), [ADR 0005](../02-architecture/decisions/0005-root-helper-dbus-polkit.md)
2. **System files:** polkit policy, D-Bus policy and activation file, `deecure-helper.service`. Add them to the `.deb`.
   → [Files on disk](../02-architecture/overview.md#files-on-disk)
3. **Access level:** Basic / Full choice in onboarding and in Settings › Access.
   → [Settings page](../03-ui-design/05-pages/22-settings.md)
4. **"Admin" insights:** disk health (SMART), firewall, all users' cron jobs, and the rest of the catalog's Admin items.
   → [Collectors](../02-architecture/collectors.md)
5. **Fix engine:** plan → preview → protect → apply → verify, plus the undo store.
   → [Fix engine](../02-architecture/fix-engine.md)
6. **Fix drawer UI** and the state changes after each fix.
   → [Overlays](../03-ui-design/02-shell/overlays.md), [Fixes](../03-ui-design/04-behavior/fixes.md)
7. **First 20 fixes.** Each one passes apply → verify → undo in a VM.
   → [Writing rules](../04-development/writing-rules-and-kb.md)
8. **Undo screens:** Activity › DeeCure Actions and Backups › Restore Points.
   → [Activity page](../03-ui-design/05-pages/04-activity.md), [Backups page](../03-ui-design/05-pages/15-backups.md)
9. **Security review of the helper:** two reviewers on every helper change, plus one full review before the beta.
   → [Security model](../02-architecture/security-model.md)
10. **Distribution:** signed APT repository, install script, GitHub Releases, Launchpad PPA optional.
    → [Packaging](../06-packaging/README.md), [APT repository](../06-packaging/apt-repository.md)
11. **User docs:** install, update, and uninstall guide.
    → [Install guide](../07-user-guide/install.md)
12. **Close all open decisions** before going public.
    → [Open decisions](../01-product/open-decisions.md)

## Done when

- [ ] 80% or more of beginners finish "free up space" without help.
- [ ] 95% or more of fixes verify as successful in the VM matrix.
- [ ] `piuparts` passes: install → upgrade → remove → purge leaves nothing behind.
- [ ] A new user installs DeeCure with one command on clean Ubuntu 24.04 and 26.04 VMs.

---

← Previous: [Phase 1](phase-1-mvp.md) · [Index](README.md) · Next: [Phase 3: History](phase-3-history.md) →
