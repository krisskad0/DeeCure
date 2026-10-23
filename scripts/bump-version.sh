#!/usr/bin/env bash
# Set one version everywhere: Cargo.toml, app/package.json, tauri.conf.json, and the
# AppStream release list. Usage: scripts/bump-version.sh 0.2.0
set -euo pipefail
v="${1:?usage: $0 <version, such as 0.2.0>}"
[[ "$v" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-rc\.[0-9]+)?$ ]] || { echo "not a version: $v" >&2; exit 2; }
repo="$(cd "$(dirname "$0")/.." && pwd)"
cd "$repo"
sed -i "0,/^version = \".*\"/s//version = \"$v\"/" Cargo.toml
# In these two files the first "version" is the top-level one.
sed -i "0,/\"version\": \".*\"/s//\"version\": \"$v\"/" app/package.json app/src-tauri/tauri.conf.json
# The lockfile has many "version" fields; edit it as JSON so only the root ones change.
python3 - "$v" <<'PY'
import json, sys
v = sys.argv[1]
def edit(path, change):
    with open(path) as f:
        data = json.load(f)
    change(data)
    with open(path, "w") as f:
        f.write(json.dumps(data, indent=2, ensure_ascii=False) + "\n")
def lock(d):
    d["version"] = v
    d["packages"][""]["version"] = v
edit("app/package-lock.json", lock)
PY
today="$(date +%F)"
if ! grep -q "release version=\"$v\"" packaging/desktop/io.github.krisskad0.DeeCure.metainfo.xml; then
  sed -i "s|^\( *\)<releases>|&\n\1  <release version=\"$v\" date=\"$today\"/>|" packaging/desktop/io.github.krisskad0.DeeCure.metainfo.xml
fi
cargo update --workspace --offline >/dev/null 2>&1 || true
echo "Version set to $v. Next:"
echo "  1. Move the Unreleased notes in CHANGELOG.md under $v"
echo "  2. dch -v ${v/-rc./~rc}-1 -D noble \"New upstream release.\"   (in packaging/)"
echo "  3. Commit, then: git tag -s v$v && git push --tags"
