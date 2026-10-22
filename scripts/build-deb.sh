#!/usr/bin/env bash
# Build the deecure .deb into target/deb/ and check it with lintian.
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

# Reuse one Cargo cache between package builds.
export CARGO_TARGET_DIR="$repo/target/deb-cargo"
(cd "$work/src" && dpkg-buildpackage -us -uc -b)

mv "$work"/*.deb "$out"/
rm -rf "$work"
lintian --fail-on error,warning "$out"/deecure_*.deb
echo "Built: $(ls "$out"/deecure_*.deb)"
