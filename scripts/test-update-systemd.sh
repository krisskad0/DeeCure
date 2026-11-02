#!/usr/bin/env bash
# The real update paths, with systemd running as it does on a desktop (the other container
# tests have no systemd, so maintainer scripts that stop services are skipped there).
# Run inside a booted systemd container (make update-systemd-test starts one):
#
#   scripts/test-update-systemd.sh --throwaway <broken old> <old> <new>
#   such as: --throwaway 0.1.0-alpha.6 0.1.0-alpha.7 0.1.0-alpha.8
#
# 1. The bug is caught: <broken old>'s in-app update to <old> stops half way (its package
#    stopped the service installing it). This proves the test sees the problem.
# 2. Repair: the one-command install.sh from main finishes on the newest release.
# 3. The in-app update (D-Bus Run install_update, as the Install button) from <old> to <new>.
# 4. The automatic update (deecure-update.service through systemd) from <old> to <new>.
# 5. Through the admin helper, upgrade a package that writes /lib/modules and installs a
#    setuid program, as a kernel or sudo update does.
set -euo pipefail

if [ "${1:-}" != "--throwaway" ] || [ -z "${4:-}" ]; then
  echo "Usage: $0 --throwaway <broken old> <old> <new>   (only in a throwaway systemd container)" >&2
  exit 2
fi
broken="$2"
old="$3"
new="$4"
[ "$(id -u)" = 0 ] || { echo "$0: must run as root" >&2; exit 2; }
[ -d /run/systemd/system ] || { echo "$0: systemd is not running" >&2; exit 2; }
export DEBIAN_FRONTEND=noninteractive
fail() { echo "FAIL: $*" >&2; exit 1; }
deb_version() { echo "${1/-/\~}-1"; }

# Downloads a published release and checks it against its SHA256SUMS.
fetch() { # version, folder
  local v="$1" dir="$2" name="${1/-/.}" base="https://github.com/krisskad0/DeeCure/releases/download/v$1"
  mkdir -p "$dir"
  curl -fsSL -o "$dir/SHA256SUMS" "$base/SHA256SUMS" || fail "could not download SHA256SUMS of $v"
  for p in deecure deecure-desktop; do
    curl -fsSL -o "$dir/$p.deb" "$base/${p}_${name}-1_amd64.deb" || fail "could not download $p $v"
    sum="$(grep " ${p}_${name}-1_amd64.deb\$" "$dir/SHA256SUMS" | cut -d' ' -f1)"
    echo "$sum  $dir/$p.deb" | sha256sum -c --quiet - || fail "$p $v does not match its checksum"
  done
}

# Puts a published version in place, downgrading when needed.
install_version() { # version
  fetch "$1" "/tmp/rel-$1"
  apt-get install -y -qq --no-install-recommends --allow-downgrades "/tmp/rel-$1/deecure.deb" "/tmp/rel-$1/deecure-desktop.deb" >/dev/null
  [ "$(dpkg-query -W -f='${db:Status-Abbrev}${Version}' deecure)" = "ii $(deb_version "$1")" ] || fail "could not install $1"
}

# Both packages fully installed at this version, and dpkg has nothing half done.
check_installed() { # version, what
  local want
  want="$(deb_version "$1")"
  for p in deecure deecure-desktop; do
    got="$(dpkg-query -W -f='${db:Status-Abbrev}${Version}' "$p")"
    [ "$got" = "ii $want" ] || fail "$2: $p is '$got', not 'ii $want'"
  done
  [ -z "$(dpkg --audit)" ] || fail "$2: dpkg reports half-done packages: $(dpkg --audit)"
  echo "ok: $2: both packages fully installed at $1, nothing half done"
}

call() { # json; the reply or the error
  su deecure-test -s /bin/sh -c "dbus-send --system --print-reply --reply-timeout=600000 --dest=io.github.krisskad0.DeeCure.Helper /io/github/krisskad0/DeeCure/Helper io.github.krisskad0.DeeCure.Helper1.Run string:'$1'" 2>&1
}

echo "== set up: a desktop user polkit trusts, and the tools"
# Container images block service starts and stops in package scripts (policy-rc.d exits 101).
# A desktop has no such file, and that is exactly what broke the alpha.6 and alpha.7 updates.
rm -f /usr/sbin/policy-rc.d
apt-get update -qq >/dev/null
for tool in curl dbus-send pkcheck; do command -v "$tool" >/dev/null || fail "$tool is missing (the container image installs it)"; done
id deecure-test >/dev/null 2>&1 || useradd -m deecure-test
mkdir -p /etc/polkit-1/rules.d
printf '%s\n' 'polkit.addRule(function (action, subject) {' \
  '  if (action.id.indexOf("io.github.krisskad0.deecure.") === 0 && subject.user === "deecure-test") {' \
  '    return polkit.Result.YES;' '  }' '});' >/etc/polkit-1/rules.d/50-deecure-test.rules
# polkit reads new rule files by itself; give it a moment.
systemctl start polkit.service
sleep 2
echo "ok"

echo "== 1. the bug is caught: $broken's in-app update to $old"
install_version "$broken"
out="$(call "{\"op\":\"install_update\",\"version\":\"$old\"}" || true)"
state="$(dpkg-query -W -f='${db:Status-Abbrev}' deecure) $(dpkg-query -W -f='${db:Status-Abbrev}' deecure-desktop)"
if [ "$state" = "ii  ii " ] && [ "$(dpkg-query -W -f='${Version}' deecure)" = "$(deb_version "$old")" ]; then
  fail "$broken to $old finished, but it broke on a real desktop; this test does not see the problem"
fi
echo "ok: reproduced: '$(echo "$out" | tail -1)', package states: $state"

echo "== 2. repair with the one-command install"
# The repo's install.sh: the same file GitHub serves from main once it is pushed.
sh "$(dirname "$0")/../packaging/install.sh" >/tmp/install.log 2>&1 || fail "install.sh failed: $(tail -25 /tmp/install.log)"
check_installed "$new" "repair"

echo "== 3. the in-app update (the Install button): $old to $new"
install_version "$old"
out="$(call "{\"op\":\"install_update\",\"version\":\"$new\"}")" || fail "the helper did not answer: $out"
echo "$out" | grep -q "is installed" || fail "unexpected reply: $out"
check_installed "$new" "in-app update"
systemctl is-failed --quiet deecure-helper.service && fail "deecure-helper.service failed"
echo "ok: the helper answered: $(echo "$out" | grep -o 'DeeCure [^"]*')"

echo "== 4. the automatic update through systemd: $old to $new"
install_version "$old"
mkdir -p /etc/deecure
printf '{"auto": true, "check": "daily", "channel": "prerelease"}' >/etc/deecure/update.json
chmod 0644 /etc/deecure/update.json
systemctl start deecure-update.service || fail "deecure-update.service failed: $(journalctl -u deecure-update.service --no-pager | tail -5)"
check_installed "$new" "automatic update"
rm -f /etc/deecure/update.json

echo "== 5. a kernel-like and a sudo-like upgrade through the admin helper"
# apt downloads as the _apt user, so the folder must be readable by everyone.
work=/srv/deecure-sandbox-test
mkdir -p "$work"
chmod 0755 "$work"
for v in 1 2; do
  d="$work/pkg$v"
  mkdir -p "$d/DEBIAN" "$d/usr/lib/modules/0.0.0-deecure-test" "$d/usr/bin"
  printf 'Package: deecure-sandbox-test\nVersion: %s\nArchitecture: all\nMaintainer: test <test@example.com>\nDescription: test package\n' "$v" >"$d/DEBIAN/control"
  echo "v$v" >"$d/usr/lib/modules/0.0.0-deecure-test/modules.test"
  cp /usr/bin/true "$d/usr/bin/deecure-setuid-test"
  chmod 4755 "$d/usr/bin/deecure-setuid-test"
  dpkg-deb --root-owner-group -b "$d" "$work/deecure-sandbox-test_${v}_all.deb" >/dev/null
done
dpkg -i "$work/deecure-sandbox-test_1_all.deb" >/dev/null
mkdir -p "$work/repo"
chmod 0755 "$work/repo"
cp "$work/deecure-sandbox-test_2_all.deb" "$work/repo/"
# The repository index by hand (dpkg-scanpackages is not on a desktop).
f="$work/repo/deecure-sandbox-test_2_all.deb"
{ dpkg-deb -f "$f"; echo "Filename: ./$(basename "$f")"; echo "Size: $(stat -c %s "$f")"; echo "SHA256: $(sha256sum "$f" | cut -d' ' -f1)"; echo; } >"$work/repo/Packages"
echo "deb [trusted=yes] file:$work/repo ./" >/etc/apt/sources.list.d/deecure-test.list
apt-get update -qq >/dev/null
out="$(call '{"op":"upgrade_packages","packages":["deecure-sandbox-test"]}')" || fail "the upgrade through the helper failed: $out"
[ "$(dpkg-query -W -f='${db:Status-Abbrev}${Version}' deecure-sandbox-test)" = "ii 2" ] || fail "the test package is not at version 2: $out"
grep -q v2 /usr/lib/modules/0.0.0-deecure-test/modules.test || fail "/lib/modules was not written"
[ -u /usr/bin/deecure-setuid-test ] || fail "the setuid program lost its setuid bit"
echo "ok: /lib/modules written and a setuid program installed through the helper"

echo "PASS: in-app and automatic updates and the helper's sandbox, with systemd"
