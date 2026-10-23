#!/usr/bin/env bash
# Creates or updates the labels in .github/labels.yml on GitHub. Run it as a maintainer
# with the gh command logged in (gh auth login). It never deletes labels.
#
#   scripts/sync-labels.sh              # for krisskad0/DeeCure
#   scripts/sync-labels.sh owner/repo   # for another copy
set -euo pipefail
repo="${1:-krisskad0/DeeCure}"
cd "$(dirname "$0")/.."
gh auth status >/dev/null 2>&1 || { echo "sync-labels: log in first with: gh auth login" >&2; exit 1; }

# One label per line: - {name: "...", color: "...", description: "..."}
sed -n 's/^- {name: "\(.*\)", color: "\(.*\)", description: "\(.*\)"}$/\1\t\2\t\3/p' .github/labels.yml |
  while IFS=$'\t' read -r name color description; do
    gh label create "$name" --repo "$repo" --color "$color" --description "$description" --force >/dev/null
    echo "ok: $name"
  done
