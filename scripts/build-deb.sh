#!/usr/bin/env bash
# Build the deecure and deecure-desktop .debs into target/deb/ and check them with lintian.
# Usage: scripts/build-deb.sh        (or: make deb)
set -euo pipefail

repo="$(cd "$(dirname "$0")/.." && pwd)"
out="$repo/target/deb"
work="$out/build"
rm -rf "$work" "$out"/*.deb
mkdir -p "$work/src"

# A clean copy of the source: every file git tracks, plus new files that are not ignored.
(cd "$repo" && git ls-files -z --cached --others --exclude-standard \
  | tar --null --ignore-failed-read -T - -cf -) 2>/dev/null | tar -x -C "$work/src"
cp -r "$repo/packaging/debian" "$work/src/debian"

# The desktop app embeds the built web UI. Node is needed here, not inside dpkg-buildpackage.
echo "== building the web UI"
(cd "$work/src/app" && npm ci --no-audit --no-fund --loglevel=error && npm run build >/dev/null)

# Reuse one Cargo cache between package builds.
export CARGO_TARGET_DIR="$repo/target/deb-cargo"
(cd "$work/src" && dpkg-buildpackage -us -uc -b)

mv "$work"/*.deb "$out"/
rm -rf "$work"
lintian --fail-on error,warning "$out"/deecure_*.deb "$out"/deecure-desktop_*.deb
echo "Built:"; ls "$out"/*.deb
