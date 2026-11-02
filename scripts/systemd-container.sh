#!/usr/bin/env bash
# Runs a script inside a throwaway Ubuntu container with systemd booted (PID 1), as on a
# desktop, then removes the container. The repo is mounted read-only at /src.
#
#   scripts/systemd-container.sh <ubuntu image> <script in the repo> [arguments...]
#   such as: scripts/systemd-container.sh ubuntu:24.04 scripts/test-update-systemd.sh --throwaway ...
set -euo pipefail

image="${1:?usage: $0 <ubuntu image> <script> [arguments]}"
script="${2:?usage: $0 <ubuntu image> <script> [arguments]}"
shift 2
repo="$(cd "$(dirname "$0")/.." && pwd)"
name="deecure-systemd-$$"
trap 'docker rm -f "$name" >/dev/null 2>&1 || true' EXIT

# systemd needs its own writable cgroup tree and /run. With a private cgroup namespace the
# container sees only its own branch, so making it writable (which needs CAP_SYS_ADMIN, and
# AppArmor's docker profile forbids the remount) never touches the host's cgroups.
docker run -d -t --name "$name" -e container=docker --cgroupns=private --cap-add SYS_ADMIN \
  --security-opt apparmor=unconfined --tmpfs /run --tmpfs /run/lock --tmpfs /tmp:exec \
  -v "$repo:/src:ro" "$image" bash -c \
  'mount -o remount,rw /sys/fs/cgroup && apt-get update -qq >/dev/null && DEBIAN_FRONTEND=noninteractive apt-get install -y -qq --no-install-recommends systemd systemd-sysv dbus polkitd curl ca-certificates python3 >/dev/null && exec /sbin/init' >/dev/null

echo "== boot systemd in $image"
for _ in $(seq 1 180); do
  state="$(docker exec "$name" systemctl is-system-running 2>/dev/null || true)"
  case "$state" in running | degraded) break ;; esac
  sleep 1
done
case "$state" in running | degraded) echo "ok: systemd is $state" ;; *) echo "FAIL: systemd did not start ($state)" >&2; docker logs "$name" | tail -20 >&2; exit 1 ;; esac

docker exec "$name" bash "/src/$script" "$@"
