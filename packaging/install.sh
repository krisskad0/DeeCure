#!/bin/sh
# Install DeeCure with one command. Read it before you run it:
#
#   curl -fsSL https://raw.githubusercontent.com/krisskad0/DeeCure/main/packaging/install.sh -o install.sh
#   less install.sh && sudo sh install.sh
#
# Where it installs from:
#   - DeeCure's signed APT repository, when it is online: updates then come through
#     Software Updater like any other app (docs/06-packaging/apt-repository.md).
#   - Otherwise the newest GitHub Release: the two .deb files are downloaded, checked
#     against the release's SHA256SUMS, and installed with apt, which also installs
#     the libraries they need from Ubuntu. Run this script again to update.
# It changes no other repositories or settings, and it is safe to run twice.
#
# Options:
#   --beta        use pre-releases
#   --server      install only the agent and the deecure command (no desktop app)
#   --from-github always install from GitHub Releases, even when the APT repository is online
#   --dry-run     check and download, but change nothing
#   --uninstall   remove DeeCure, its key, and its source
set -eu

REPO="${DEECURE_REPO_URL:-https://krisskad0.github.io/DeeCure/apt}"
RELEASES="${DEECURE_RELEASES_URL:-https://api.github.com/repos/krisskad0/DeeCure/releases}"
KEYRING=/etc/apt/keyrings/deecure.asc
SOURCE=/etc/apt/sources.list.d/deecure.sources
COMPONENT=main
PACKAGES="deecure deecure-desktop"
DRY=no
ACTION=install
FROM=auto
BETA=no

for arg in "$@"; do
  case "$arg" in
    --beta) COMPONENT=beta; BETA=yes ;;
    --server) PACKAGES=deecure ;;
    --from-github) FROM=github ;;
    --dry-run) DRY=yes ;;
    --uninstall) ACTION=uninstall ;;
    -h|--help) sed -n '2,20p' "$0" 2>/dev/null || echo "See the top of install.sh for options."; exit 0 ;;
    *) echo "install.sh: unknown option $arg (see --help)" >&2; exit 2 ;;
  esac
done

run() {
  echo "+ $*"
  [ "$DRY" = yes ] || "$@"
}

fail() {
  echo "install.sh: $*" >&2
  exit 1
}

# Checks first.
[ "$DRY" = yes ] || [ "$(id -u)" = 0 ] || fail "run it with sudo"
[ -r /etc/os-release ] || fail "cannot tell which system this is"
. /etc/os-release
case "${ID:-}:${VERSION_CODENAME:-}" in
  ubuntu:noble|ubuntu:resolute) ;;
  *) fail "${PRETTY_NAME:-this system} is not supported yet (Ubuntu 24.04 and 26.04 are)" ;;
esac
ARCH="$(dpkg --print-architecture)"
[ "$ARCH" = amd64 ] || fail "only amd64 is supported for now (this is $ARCH)"

if [ "$ACTION" = uninstall ]; then
  run apt-get remove -y deecure-desktop deecure
  run rm -f "$SOURCE" "$KEYRING"
  run apt-get update
  echo "DeeCure is removed. Your history stays in ~/.local/share/deecure until you delete it."
  exit 0
fi

# A downloader: curl, or wget (Ubuntu Desktop has wget but not always curl).
if command -v curl >/dev/null 2>&1; then
  fetch() { curl -fsSL --retry 3 "$1"; }
  reachable() { curl -fsIL --retry 2 "$1" >/dev/null 2>&1; }
elif command -v wget >/dev/null 2>&1; then
  fetch() { wget -qO- --tries=3 "$1"; }
  reachable() { wget -q --spider --tries=2 "$1"; }
else
  run apt-get install -y --no-install-recommends curl ca-certificates
  fetch() { curl -fsSL --retry 3 "$1"; }
  reachable() { curl -fsIL --retry 2 "$1" >/dev/null 2>&1; }
fi

APT="apt-get -o Acquire::Retries=3"

# The one optional extra DeeCure itself uses (the tray icon), when this Ubuntu has it.
# Everything else "recommended" by the libraries is left out: it is not needed and it
# makes a first install much bigger (about 30 MB and 57 packages on a new system).
extras() {
  case "$PACKAGES" in *deecure-desktop*) ;; *) return 0 ;; esac
  if apt-cache show libayatana-appindicator3-1 >/dev/null 2>&1; then echo libayatana-appindicator3-1; fi
}

# Installs packages without the optional extras. If it fails because the package lists
# are old, it refreshes them and tries once more.
install_packages() {
  echo "+ $APT install -y --no-install-recommends $*"
  [ "$DRY" = yes ] && return 0
  $APT install -y --no-install-recommends "$@" || {
    echo "Refreshing the package lists and trying again..."
    $APT update && $APT install -y --no-install-recommends "$@"
  }
}

# Package lists refreshed in the last 6 hours are fresh enough: skip the slow refresh.
refresh_lists() {
  if [ -n "$(find /var/lib/apt/lists -maxdepth 1 -name '*_Packages*' -mmin -360 2>/dev/null | head -1)" ]; then
    echo "Package lists are fresh; not refreshing them."
  else
    run $APT update
  fi
}

if [ "$FROM" = auto ]; then
  if reachable "$REPO/deecure-archive-keyring.asc"; then FROM=apt; else FROM=github; fi
fi

if [ "$FROM" = apt ]; then
  echo "Installing from DeeCure's APT repository: $REPO"
  run install -d -m 0755 /etc/apt/keyrings
  echo "+ fetch $REPO/deecure-archive-keyring.asc > $KEYRING"
  if [ "$DRY" = no ]; then
    fetch "$REPO/deecure-archive-keyring.asc" > "$KEYRING.new"
    mv "$KEYRING.new" "$KEYRING"
    chmod 0644 "$KEYRING"
  fi
  echo "+ write $SOURCE"
  if [ "$DRY" = no ]; then
    cat > "$SOURCE" <<SOURCES
Types: deb
URIs: $REPO
Suites: $VERSION_CODENAME
Components: $COMPONENT
Signed-By: $KEYRING
SOURCES
  fi
  # A new or changed source always needs a refresh, so the newest DeeCure is seen.
  run $APT update
  echo "Installing DeeCure and the libraries it needs from Ubuntu (1-3 minutes on a new system)..."
  # shellcheck disable=SC2086 # the package lists are meant to split
  install_packages $PACKAGES $(extras)
  [ "$DRY" = yes ] && { echo "Dry run: nothing was changed."; exit 0; }
  echo "DeeCure is installed. Open it from the app menu, or run: deecure status"
  exit 0
fi

# From GitHub Releases.
# python3 reads GitHub's release list. Every Ubuntu has it; a bare system may not, or
# may have only python3-minimal, which lacks the json module.
if ! python3 -c 'import json' >/dev/null 2>&1; then
  run apt-get install -y --no-install-recommends python3
fi
[ "$DRY" = yes ] || python3 -c 'import json' >/dev/null 2>&1 || fail "python3 (with its json module) is needed to read the release list"
echo "Installing from GitHub Releases: $RELEASES"
LIST="$(fetch "$RELEASES")" || fail "could not reach GitHub ($RELEASES); check the internet connection"

# The newest release that is not a draft; a stable one unless --beta. While there is
# no stable release yet, the newest pre-release is used, and that is said below.
CHOICE="$(printf '%s' "$LIST" | python3 -c '
import json, sys
beta = sys.argv[1] == "yes"
try:
    releases = [r for r in json.load(sys.stdin) if not r.get("draft")]
except Exception:
    sys.exit(3)
stable = [r for r in releases if not r.get("prerelease")]
pick = (releases if beta else stable or releases)[:1]
if not pick:
    sys.exit(4)
r = pick[0]
urls = {a["name"]: a["browser_download_url"] for a in r.get("assets", [])}
print(r["tag_name"], "pre-release" if r.get("prerelease") else "release")
for name, url in sorted(urls.items()):
    if name == "SHA256SUMS" or (name.endswith("_amd64.deb") and name.startswith(("deecure_", "deecure-desktop_"))):
        print(name, url)
' "$BETA")" || {
  case $? in
    4) fail "no published release was found yet; see https://github.com/krisskad0/DeeCure/releases" ;;
    *) fail "could not read GitHub's release list; try again in a minute, or see https://github.com/krisskad0/DeeCure/releases" ;;
  esac
}

TAG="$(printf '%s\n' "$CHOICE" | sed -n '1p' | cut -d' ' -f1)"
KIND="$(printf '%s\n' "$CHOICE" | sed -n '1p' | cut -d' ' -f2)"
if [ "$KIND" = pre-release ] && [ "$BETA" = no ]; then
  echo "Note: there is no stable release yet, so the newest pre-release ($TAG) is used."
fi
echo "Release: $TAG ($KIND)"

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
trap 'exit 1' INT TERM
# apt reads local packages as the _apt user, so it must be able to read them.
chmod 0755 "$TMP"

want="SHA256SUMS deecure_"
case "$PACKAGES" in *deecure-desktop*) want="$want deecure-desktop_" ;; esac
LOCAL=""
for prefix in $want; do
  line="$(printf '%s\n' "$CHOICE" | sed '1d' | grep "^$prefix" | head -1)" || true
  [ -n "$line" ] || fail "release $TAG has no ${prefix%_} file for amd64"
  name="${line%% *}"
  url="${line#* }"
  echo "+ download $name (from GitHub)"
  fetch "$url" > "$TMP/$name" || fail "could not download $name"
  chmod 0644 "$TMP/$name"
  [ "$name" = SHA256SUMS ] || LOCAL="$LOCAL ./$name"
done

# Every package must match the release's checksum, or nothing is installed.
echo "+ check the downloads against SHA256SUMS"
for file in $LOCAL; do
  name="${file#./}"
  # GitHub may turn "~" into "." in file names, so both spellings match.
  expected="$(awk -v n="$name" '{ f = $2; sub(/^\*/, "", f); gsub(/~/, ".", f); m = n; gsub(/~/, ".", m); if (f == m) print $1 }' "$TMP/SHA256SUMS" | head -1)"
  [ -n "$expected" ] || fail "$name is not listed in SHA256SUMS"
  actual="$(sha256sum "$TMP/$name" | cut -d' ' -f1)"
  [ "$expected" = "$actual" ] || fail "$name does not match its checksum; nothing was installed"
done
echo "  ok"

refresh_lists
echo "Installing DeeCure and the libraries it needs from Ubuntu (1-3 minutes on a new system)..."
# shellcheck disable=SC2086 # the file and package lists are meant to split
(cd "$TMP" && install_packages $LOCAL $(extras))
if [ "$DRY" = yes ]; then
  echo "Dry run: DeeCure $TAG was downloaded and checked; nothing was changed."
  exit 0
fi
echo "DeeCure $TAG is installed. Open it from the app menu, or run: deecure status"
echo "To update later, run this command again."
