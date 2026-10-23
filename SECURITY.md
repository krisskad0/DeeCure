# Security policy

DeeCure runs a helper with root rights. We take security reports seriously.

## Supported versions

| Version | Security fixes |
| --- | --- |
| Latest release | Yes |
| Older releases | No. Please update. |

While DeeCure is on `0.x`, only the latest release gets fixes.

## How releases are protected

- Every release is built and tested by the [release workflow](.github/workflows/release.yml) on GitHub, from a tagged commit. Nobody uploads packages from their own computer.
- Each package, `install.sh`, and `SHA256SUMS` comes with **signed build provenance** (Sigstore, through GitHub attestations): proof of which workflow and which commit made it. An **SBOM** (`deecure-sbom.spdx.json`, the list of every component and its license) is attached and signed too. Check a download with:

  ```bash
  gh attestation verify deecure_0.1.0.alpha.2-1_amd64.deb --repo krisskad0/DeeCure
  ```

- CI actions are pinned to exact commits; dependencies are checked for known vulnerabilities and licenses (`cargo-deny`, `npm audit`), and the code is scanned with CodeQL. The [OpenSSF Scorecard](https://securityscorecards.dev/viewer/?uri=github.com/krisskad0/DeeCure) rates these practices publicly.
- `main` and release tags are protected: changes need review and green checks, and tags cannot be moved or deleted ([repository settings](docs/08-contributing/06-repository-settings.md)).

## How to report a problem

**Do not open a public issue.**

1. Go to the repository's **Security** tab on GitHub.
2. Click **Report a vulnerability**. This sends a private report to the maintainers.

If you cannot use GitHub, email **TODO: add security email before going public**.

Please include:

- The DeeCure version (`deecure --version`) and Ubuntu version (`lsb_release -a`).
- What an attacker can do, and what access they need first.
- Steps to reproduce it.

## What happens next

| Step | Our goal |
| --- | --- |
| We confirm we got your report | Within 3 working days |
| We tell you if we accept it and our plan | Within 14 days |
| We release a fix | As fast as possible. Root helper issues come first. |
| We publish an advisory and credit you | After the fix is out, unless you prefer to stay anonymous |

## What counts most

Highest priority:

- `deecure-helper` (runs as root), its D-Bus and polkit settings.
- Any way to run commands or change files without the user's approval.
- The package signing key, the APT repository, and the install script.
- Data leaving the computer without the user turning it on.

How DeeCure protects itself: [Security model](docs/02-architecture/security-model.md).
