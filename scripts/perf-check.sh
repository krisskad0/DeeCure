#!/usr/bin/env bash
# Checks the agent against the performance budget and fails if it is over.
# Budget: docs/02-architecture/performance-budget.md
# Usage: scripts/perf-check.sh        (or: make perf)
set -euo pipefail

# The budget. Keep these in step with performance-budget.md.
MAX_IDLE_CPU_PERCENT=1.0
MAX_RAM_MB=60
MAX_FIRST_RESULT_SECONDS=10
MAX_DB_MB=200
IDLE_SECONDS=65   # long enough to include one full collection cycle (every 60 s)
REQUESTS=300

repo="$(cd "$(dirname "$0")/.." && pwd)"
cargo build --quiet --release --locked -p deecure-agent -p deecure-cli
agent="$repo/target/release/deecure-agent"
deecure="$repo/target/release/deecure"

tmp="$(mktemp -d)"
pid=""
cleanup() { [ -n "$pid" ] && kill "$pid" 2>/dev/null; rm -rf "$tmp"; }
trap cleanup EXIT
socket="$tmp/agent.sock"
ask() { "$deecure" --socket "$socket" status --json 2>/dev/null; }

start_ns=$(date +%s%N)
"$agent" --socket "$socket" --db "$tmp/deecure.db" --rules "$repo/rules" 2>"$tmp/agent.log" &
pid=$!

# First useful result: the rules have run once (checked_at is set).
until ask | grep -q '"checked_at"'; do
  kill -0 "$pid" 2>/dev/null || { cat "$tmp/agent.log"; echo "FAIL: the agent stopped" >&2; exit 1; }
  (( ($(date +%s%N) - start_ns) / 1000000000 < 30 )) || { echo "FAIL: no result after 30 s" >&2; exit 1; }
  sleep 0.1
done
first_ms=$(( ($(date +%s%N) - start_ns) / 1000000 ))

# CPU while idle: CPU ticks used over the window, as a share of one core.
ticks() { awk '{print $14 + $15}' "/proc/$pid/stat"; }
t1=$(ticks); sleep "$IDLE_SECONDS"; t2=$(ticks)
idle_cpu=$(awk -v d=$((t2 - t1)) -v hz="$(getconf CLK_TCK)" -v s="$IDLE_SECONDS" 'BEGIN {printf "%.2f", d * 100 / (hz * s)}')

# RAM after a burst of requests: the peak resident size (VmHWM).
for _ in $(seq "$REQUESTS"); do ask >/dev/null; done
peak_mb=$(awk '/VmHWM/ {printf "%.1f", $2 / 1024}' "/proc/$pid/status")
db_mb=$(du -cm "$tmp"/deecure.db* | awk 'END {print $1}')

failed=0
row() { # name, value, unit, limit, ok
  local mark="ok"; [ "$5" = 1 ] || { mark="OVER BUDGET"; failed=1; }
  printf '| %-26s | %8s %-3s | %6s %-3s | %-11s |\n' "$1" "$2" "$3" "$4" "$3" "$mark"
}
le() { awk -v a="$1" -v b="$2" 'BEGIN {print (a <= b) ? 1 : 0}'; }

echo
echo "| Measure                    | Now          | Budget     | Result      |"
echo "| -------------------------- | ------------ | ---------- | ----------- |"
first_s=$(awk -v ms="$first_ms" 'BEGIN {printf "%.1f", ms / 1000}')
row "First useful result" "$first_s" "s" "$MAX_FIRST_RESULT_SECONDS" "$(le "$first_s" "$MAX_FIRST_RESULT_SECONDS")"
row "Agent CPU when idle" "$idle_cpu" "%" "$MAX_IDLE_CPU_PERCENT" "$(le "$idle_cpu" "$MAX_IDLE_CPU_PERCENT")"
row "Agent RAM (peak)" "$peak_mb" "MB" "$MAX_RAM_MB" "$(le "$peak_mb" "$MAX_RAM_MB")"
row "Database after first scan" "$db_mb" "MB" "$MAX_DB_MB" "$(le "$db_mb" "$MAX_DB_MB")"
echo
[ "$failed" = 0 ] && echo "Within budget." || { echo "Over budget. See docs/02-architecture/performance-budget.md" >&2; exit 1; }
