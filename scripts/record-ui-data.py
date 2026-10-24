#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Record the agent's real answers for every desktop bridge command, for the all-pages UI check.

Usage: scripts/record-ui-data.py SOCKET OUT.json KIND,KIND,...
Easier: make ui-real (it starts a throwaway agent, records, and runs the check).

The file holds this computer's details (host name, Wi-Fi name, serial numbers, processes), so
write it outside the repository and never commit it. Kinds that act instead of only reading
(the package file check, the diagnostics) are never called.
"""
import json
import socket
import sys
import time

SOCK, OUT, KINDS = sys.argv[1], sys.argv[2], sys.argv[3].split(",")
# Kinds that act instead of only reading: never called here.
SKIP = {"package_verify_run", "diagnose_performance", "diagnose_network"}


def ask(method, params=None):
    s = socket.socket(socket.AF_UNIX)
    s.settimeout(60)
    s.connect(SOCK)
    req = {"jsonrpc": "2.0", "id": 1, "method": method}
    if params is not None:
        req["params"] = params
    s.sendall((json.dumps(req) + "\n").encode())
    buf = b""
    while not buf.endswith(b"\n"):
        chunk = s.recv(1 << 20)
        if not chunk:
            break
        buf += chunk
    reply = json.loads(buf)
    return reply.get("result") if "result" in reply else {"__error": reply.get("error")}


now = int(time.time() * 1000)
data = {"inventory": {}, "snapshot": {}, "metrics": {}}
for kind in KINDS:
    if kind in SKIP:
        continue
    data["inventory"][kind] = ask("inventory", {"kind": kind})
for kind in ["filesystems", "services", "boot", "updates"]:
    data["snapshot"][kind] = ask("snapshot", {"kind": kind})
for name in ["cpu.usage", "temp.cpu", "battery.charge", "memory.used", "net.rx", "disk.read", "cpu.throttle_total", "health.score"]:
    data["metrics"][name] = ask("metrics", {"name": name, "since": now - 86_400_000, "until": now, "max_points": 160})
for m in ["status", "latest", "notify_settings", "data_settings", "weekly_reports", "fix_runs", "notifications"]:
    data[m] = ask(m)
data["events"] = ask("events", {"since": now - 7 * 86_400_000, "until": now, "limit": 500})
procs = data["inventory"].get("processes") or []
shell = next((p["pid"] for p in procs if p.get("name") == "gnome-shell"), procs[0]["pid"] if procs else 1)
data["process_detail"] = ask("process_detail", {"pid": shell})
data["package_detail"] = ask("package_detail", {"name": "coreutils"})
data["fix_plan"] = ask("fix_plan", {"action": "power.set_turbo", "subject": "off"})
json.dump(data, open(OUT, "w"))
errors = {k: v for k, v in data["inventory"].items() if isinstance(v, dict) and "__error" in v}
print("kinds:", len(data["inventory"]), "errors:", errors)
