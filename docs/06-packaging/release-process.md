# Release process

[Docs](../README.md) › [Packaging](README.md)

How we number versions and ship a release.

## Version numbers

- We use Semantic Versioning: `MAJOR.MINOR.PATCH`, for example `0.3.0`.
- We stay on `0.x` until the first stable release.
- One version everywhere: `Cargo.toml`, `app/package.json`, `tauri.conf.json`, `packaging/debian/changelog`. The script `scripts/bump-version.sh` changes them all at once.

| Kind | Upstream version | Debian version |
| --- | --- | --- |
| Release | `0.3.0` | `0.3.0-1~ubuntu24.04`, `0.3.0-1~ubuntu26.04` |
| Pre-release (alpha, beta, release candidate) | `0.3.0-alpha.1`, `0.3.0-beta.1`, `0.3.0-rc.1` | `0.3.0~alpha.1-1`, `0.3.0~rc.1-1` |
| Packaging fix only | - | `0.3.0-2~ubuntu24.04` |

The `~` makes a pre-release sort **before** the final release, so apt upgrades from it.

## Branches and tags

- `main` must always build and pass tests.
- Every release is a signed tag: `v0.3.0`.
- Patch releases for an older version come from a `release/0.3` branch.

## Release checklist (as used for `v0.1.0-alpha.3`)

1. Run `scripts/bump-version.sh 0.3.0`. It prints the next steps.
2. Move the "Unreleased" notes in `CHANGELOG.md` under the new version. Give the AppStream entry the script added in `packaging/desktop/*.metainfo.xml` a description.
3. Add a `packaging/debian/changelog` entry: in `packaging/`, run `dch -v 0.3.0-1 -D noble "…"` (a pre-release such as `0.1.0-alpha.3` is `0.1.0~alpha.3-1`) with `DEBFULLNAME="DeeCure maintainers"` and `DEBEMAIL="krisskad0@users.noreply.github.com"`.
4. `make release-check`: builds the packages once, then runs, at the same time, the upgrade from the newest published release to these packages and every page with this computer's real answers (`ui-real`). The install and helper tests are not run here: CI and the release job run them on every build.
5. Commit with a sign-off (`git commit -s`), push, and tag right away: `git tag -a v0.3.0 -m "…"`, `git push origin v0.3.0`. No need to wait for CI first: the release job builds while CI runs, and its "CI passed on this commit" job holds publishing until the CI workflow passed on that exact commit (it fails after 55 minutes, or when CI fails). (Tags become signed once the signing key exists, decision #12.)
6. The release workflow, all automatic:
   1. Checks the tag matches the version in `Cargo.toml`.
   2. Builds the two `.deb` files and checks them with `lintian`.
   3. Installs, uses, and purges them, and tests the helper with real D-Bus and polkit, in clean containers.
   4. Gives the files GitHub-safe names (GitHub turns `~` into `.`), writes `SHA256SUMS` and an SBOM.
   5. Waits until CI passed on the tagged commit; nothing below runs before that.
   6. Signs build provenance for every file (Sigstore, no long-lived key).
   7. Creates the GitHub Release with the two `.deb` files, `SHA256SUMS`, the SBOM, and `install.sh`. A **pre-release tag** (with `-alpha`, `-beta`, or `-rc`) is **published at once**, so the one-command installer can find it. A **stable tag** is created as a **draft**, to publish by hand after step 7.
   8. Updates the APT repository, only when the `APT_SIGNING_KEY` secret exists (not yet).
7. Check: `gh release view v0.3.0`, then `make after-release-test`. It runs, all at the same time (about 3 to 4 minutes instead of 11; 192 s for 0.1.0-alpha.10): the real one-command install from GitHub on clean 24.04 and 26.04 containers, which must get the new version; the real in-app update from the release before; and `update-systemd-test` (the in-app and automatic update, a repair with `install.sh`, and the helper's sandbox, with systemd running as on a desktop; the other container tests cannot see a package that stops a running service). Each check has its own log under `target/parallel-*`. Before a stable release, also smoke test on real 24.04 and 26.04 VMs: install, open the app, run `deecure status`, upgrade from the last version.
8. Publish the GitHub Release (stable only). Later: promote to the `main` APT component, upload to the PPA.
9. Post the release notes. The GitHub release already carries them (`scripts/release-notes.sh` copies the version's section of `CHANGELOG.md`; the release fails without one), and the app shows them before and after an update, so write the changelog entries for users: what they will notice, in plain words.

Planned for the release workflow: `piuparts` and `autopkgtest` ([pending work](../05-roadmap/pending.md)).

## Security releases

Follow [SECURITY.md](../../SECURITY.md). Fix in private, release a patch version, then publish the advisory.
