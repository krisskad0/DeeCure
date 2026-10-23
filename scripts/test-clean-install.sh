#!/usr/bin/env bash
# The real one-command install, exactly as a user runs it, on a clean system: it
# downloads install.sh from GitHub, which installs the newest published release.
# Then it checks the result, runs the command again (the update path), and uninstalls.
# It changes the system, so it only runs in a throwaway container:
#
#   make clean-install-test        # Ubuntu 24.04 with curl, and 26.04 with wget
set -euo pipefail
[ "${1:-}" = "--throwaway" ] || { echo "Usage: $0 --throwaway [curl|wget]   (only in a container)" >&2; exit 2; }
tool="${2:-curl}"
url="https://raw.githubusercontent.com/krisskad0/DeeCure/main/packaging/install.sh"
export DEBIAN_FRONTEND=noninteractive
fail() { echo "FAIL: $*" >&2; exit 1; }
. /etc/os-release
echo "== $PRETTY_NAME, downloading with $tool"

# A desktop has curl or wget; a bare container has neither.
apt-get update -qq >/dev/null
apt-get install -y -qq "$tool" ca-certificates >/dev/null
case "$tool" in
  curl) get() { curl -fsSL "$url"; } ;;
  wget) get() { wget -qO- "$url"; } ;;
  *) fail "unknown downloader $tool" ;;
esac

echo "== the one command"
get | sh 2>&1 | tail -n 12
dpkg-query -W -f='${Package} ${Version} ${Status}\n' deecure deecure-desktop
[ "$(dpkg-query -W -f='${Status}' deecure)" = "install ok installed" ] || fail "deecure is not installed"
[ "$(dpkg-query -W -f='${Status}' deecure-desktop)" = "install ok installed" ] || fail "deecure-desktop is not installed"

echo "== it works"
deecure --version
missing="$(ldd /usr/bin/deecure-desktop | grep "not found" || true)"
[ -z "$missing" ] || fail "the app is missing libraries: $missing"
id riya >/dev/null 2>&1 || useradd -m riya
su riya -c '
  set -e
  /usr/libexec/deecure/deecure-agent --socket /tmp/dc.sock 2>/tmp/dc.log &
  # It answers at once; the first check of the computer takes a few seconds more.
  for i in $(seq 1 45); do
    deecure --socket /tmp/dc.sock status >/tmp/dc-status.txt 2>&1 && grep -q "DeeCure status" /tmp/dc-status.txt && break
    sleep 1
  done
  head -n 4 /tmp/dc-status.txt
  grep -q "DeeCure status" /tmp/dc-status.txt
  kill $! 2>/dev/null || true
' || { cat /tmp/dc.log 2>/dev/null; fail "the agent did not answer"; }
echo "ok: installed, every library found, the agent answers"

echo "== the same command again (updating)"
get | sh >/tmp/again.txt 2>&1 || { cat /tmp/again.txt; fail "running it again failed"; }
tail -n 2 /tmp/again.txt

echo "== uninstall"
get | sh -s -- --uninstall >/tmp/remove.txt 2>&1 || { cat /tmp/remove.txt; fail "uninstall failed"; }
dpkg-query -W -f='${Status}\n' deecure 2>/dev/null | grep -q "install ok installed" && fail "deecure is still installed"
echo "ok: removed"
echo "PASS: one-command install on $PRETTY_NAME with $tool"
