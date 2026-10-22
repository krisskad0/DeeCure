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
| Release candidate | `0.3.0-rc.1` | `0.3.0~rc1-1~ubuntu24.04` |
| Packaging fix only | — | `0.3.0-2~ubuntu24.04` |

The `~` makes a release candidate sort **before** the final release, so apt upgrades from it.

## Branches and tags

- `main` must always build and pass tests.
- Every release is a signed tag: `v0.3.0`.
- Patch releases for an older version come from a `release/0.3` branch.

## Release checklist

1. CI is green on `main`.
2. Move the "Unreleased" notes in `CHANGELOG.md` under the new version.
3. Run `scripts/bump-version.sh 0.3.0`.
4. Add a `packaging/debian/changelog` entry with `dch`.
5. Commit, create the signed tag, push.
6. CI release workflow, all automatic:
   1. Build the `.deb` for each release and CPU, in clean containers.
   2. Run `lintian`, `piuparts`, and `autopkgtest`.
   3. Create `SHA256SUMS` and sign it.
   4. Create a draft GitHub Release with the files.
   5. Publish to the `beta` component of the APT repository.
7. Smoke test on clean Ubuntu 24.04 and 26.04 VMs: install, open the app, run `deecure status`, upgrade from the last version.
8. Promote to the `main` component and publish the GitHub Release.
9. Upload to the PPA (when it exists).
10. Post the release notes.

## Security releases

Follow [SECURITY.md](../../SECURITY.md). Fix in private, release a patch version, then publish the advisory.
