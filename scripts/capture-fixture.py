#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Capture test data from this computer into tests/fixtures/<name>/.

Copies only what DeeCure's collectors read, and removes private data:
your user name and host name are replaced, disk UUIDs are blanked, and
battery serial numbers, models, and makers are not copied.

Usage: scripts/capture-fixture.py <name> [--description "..."] [--force]
Then:  cargo test -p deecure-agent --test fixtures
"""

import argparse
import datetime
import getpass
import json
import os
import re
import shutil
import socket
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
PROC_FILES = ["/proc/stat", "/proc/loadavg", "/proc/meminfo", "/proc/mounts"]
BATTERY_FILES = [
    "type", "scope", "present", "status", "capacity",
    "energy_now", "energy_full", "energy_full_design",
    "charge_now", "charge_full", "charge_full_design",
    "power_now", "current_now", "voltage_now",
]
# Must match LOCAL_TYPES in crates/deecure-agent/src/collect/filesystems.rs.
LOCAL_TYPES = {
    "ext2", "ext3", "ext4", "btrfs", "xfs", "f2fs", "vfat", "exfat", "ntfs", "ntfs3",
    "fuseblk", "zfs", "jfs", "reiserfs", "bcachefs",
}
UUID = re.compile(r"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}")


def scrub(text: str) -> str:
    """Replaces the user name, the host name, and UUIDs."""
    for secret, stand_in in ((getpass.getuser(), "user"), (socket.gethostname(), "host")):
        if len(secret) >= 3:
            text = text.replace(secret, stand_in)
    return UUID.sub("00000000-0000-0000-0000-000000000000", text)


def unescape(field: str) -> str:
    return re.sub(r"\\([0-7]{3})", lambda m: chr(int(m.group(1), 8)), field)


def local_mounts():
    """Mount points of local filesystems, first mount per device, like the collector."""
    seen, points = set(), []
    for line in Path("/proc/mounts").read_text().splitlines():
        parts = line.split()
        if len(parts) >= 3 and parts[2] in LOCAL_TYPES and parts[0] not in seen:
            seen.add(parts[0])
            points.append(unescape(parts[1]))
    return points


def disk_space():
    usage = {}
    for point in local_mounts():
        try:
            st = os.statvfs(point)
        except OSError:
            continue  # not readable for this user: the collector skips it too
        usage[scrub(point)] = {
            "size": st.f_blocks * st.f_frsize,
            "used": (st.f_blocks - st.f_bfree) * st.f_frsize,
            "free": st.f_bavail * st.f_frsize,
        }
    return usage


def services():
    reply = subprocess.run(
        ["busctl", "--system", "--json=short", "call", "org.freedesktop.systemd1",
         "/org/freedesktop/systemd1", "org.freedesktop.systemd1.Manager", "ListUnits"],
        check=True, capture_output=True, text=True,
    )
    units = json.loads(reply.stdout)["data"][0]
    return [
        {"name": u[0], "description": u[1], "load": u[2], "active": u[3], "sub": u[4]}
        for u in units
        if u[0].endswith(".service")
    ]


def os_name():
    for line in Path("/etc/os-release").read_text().splitlines():
        if line.startswith("PRETTY_NAME="):
            return line.split("=", 1)[1].strip('"')
    return "unknown"


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("name", help="folder name, such as dell-latitude-3450-ubuntu-24.04")
    parser.add_argument("--description", default="", help="one line about this machine")
    parser.add_argument("--force", action="store_true", help="replace an existing fixture")
    args = parser.parse_args()

    if not re.fullmatch(r"[a-z0-9][a-z0-9.-]*", args.name):
        sys.exit("name: use lowercase letters, digits, dots, and dashes")
    out = REPO / "tests" / "fixtures" / args.name
    if out.exists():
        if not args.force:
            sys.exit(f"{out} already exists; pass --force to replace it")
        shutil.rmtree(out)

    for path in PROC_FILES:
        target = out / path.lstrip("/")
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(scrub(Path(path).read_text()))

    supplies = Path("/sys/class/power_supply")
    for supply in sorted(supplies.iterdir()) if supplies.exists() else []:
        target = out / "sys/class/power_supply" / supply.name
        for name in BATTERY_FILES:
            source = supply / name
            if source.is_file():
                try:
                    text = source.read_text()
                except OSError:
                    continue
                target.mkdir(parents=True, exist_ok=True)
                (target / name).write_text(text)

    (out / "statvfs.json").write_text(json.dumps(disk_space(), indent=2) + "\n")
    (out / "services.json").write_text(scrub(json.dumps(services(), indent=2)) + "\n")

    dmi = Path("/sys/class/dmi/id")
    machine = " ".join(
        (dmi / f).read_text().strip() for f in ("sys_vendor", "product_name") if (dmi / f).is_file()
    )
    (out / "fixture.yaml").write_text(
        f"description: {json.dumps(args.description or 'Captured with scripts/capture-fixture.py')}\n"
        f"source: real\n"
        f"captured: {datetime.date.today().isoformat()}\n"
        f"machine: {json.dumps(machine or 'unknown')}\n"
        f"os: {json.dumps(os_name())}\n"
        f"kernel: {json.dumps(os.uname().release)}\n"
    )
    (out / "expected-findings.yaml").write_text(
        "# The exact findings DeeCure must report for this machine.\n"
        "# Run `cargo test -p deecure-agent --test fixtures`: it prints what DeeCure finds.\n"
        "# Check that it is right, then copy it here.\n"
        "[]\n"
    )
    print(f"Saved {out.relative_to(REPO)}")
    print("Before committing, read the files and remove anything private (disk labels, for example).")


if __name__ == "__main__":
    main()
