# Security policy

DeeCure runs a helper with root rights. We take security reports seriously.

## Supported versions

| Version | Security fixes |
| --- | --- |
| Latest release | Yes |
| Older releases | No. Please update. |

While DeeCure is on `0.x`, only the latest release gets fixes.

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
