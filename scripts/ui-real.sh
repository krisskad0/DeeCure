#!/bin/sh
# SPDX-License-Identifier: Apache-2.0
# Renders every page at the Expert level with this computer's real agent answers
# (app/src/allpages.test.tsx). It starts a throwaway agent with its own database and socket,
# records its answers to a private file outside the repository, and stops the agent again.
# It only reads: kinds that act are never called (scripts/record-ui-data.py).
set -eu
cd "$(dirname "$0")/.."
work="$(mktemp -d /tmp/deecure-ui-real.XXXXXX)"
trap 'kill -INT "$agent" 2>/dev/null || true; sleep 1; rm -rf "$work"' EXIT
cargo build -q -p deecure-agent
./target/debug/deecure-agent --rules rules --db "$work/x.db" --socket "$work/a.sock" >"$work/agent.log" 2>&1 &
agent=$!
i=0
until [ -S "$work/a.sock" ]; do
  i=$((i + 1))
  [ "$i" -gt 60 ] && { echo "the agent did not start"; cat "$work/agent.log"; exit 1; }
  sleep 0.5
done
echo "waiting 70 s for the first readings…"
sleep 70
kinds="$(sed -n '/pub const KINDS/,/^\];/p' crates/deecure-agent/src/inventory/mod.rs | grep -oE '"[a-z_]+"' | tr -d '"' | paste -sd, -)"
python3 -I scripts/record-ui-data.py "$work/a.sock" "$work/real.json" "$kinds"
cd app
DEECURE_UI_DATA="$work/real.json" npx vitest run src/allpages.test.tsx
