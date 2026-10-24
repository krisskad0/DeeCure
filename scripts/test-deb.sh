#!/usr/bin/env bash
# Install the .deb, use it as a normal user, then purge it and check nothing is left.
# It changes the system, so it only runs on a throwaway system (a container or CI):
#
#   make deb-test                                         # in a clean Ubuntu container
#   sudo scripts/test-deb.sh --throwaway target/deb/deecure_*.deb   # in CI
set -euo pipefail

if [ "${1:-}" != "--throwaway" ] || [ -z "${2:-}" ]; then
  echo "Usage: $0 --throwaway <deecure.deb> [deecure-desktop.deb]   (only on a container or CI machine)" >&2
  exit 2
fi
deb="$(realpath "$2")"
app="${3:+$(realpath "$3")}"
[ "$(id -u)" = 0 ] || { echo "$0: must run as root" >&2; exit 2; }
export DEBIAN_FRONTEND=noninteractive
fail() { echo "FAIL: $*" >&2; exit 1; }

echo "== install"
apt-get update -qq >/dev/null
apt-get install -y -qq "$deb" >/dev/null
dpkg-query -W -f='${Package} ${Version} ${Status}\n' deecure

echo "== enabled for every user"
for unit in sockets.target.wants/deecure-agent.socket default.target.wants/deecure-agent.service; do
  [ -L "/etc/systemd/user/$unit" ] || fail "$unit is not enabled"
  echo "ok: $unit"
done

echo "== admin helper (installed, never started by the package)"
for f in /usr/libexec/deecure/deecure-helper /usr/lib/systemd/system/deecure-helper.service \
  /usr/lib/systemd/system/deecure-power.service \
  /usr/share/dbus-1/system-services/io.github.krisskad0.DeeCure.Helper.service \
  /usr/share/dbus-1/system.d/io.github.krisskad0.DeeCure.Helper.conf \
  /usr/share/polkit-1/actions/io.github.krisskad0.deecure.policy; do
  [ -e "$f" ] || fail "missing $f"
done
/usr/libexec/deecure/deecure-helper --version
[ -L /etc/systemd/system/multi-user.target.wants/deecure-helper.service ] && fail "the helper must not be enabled"
# Enabled for boot, but it does nothing until a power setting is kept.
[ -L /etc/systemd/system/multi-user.target.wants/deecure-power.service ] || fail "deecure-power.service is not enabled"
[ -e /etc/deecure/power.json ] && fail "the package must not keep any power setting"
# Run as a normal user, it refuses to start.
id deecure-test >/dev/null 2>&1 || useradd -m deecure-test
if su deecure-test -c /usr/libexec/deecure/deecure-helper 2>/dev/null; then fail "the helper ran without root"; fi
grep -c '<action id=' /usr/share/polkit-1/actions/io.github.krisskad0.deecure.policy
echo "ok: helper files in place, not enabled, refuses to run without root"

echo "== commands"
deecure --version
/usr/libexec/deecure/deecure-agent --version
# Check the package, not the disk: minimal container images skip installing man pages.
dpkg-deb -c "$deb" | grep -q "usr/share/man/man1/deecure.1.gz" || fail "no man page in the package"

echo "== run as a normal user"
id deecure-test >/dev/null 2>&1 || useradd -m deecure-test
su deecure-test -c '
  set -e
  /usr/libexec/deecure/deecure-agent --socket /tmp/deecure-test.sock 2>/tmp/deecure-test.log &
  agent=$!
  for i in $(seq 1 50); do [ -S /tmp/deecure-test.sock ] && break; sleep 0.1; done
  # The first start also imports history; wait until the agent answers (at most 30 s).
  ok=no
  for i in $(seq 1 30); do
    if deecure --socket /tmp/deecure-test.sock status >/tmp/deecure-status.txt 2>&1; then ok=yes; break; fi
    sleep 1
  done
  cat /tmp/deecure-status.txt
  if [ "$ok" != yes ]; then echo "the agent did not answer within 30 s; its log:"; cat /tmp/deecure-test.log; exit 1; fi
  deecure --socket /tmp/deecure-test.sock status --json > /tmp/deecure-test.json
  kill $agent || true
  # Every installed rule must load: the count in the log matches the files shipped.
  rules=$(find /usr/share/deecure/rules -name "*.yaml" | wc -l)
  grep -q "$rules rules from /usr/share/deecure/rules" /tmp/deecure-test.log || { echo "rules did not all load:"; cat /tmp/deecure-test.log; exit 1; }
  grep -q "knowledge base from /usr/share/deecure/kb" /tmp/deecure-test.log
  # "! grep" would not stop the script under set -e, so test it explicitly.
  if grep "knowledge base: " /tmp/deecure-test.log; then exit 1; fi
  [ "$(stat -c %a ~/.local/share/deecure/deecure.db)" = 600 ]
'
grep -q '"generated_at"' /tmp/deecure-test.json || fail "status --json did not print the report"
echo "ok: agent ran, loaded the rules, and kept its database private"

if [ -n "$app" ]; then
  echo "== desktop app"
  apt-get install -y -qq "$app" >/dev/null
  dpkg-query -W -f='${Package} ${Version} ${Status}\n' deecure-desktop
  id=io.github.krisskad0.DeeCure
  for f in /usr/bin/deecure-desktop /usr/share/applications/$id.desktop /usr/share/metainfo/$id.metainfo.xml /usr/share/icons/hicolor/scalable/apps/$id.svg; do
    [ -e "$f" ] || fail "missing $f"
  done
  # Every library the app needs came with its dependencies.
  missing="$(ldd /usr/bin/deecure-desktop | grep "not found" || true)"
  [ -z "$missing" ] || fail "missing libraries: $missing"
  grep -q "^Exec=deecure-desktop$" /usr/share/applications/$id.desktop || fail "the menu entry does not start the app"
  echo "ok: app, menu entry, icon, and AppStream data installed; all libraries found"
fi

echo "== purge"
[ -z "$app" ] || apt-get purge -y -qq deecure-desktop >/dev/null
apt-get purge -y -qq deecure >/dev/null
left="$(ls -d /usr/bin/deecure /usr/bin/deecure-desktop /usr/share/polkit-1/actions/io.github.krisskad0.deecure.policy /usr/share/dbus-1/system.d/io.github.krisskad0.DeeCure.Helper.conf /usr/libexec/deecure /usr/share/deecure /usr/share/applications/io.github.krisskad0.DeeCure.desktop /usr/lib/systemd/user/deecure-agent.* /etc/systemd/user/*/deecure-agent.* /usr/lib/systemd/system/deecure-power.service /etc/systemd/system/*/deecure-power.service /etc/deecure /var/log/deecure 2>/dev/null || true)"
[ -z "$left" ] || fail "left behind: $left"
echo "ok: nothing left behind"
