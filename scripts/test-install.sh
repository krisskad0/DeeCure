#!/usr/bin/env bash
# Build a signed repository with a throwaway key, then install, reinstall, and remove
# DeeCure with packaging/install.sh. Only on a throwaway system (make install-test).
set -euo pipefail
[ "${1:-}" = "--throwaway" ] || { echo "Usage: $0 --throwaway <deb>..." >&2; exit 2; }
shift
export DEBIAN_FRONTEND=noninteractive
fail() { echo "FAIL: $*" >&2; exit 1; }
apt-get update -qq >/dev/null
apt-get install -y -qq reprepro gnupg curl ca-certificates python3-minimal >/dev/null

echo "== build a signed repository with a throwaway key"
export GNUPGHOME=/tmp/gnupg
mkdir -p -m 0700 "$GNUPGHOME"
gpg --batch --pinentry-mode loopback --passphrase '' --quick-gen-key "DeeCure test <test@example.invalid>" rsa4096 sign 1d 2>/dev/null
/src/scripts/build-apt-repo.sh /site noble main "$@" >/dev/null
[ -f /site/apt/dists/noble/InRelease ] || fail "no signed index"
gpg --verify /site/apt/dists/noble/InRelease 2>/dev/null || fail "the index signature does not verify"
sha256sum -c --status <(cd /site && cat install.sh.sha256) 2>/dev/null || (cd /site && sha256sum -c install.sh.sha256 >/dev/null) || fail "install.sh checksum"
echo "ok: signed index and install.sh with its checksum"

export DEECURE_REPO_URL=file:///site/apt
echo "== dry run changes nothing"
sh /site/install.sh --dry-run >/tmp/dry.txt
[ -e /etc/apt/sources.list.d/deecure.sources ] && fail "dry run wrote the source"
grep -q "install -y --no-install-recommends deecure deecure-desktop" /tmp/dry.txt || fail "dry run does not show the install step"
echo "ok"

echo "== install, twice"
sh /site/install.sh >/tmp/install1.txt 2>&1 || { cat /tmp/install1.txt; fail "install failed"; }
sh /site/install.sh >/tmp/install2.txt 2>&1 || { cat /tmp/install2.txt; fail "second install failed"; }
dpkg-query -W -f='${Package} ${Status}\n' deecure deecure-desktop | grep -c "install ok installed" | grep -q 2 || fail "packages not installed"
grep -q "Signed-By: /etc/apt/keyrings/deecure.asc" /etc/apt/sources.list.d/deecure.sources || fail "source not limited to our key"
deecure --version
echo "ok: installed from the signed repository; running it again is safe"

echo "== unsupported options and systems are refused"
sh /site/install.sh --frobnicate 2>/dev/null && fail "unknown option accepted"
echo "ok"

echo "== uninstall"
sh /site/install.sh --uninstall >/dev/null 2>&1 || fail "uninstall failed"
[ -e /etc/apt/sources.list.d/deecure.sources ] && fail "source left behind"
[ -e /etc/apt/keyrings/deecure.asc ] && fail "key left behind"
dpkg-query -W -f='${Status}\n' deecure 2>/dev/null | grep -q "install ok installed" && fail "package still installed"
echo "ok: package, key, and source removed"

echo "== install from a GitHub Release (no APT repository)"
rel=/tmp/release
mkdir -p "$rel"
cp "$@" "$rel/"
(cd "$rel" && sha256sum ./*.deb | sed 's| \./| |' > SHA256SUMS)
python3 - "$rel" <<'PY'
import json, os, sys
d = sys.argv[1]
assets = [{"name": n, "browser_download_url": f"file://{d}/{n}"} for n in sorted(os.listdir(d))]
json.dump([{"tag_name": "v0.0.0-test.1", "prerelease": True, "draft": False, "assets": assets}], open(f"{d}/releases.json", "w"))
PY
export DEECURE_REPO_URL=file:///nowhere DEECURE_RELEASES_URL="file://$rel/releases.json"
sh /site/install.sh >/tmp/gh.txt 2>&1 || { cat /tmp/gh.txt; fail "install from a release failed"; }
grep -q "newest pre-release" /tmp/gh.txt || fail "the pre-release note is missing"
dpkg-query -W -f='${Package} ${Status}\n' deecure deecure-desktop | grep -c "install ok installed" | grep -q 2 || fail "packages not installed from the release"
[ -e /etc/apt/sources.list.d/deecure.sources ] && fail "a release install must not add an APT source"
# A package that does not match its checksum is never installed.
sh /site/install.sh --uninstall >/dev/null 2>&1
printf 'tampered' >> "$rel/$(ls "$rel" | grep '^deecure_' | head -1)"
sh /site/install.sh >/tmp/bad.txt 2>&1 && fail "a tampered package was installed"
grep -q "does not match its checksum" /tmp/bad.txt || fail "no checksum message"
dpkg-query -W -f='${Status}\n' deecure 2>/dev/null | grep -q "install ok installed" && fail "tampered package installed"
echo "ok: installs from a release, and refuses a tampered package"
echo "PASS: install script end to end"
