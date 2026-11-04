# DeeCure for system administrators

[Docs](../../README.md) › [User guide](../README.md) › [Who DeeCure is for](README.md)

**For:** Linux administrators, SREs, and anyone who runs Ubuntu machines, with or without a desktop.

## What usually goes wrong

- A service crashes in a loop, and nobody notices until users do.
- "What changed on this box since Tuesday?" takes an hour of `apt` logs, `journalctl`, and `diff`.
- A disk starts throwing I/O errors weeks before it dies.
- A port is open to the world, or SSH lets root sign in with a password.
- The same checks are written again as shell scripts on every machine.

## How DeeCure makes it easy

| You want to… | Use | What you get |
| --- | --- | --- |
| Health and problems in one line | `deecure status` | Problems, most serious first, in plain English. Add `--json` for scripts. |
| What changed, and when | `deecure history --since 2d` | Packages, config files, devices, system, services, and crashes, newest first. Filter with `--kind`. |
| A report for the ticket | `deecure report --since 7d --html report.html` | One HTML file: health, problems, what changed, disks, and a tip |
| Fix with a preview | `deecure fix --list`, then `deecure fix <name>` | The preview is printed first, and the fix runs after you say yes |
| Take a fix back | `deecure history --fixes`, then `deecure undo <number>` | The fix is undone where undo is possible |
| Look at services | **Services** | System and user services, startup apps, and how locked down each service is (exposure score) |
| Look at scheduled work | **Scheduled Jobs** | Cron, anacron, `at`, and systemd timers in one calendar, with failed jobs |
| Read logs fast | **Logs** | The journal, errors grouped with explanations, kernel messages, crashes, and every boot with how it ended |
| Check the boot | **System › Boot** | What made the last boot slow, step by step |

On a server, install without the desktop app:

```bash
curl -fsSL https://raw.githubusercontent.com/krisskad0/DeeCure/main/packaging/install.sh | sudo sh -s -- --server
```

## Warnings an admin wants

- **Service keeps restarting:** 10 or more restarts since boot.
- **A scheduled job failed.**
- **Disk read or write errors**, **disk became read-only**, or **drive wearing out**.
- **Kernel crash:** an oops, BUG, or panic in the last 7 days.
- **PCIe link failing**, or **CPU hardware errors** (machine checks).
- **Automatic security updates stopped working.**
- **SSH lets root sign in with a password**, and **the firewall is off** while something listens.

Every warning has a default you can see, and you can turn it off or change its limit in **Settings › Notifications** (Advanced and Expert views).

## Why it is easy to trust on a machine you manage

- **Read-only by default.** The agent runs as your normal user and only reads.
- **A small admin helper** with a fixed list of actions, input checks against the live system, no shell, and an audit log. It starts only when needed.
- **Light:** about 0.1% CPU and about 21 MB of memory when idle. Two years of history fit in under 200 MB.
- **Nothing is sent anywhere.** The only connection is an update check to GitHub, which you can turn off.
- **Checked releases:** each update is verified against the release checksums, with signed build provenance and a software bill of materials.

## Your first 10 minutes

1. Install it (with `--server` on a machine without a desktop).
2. Run `deecure status`. Deal with anything critical.
3. Run `deecure history --since 7d` to see what changed this week.
4. Add `deecure status --json` to your own monitoring if you like.

---

Previous: [Looking after other PCs](05-looking-after-other-pcs.md) · Next: [Security and privacy](07-security-and-privacy.md) · [All roles](README.md)
