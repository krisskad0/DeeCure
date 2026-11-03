#!/usr/bin/env bash
# Prints the release notes for one version: its section of CHANGELOG.md, then how to update
# and how to check a file. The release workflow puts this on the GitHub release page, and the
# app shows it before and after an update. Fails when CHANGELOG.md has no section for it, so
# no release goes out without notes.
#
#   scripts/release-notes.sh 0.1.0-alpha.8
set -euo pipefail

version="${1:?usage: $0 <version, such as 0.1.0-alpha.8>}"
repo="$(cd "$(dirname "$0")/.." && pwd)"
changelog="$repo/CHANGELOG.md"

section="$(awk -v v="$version" '
  index($0, "## [" v "]") == 1 { found = 1; next }
  found && /^## \[/ { exit }
  found { print }
' "$changelog")"
# Without blank lines and spaces, nothing is left when the section is empty.
if [ -z "$(printf '%s' "$section" | tr -d '[:space:]')" ]; then
  echo "$0: CHANGELOG.md has no notes for $version" >&2
  exit 1
fi

printf '%s\n' "$section" | sed -e '/./,$!d'
cat <<EOF

### How to update

DeeCure tells you when this version is out. Install it from **Settings › About › Updates**, or run the one-command install again:

\`\`\`
curl -fsSL https://raw.githubusercontent.com/krisskad0/DeeCure/main/packaging/install.sh | sudo sh
\`\`\`

Every file is checked against \`SHA256SUMS\` before it is installed. To check a file yourself: \`gh attestation verify <file> --repo krisskad0/DeeCure\`.
EOF
