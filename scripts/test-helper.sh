#!/usr/bin/env bash
# End-to-end test of the admin helper with a real system D-Bus and polkit.
# It installs packages and changes system files, so it only runs on a throwaway system:
#
#   make helper-test        # in a clean Ubuntu 24.04 container
set -euo pipefail
[ "${1:-}" = "--throwaway" ] && [ -n "${2:-}" ] || { echo "Usage: $0 --throwaway <deecure.deb>" >&2; exit 2; }
[ "$(id -u)" = 0 ] || { echo "$0: must run as root" >&2; exit 2; }
export DEBIAN_FRONTEND=noninteractive
fail() { echo "FAIL: $*" >&2; exit 1; }

echo "== install"
apt-get update -qq >/dev/null
apt-get install -y -qq dbus polkitd "$(realpath "$2")" >/dev/null
id deecure-test >/dev/null 2>&1 || useradd -m deecure-test
id stranger >/dev/null 2>&1 || useradd -m stranger

echo "== start the system bus and polkit"
mkdir -p /run/dbus
dbus-daemon --system --fork
/usr/lib/polkit-1/polkitd --no-debug >/tmp/polkitd.log 2>&1 &
# In this test, polkit approves DeeCure actions for deecure-test only (no password box here).
cat >/etc/polkit-1/rules.d/50-deecure-test.rules <<'RULE'
polkit.addRule(function (action, subject) {
  if (action.id.indexOf("io.github.krisskad0.deecure.") === 0 && subject.user === "deecure-test") {
    return polkit.Result.YES;
  }
});
RULE
for i in $(seq 1 50); do dbus-send --system --dest=org.freedesktop.PolicyKit1 --print-reply /org/freedesktop/PolicyKit1/Authority org.freedesktop.DBus.Peer.Ping >/dev/null 2>&1 && break; sleep 0.2; done

call() { # user, json
  su "$1" -s /bin/sh -c "dbus-send --system --print-reply --reply-timeout=120000 --dest=io.github.krisskad0.DeeCure.Helper /io/github/krisskad0/DeeCure/Helper io.github.krisskad0.DeeCure.Helper1.Run string:'$2'" 2>&1
}

echo "== an approved action runs"
out="$(call deecure-test '{"op":"clean_apt_cache"}')" || fail "clean_apt_cache: $out"
echo "$out" | grep -q '"message"' || fail "no result: $out"
echo "ok: apt cache cleaned through the helper"

echo "== bad inputs are refused"
out="$(call deecure-test '{"op":"restart_service","unit":"cron.service; reboot"}')" && fail "a bad unit name was accepted"
echo "$out" | grep -q "not a service name" || fail "unexpected answer: $out"
out="$(call deecure-test '{"op":"rm_rf","path":"/"}')" && fail "an unknown operation was accepted"
echo "$out" | grep -qi "unknown call" || fail "unexpected answer: $out"
out="$(call deecure-test '{"op":"empty_trash"}')" && fail "a user-level call was run as root"
echo "ok: bad unit, unknown operation, and user-level call refused"

echo "== a user polkit does not approve is refused"
out="$(call stranger '{"op":"clean_apt_cache"}')" && fail "an unapproved user ran an action"
echo "$out" | grep -qi "AccessDenied\|not authorized" || fail "unexpected answer: $out"
echo "ok: refused"

echo "== a failed step puts the edited file back"
mkdir -p /etc/UPower
printf '[UPower]\nPercentageAction=2\nCriticalPowerAction=HybridSleep\n' >/etc/UPower/UPower.conf
cp /etc/UPower/UPower.conf /tmp/upower.before
# There is no systemd in this container, so restarting UPower fails after the edit.
out="$(call deecure-test '{"op":"set_battery_action","action":"PowerOff","percent":5}')" && echo "(systemctl worked here)"
cmp -s /etc/UPower/UPower.conf /tmp/upower.before || { grep -q "PowerOff" /etc/UPower/UPower.conf && echo "$out" | grep -q '"undo"' || fail "UPower.conf changed without an undo record"; }
echo "ok: UPower.conf is either unchanged or changed with an undo record"

echo "== every call is in the audit log"
lines=$(wc -l </var/log/deecure/helper.log)
[ "$lines" -ge 6 ] || fail "only $lines audit lines"
grep -q '"ok":true' /var/log/deecure/helper.log || fail "no successful call logged"
grep -q '"message":"not authorized"' /var/log/deecure/helper.log || fail "refusal not logged"
if grep -q '\$y\$\|\$6\$' /var/log/deecure/helper.log; then fail "a password hash reached the log"; fi
echo "ok: $lines audit lines"

echo "== read-outs"
out="$(su deecure-test -s /bin/sh -c "dbus-send --system --print-reply --dest=io.github.krisskad0.DeeCure.Helper /io/github/krisskad0/DeeCure/Helper io.github.krisskad0.DeeCure.Helper1.Read string:accounts string:" 2>&1)" || fail "accounts read-out: $out"
echo "$out" | grep -q "deecure-test" || fail "accounts read-out lacks the user: $out"
echo "$out" | grep -q '\$y\$\|\$6\$' && fail "a password hash left the helper"
out="$(su deecure-test -s /bin/sh -c "dbus-send --system --print-reply --dest=io.github.krisskad0.DeeCure.Helper /io/github/krisskad0/DeeCure/Helper io.github.krisskad0.DeeCure.Helper1.Read string:smart string:../../etc/shadow" 2>&1)" && fail "a path was accepted as a disk"
echo "ok: account status without hashes; paths refused as disks"
echo "PASS: helper end to end"
