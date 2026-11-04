#!/usr/bin/env bash
# Runs independent checks at the same time, one log each, and says which passed and how long
# each took. Fails when any check fails; a failed check's log is printed.
#
#   scripts/run-parallel.sh "name" "command" ["name" "command" ...]
#
# Logs stay in a folder under target/ (printed at the end) so a slow check can be read after.
set -uo pipefail
[ $# -ge 2 ] && [ $(($# % 2)) -eq 0 ] || { echo "Usage: $0 \"name\" \"command\" [\"name\" \"command\" ...]" >&2; exit 2; }
cd "$(dirname "$0")/.."
logs="target/parallel-$(date +%Y%m%d-%H%M%S)"
mkdir -p "$logs"

names=()
pids=()
starts=()
while [ $# -gt 0 ]; do
  name="$1" command="$2"
  shift 2
  bash -c "$command" >"$logs/$name.log" 2>&1 &
  names+=("$name")
  pids+=("$!")
  starts+=("$(date +%s)")
  echo "started: $name"
done

failed=0
for i in "${!pids[@]}"; do
  if wait "${pids[$i]}"; then result=ok; else result=FAIL; failed=1; fi
  # A check that ends early waits here for the ones before it, so its time is read from its log.
  seconds=$(($(stat -c %Y "$logs/${names[$i]}.log") - starts[i]))
  echo "$result: ${names[$i]} (about ${seconds} s)"
  [ "$result" = ok ] || tail -n 30 "$logs/${names[$i]}.log"
done
echo "logs: $logs"
exit "$failed"
