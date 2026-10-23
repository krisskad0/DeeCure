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
  sleep 2
  deecure --socket /tmp/deecure-test.sock status
  deecure --socket /tmp/deecure-test.sock status --json > /tmp/deecure-test.json
  kill $agent || true
  # Every installed rule must load: the count in the log matches the files shipped.
  rules=$(find /usr/share/deecure/rules -name "*.yaml" | wc -l)
  grep -q "$rules rules from /usr/share/deecure/rules" /tmp/deecure-test.log
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
left="$(ls -d /usr/bin/deecure /usr/bin/deecure-desktop /usr/libexec/deecure /usr/share/deecure /usr/share/applications/io.github.krisskad0.DeeCure.desktop /usr/lib/systemd/user/deecure-agent.* /etc/systemd/user/*/deecure-agent.* 2>/dev/null || true)"
[ -z "$left" ] || fail "left behind: $left"
echo "ok: nothing left behind"
