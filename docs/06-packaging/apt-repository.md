# APT repository

[Docs](../README.md) › [Packaging](README.md)

Our own signed APT repository is the main way users install and update DeeCure.

## Layout

```text
https://krisskad0.github.io/DeeCure/apt/          (provisional address)
├── deecure-archive-keyring.asc         public signing key
├── dists/
│   ├── noble/                         Ubuntu 24.04
│   │   ├── InRelease                  signed index
│   │   ├── main/binary-amd64/         stable releases
│   │   └── beta/binary-amd64/         pre-releases
│   └── resolute/                      Ubuntu 26.04
└── pool/main/d/deecure/
    └── deecure_0.3.0-1~ubuntu24.04_amd64.deb
```

- **Suite** = the Ubuntu codename (`noble`, `resolute`).
- **Component** = `main` for stable, `beta` for pre-releases.

## What is built

| Piece | File |
| --- | --- |
| Install script | `packaging/install.sh` (options `--beta`, `--server`, `--dry-run`, `--uninstall`) |
| Repository builder | `scripts/build-apt-repo.sh <site> <suite> <component> <debs...>` (reprepro; its database lives in `<site>/.reprepro`, so releases add up) |
| Suites | `packaging/apt-repo/conf/distributions` (`noble`, `resolute`; components `main`, `beta`) |
| Release workflow | `.github/workflows/release.yml`: on a `v*` tag, builds and tests the packages, makes `SHA256SUMS`, creates the GitHub Release (published for pre-releases, a draft for stable releases), and (when the `APT_SIGNING_KEY` secret exists) publishes to the `gh-pages` branch |
| Version bump | `scripts/bump-version.sh 0.2.0` |
| Test | `make install-test`: in a clean container, builds a repository signed with a throwaway key, checks the signature, then runs `install.sh` (dry run, install twice, unknown option, uninstall), and installs from a local copy of a GitHub Release (including refusing a tampered package) |

## Tools and hosting

- Build the repository with `reprepro`. Its config lives in `packaging/apt-repo/conf/`.
- Host it on GitHub Pages to start. That is fine while the repo stays under 1 GB. Move to a CDN or a package host (for example Cloudsmith) when it grows.
- The release workflow in CI updates it. Nobody uploads by hand.

## Signing key

- One OpenPGP key: RSA 4096. Every apt version accepts it.
- Keep the primary key offline. CI holds only a signing subkey, stored as a CI secret.
- The key expires after 2 years. Extend it at least 3 months before that date.
- Publish the key fingerprint in the README, in the install guide, and on the website.

## Manual setup (what users run)

```bash
sudo install -d -m 0755 /etc/apt/keyrings
curl -fsSL https://krisskad0.github.io/DeeCure/apt/deecure-archive-keyring.asc \
  | sudo tee /etc/apt/keyrings/deecure.asc > /dev/null

sudo tee /etc/apt/sources.list.d/deecure.sources > /dev/null <<EOF
Types: deb
URIs: https://krisskad0.github.io/DeeCure/apt
Suites: $(. /etc/os-release && echo "$VERSION_CODENAME")
Components: main
Signed-By: /etc/apt/keyrings/deecure.asc
EOF

sudo apt update
sudo apt install deecure
```

This uses the modern `.sources` format (called deb822), the same format Ubuntu 24.04 uses for its own sources. `Signed-By` makes apt trust our key **only** for our repository.

## Install script rules

`packaging/install.sh` does exactly the manual steps above, nothing more.

- Short and plain, so people can read it before running it.
- Checks first: a supported Ubuntu or Debian release, a supported CPU, and root rights.
- Safe to run twice.
- Options: `--beta` (use the beta component), `--dry-run` (print the steps only), `--uninstall` (remove the package, the key, and the source file).
- It never changes other repositories or system settings.
- Its SHA-256 checksum is published next to it.

## Launchpad PPA (after the beta)

A PPA is Ubuntu's own way to host a third-party repository: `sudo add-apt-repository ppa:deecure/stable`.

Launchpad builds packages from source on servers **without internet**. So the source package must include:

- All Rust crates, vendored with `cargo vendor`.
- The built web UI, or vendored Node packages.
- A Rust version from the Ubuntu archive (versioned packages such as `rustc-1.xx`).

Version suffix for PPA builds: `0.3.0-1~ppa1~ubuntu24.04.1`.
