#!/usr/bin/env bash
# Upgrade test: install a published release from GitHub, then upgrade to the new .deb files
# the way the in-app updater does (apt-get --no-install-recommends install), and check that
# everything the new version adds is in place and nothing the old one set up was lost.
# It changes the system, so it only runs on a throwaway system:
#
#   make upgrade-test     # in a clean Ubuntu container
set -euo pipefail

if [ "${1:-}" != "--throwaway" ] || [ -z "${2:-}" ] || [ -z "${3:-}" ]; then
  echo "Usage: $0 --throwaway <old version, such as 0.1.0-alpha.4> <new deecure.deb> [new deecure-desktop.deb]   (only on a container)" >&2
  exit 2
fi
old="$2"
deb="$(realpath "$3")"
app="${4:+$(realpath "$4")}"
[ "$(id -u)" = 0 ] || { echo "$0: must run as root" >&2; exit 2; }
export DEBIAN_FRONTEND=noninteractive
fail() { echo "FAIL: $*" >&2; exit 1; }

echo "== install the published $old"
apt-get update -qq >/dev/null
apt-get install -y -qq --no-install-recommends curl ca-certificates >/dev/null
work="$(mktemp -d)"
# GitHub shows "~" in asset names as ".".
name="${old/-/.}"
base="https://github.com/krisskad0/DeeCure/releases/download/v$old"
for p in deecure deecure-desktop; do
  curl -fsSL -o "$work/$p.deb" "$base/${p}_${name}-1_amd64.deb" || fail "could not download $p $old"
done
curl -fsSL -o "$work/SHA256SUMS" "$base/SHA256SUMS" || fail "could not download SHA256SUMS"
(cd "$work" && for p in deecure deecure-desktop; do
  sum="$(grep " ${p}_${name}-1_amd64.deb\$" SHA256SUMS | cut -d' ' -f1)"
  echo "$sum  $p.deb" | sha256sum -c --quiet - || fail "$p $old does not match its checksum"
done)
apt-get install -y -qq --no-install-recommends "$work/deecure.deb" ${app:+"$work/deecure-desktop.deb"} >/dev/null
before="$(dpkg-query -W -f='${Version}' deecure)"
echo "ok: $before installed"

echo "== upgrade as the in-app updater does"
apt-get install -y -qq --no-install-recommends "$deb" ${app:+"$app"} >/dev/null
after="$(dpkg-query -W -f='${Version}' deecure)"
dpkg --compare-versions "$after" gt "$before" || fail "the version did not go up ($before to $after)"
[ -z "$app" ] || [ "$(dpkg-query -W -f='${Version}' deecure-desktop)" = "$after" ] || fail "the app and the agent have different versions"
[ -z "$(dpkg --audit)" ] || fail "dpkg reports half-installed packages: $(dpkg --audit)"
echo "ok: $before to $after"

echo "== what the old version set up is kept, and what is new is in place"
for unit in sockets.target.wants/deecure-agent.socket default.target.wants/deecure-agent.service; do
  [ -L "/etc/systemd/user/$unit" ] || fail "$unit is no longer enabled"
done
[ -L /etc/systemd/system/multi-user.target.wants/deecure-power.service ] || fail "deecure-power.service is no longer enabled"
[ -L /etc/systemd/system/timers.target.wants/deecure-update.timer ] || fail "deecure-update.timer is no longer enabled"
[ -L /etc/systemd/system/multi-user.target.wants/deecure-helper.service ] && fail "the helper must not be enabled"
for f in /usr/lib/systemd/system/deecure-health.service /usr/lib/systemd/system/deecure-health.timer; do
  [ -e "$f" ] || fail "missing $f"
done
[ -L /etc/systemd/system/timers.target.wants/deecure-health.timer ] || fail "deecure-health.timer is not enabled after the upgrade"
want="${after%-*}"
/usr/libexec/deecure/deecure-helper --version | grep -q "${want/\~/-}$" || fail "the helper is not the new one: $(/usr/libexec/deecure/deecure-helper --version)"
rules="$(find /usr/share/deecure/rules -name '*.yaml' | wc -l)"
[ "$rules" -ge 55 ] || fail "only $rules rules installed"
/usr/libexec/deecure/deecure-helper --health-reads || fail "the daily health reads failed"
grep -q '"drives":' /run/deecure/admin-health.json || fail "no health file"
echo "ok: services and timers kept, deecure-health.timer enabled, $rules rules, health reads work"

echo "== the new agent runs as a normal user"
id deecure-test >/dev/null 2>&1 || useradd -m deecure-test
out="$(su deecure-test -s /bin/sh -c '/usr/libexec/deecure/deecure-agent --version' 2>&1)" || fail "agent: $out"
echo "ok: $out"
