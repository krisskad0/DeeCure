#!/usr/bin/env bash
# Add .deb files to the signed APT repository.
# Usage: scripts/build-apt-repo.sh <site-dir> <suite> <component> <file.deb>...
#
# <site-dir>/apt is what users download (dists/, pool/, the public key).
# <site-dir>/.reprepro keeps reprepro's database, so later releases add to it.
# The signing key must already be in the GPG keyring (CI imports a signing subkey).
set -euo pipefail
site="$1"; suite="$2"; component="$3"; shift 3
repo="$(cd "$(dirname "$0")/.." && pwd)"
state="$site/.reprepro"
mkdir -p "$state/conf" "$site/apt"
cp "$repo/packaging/apt-repo/conf/distributions" "$state/conf/"
for deb in "$@"; do
  reprepro --basedir "$state" --outdir "$site/apt" --component "$component" includedeb "$suite" "$deb"
done
gpg --armor --export > "$site/apt/deecure-archive-keyring.asc"
cp "$repo/packaging/install.sh" "$site/install.sh"
(cd "$site" && sha256sum install.sh > install.sh.sha256)
echo "Repository in $site/apt"
