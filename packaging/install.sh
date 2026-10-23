#!/bin/sh
# Install DeeCure from its signed APT repository. Read it before you run it:
#
#   curl -fsSL https://krisskad0.github.io/DeeCure/install.sh -o install.sh
#   less install.sh && sudo sh install.sh
#
# It does exactly the steps in docs/06-packaging/apt-repository.md, nothing more:
# it adds DeeCure's key and source, then installs the packages. It changes no other
# repositories or settings, and it is safe to run twice.
#
# Options:
#   --beta        use pre-releases (the beta component)
#   --server      install only the agent and the deecure command (no desktop app)
#   --dry-run     print the steps, change nothing
#   --uninstall   remove DeeCure, its key, and its source
set -eu

REPO="${DEECURE_REPO_URL:-https://krisskad0.github.io/DeeCure/apt}"
KEYRING=/etc/apt/keyrings/deecure.asc
SOURCE=/etc/apt/sources.list.d/deecure.sources
COMPONENT=main
PACKAGES="deecure deecure-desktop"
DRY=no
ACTION=install

for arg in "$@"; do
  case "$arg" in
    --beta) COMPONENT=beta ;;
    --server) PACKAGES=deecure ;;
    --dry-run) DRY=yes ;;
    --uninstall) ACTION=uninstall ;;
    -h|--help) sed -n '2,17p' "$0"; exit 0 ;;
    *) echo "install.sh: unknown option $arg (see --help)" >&2; exit 2 ;;
  esac
done

run() {
  echo "+ $*"
  [ "$DRY" = yes ] || "$@"
}

# Checks first.
[ "$DRY" = yes ] || [ "$(id -u)" = 0 ] || { echo "install.sh: run it with sudo" >&2; exit 1; }
[ -r /etc/os-release ] || { echo "install.sh: cannot tell which system this is" >&2; exit 1; }
. /etc/os-release
case "${ID:-}:${VERSION_CODENAME:-}" in
  ubuntu:noble|ubuntu:resolute) ;;
  *) echo "install.sh: ${PRETTY_NAME:-this system} is not supported yet (Ubuntu 24.04 and 26.04 are)" >&2; exit 1 ;;
esac
ARCH="$(dpkg --print-architecture)"
[ "$ARCH" = amd64 ] || { echo "install.sh: only amd64 is supported for now (this is $ARCH)" >&2; exit 1; }

if [ "$ACTION" = uninstall ]; then
  run apt-get remove -y deecure-desktop deecure
  run rm -f "$SOURCE" "$KEYRING"
  run apt-get update
  echo "DeeCure is removed. Your history stays in ~/.local/share/deecure until you delete it."
  exit 0
fi

command -v curl >/dev/null 2>&1 || run apt-get install -y curl ca-certificates

run install -d -m 0755 /etc/apt/keyrings
if [ "$DRY" = yes ]; then
  echo "+ curl -fsSL $REPO/deecure-archive-keyring.asc > $KEYRING"
else
  echo "+ curl -fsSL $REPO/deecure-archive-keyring.asc > $KEYRING"
  curl -fsSL "$REPO/deecure-archive-keyring.asc" > "$KEYRING.new"
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

run apt-get update
# shellcheck disable=SC2086 # the package list is meant to split
run apt-get install -y $PACKAGES
echo "DeeCure is installed. Open it from the app menu, or run: deecure status"
