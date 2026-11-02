#!/usr/bin/env bash
# The real in-app update, after a release is published: install an older published release,
# turn automatic updates on, and run that release's own updater (deecure-helper --auto-update,
# as deecure-update.timer does). It must download the newest release from GitHub, check it,
# and install it. This tests the updater code people already have, against the new release.
# It changes the system, so it only runs on a throwaway system:
#
#   docker run --rm -v "$PWD:/src:ro" ubuntu:24.04 bash /src/scripts/test-self-update.sh --throwaway 0.1.0-alpha.4 0.1.0-alpha.5
set -euo pipefail

if [ "${1:-}" != "--throwaway" ] || [ -z "${2:-}" ] || [ -z "${3:-}" ]; then
  echo "Usage: $0 --throwaway <older published version> <expected new version>   (only on a container)" >&2
  exit 2
fi
old="$2"
new="$3"
[ "$(id -u)" = 0 ] || { echo "$0: must run as root" >&2; exit 2; }
export DEBIAN_FRONTEND=noninteractive
fail() { echo "FAIL: $*" >&2; exit 1; }

echo "== install the published $old (agent and desktop app)"
apt-get update -qq >/dev/null
apt-get install -y -qq --no-install-recommends curl ca-certificates >/dev/null
work="$(mktemp -d)"
name="${old/-/.}"
base="https://github.com/krisskad0/DeeCure/releases/download/v$old"
curl -fsSL -o "$work/SHA256SUMS" "$base/SHA256SUMS" || fail "could not download SHA256SUMS"
for p in deecure deecure-desktop; do
  curl -fsSL -o "$work/$p.deb" "$base/${p}_${name}-1_amd64.deb" || fail "could not download $p $old"
  sum="$(grep " ${p}_${name}-1_amd64.deb\$" "$work/SHA256SUMS" | cut -d' ' -f1)"
  echo "$sum  $work/$p.deb" | sha256sum -c --quiet - || fail "$p $old does not match its checksum"
done
apt-get install -y -qq --no-install-recommends "$work/deecure.deb" "$work/deecure-desktop.deb" >/dev/null
echo "ok: $(dpkg-query -W -f='${Version}' deecure)"

echo "== automatic update with $old's own updater"
mkdir -p /etc/deecure
printf '{"auto": true, "check": "daily", "channel": "prerelease"}' >/etc/deecure/update.json
chmod 0644 /etc/deecure/update.json
/usr/libexec/deecure/deecure-helper --auto-update || fail "the updater failed: $(cat /run/deecure/update-result.json 2>/dev/null)"
want="${new/-/\~}-1"
for p in deecure deecure-desktop; do
  got="$(dpkg-query -W -f='${Version}' "$p")"
  [ "$got" = "$want" ] || fail "$p is $got, not $want: $(cat /run/deecure/*.json 2>/dev/null)"
done
[ -z "$(dpkg --audit)" ] || fail "dpkg reports half-installed packages"
/usr/libexec/deecure/deecure-helper --version | grep -q "$new\$" || fail "the helper is not the new one"
echo "ok: $old updated itself to $new"
