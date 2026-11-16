# Test DeeCure and report problems

[Docs](../README.md) › [User guide](README.md)

You can help DeeCure a lot without writing any code: use it on your computer, and tell us when something is wrong. Every report makes DeeCure better for everyone with a computer like yours.

## Who can help

- Anyone with Ubuntu 24.04 or 26.04, on a desktop, a laptop, a server, or a virtual machine (VM).
- No Linux expert knowledge is needed. "I did not understand this screen" is a useful report too.

## Is it safe to test?

- DeeCure only reads your system, until you click a fix and approve it.
- Every fix shows what it will do before it runs, and most can be undone.
- To test fixes, use a VM or a spare computer, not the computer you work on.
- You can remove DeeCure at any time (see [Remove DeeCure](#remove-deecure)).

## Step 1: install

1. Install with the one command from the [README](../../README.md#install-in-one-step). On a server without a desktop, add `--server` ([options](install.md)).
2. Check the version: `deecure --version`. You will need it in your report.

## Step 2: pick something to test

- **A ready test:** open issues with the label [`testing`](https://github.com/krisskad0/DeeCure/issues?q=is%3Aopen+label%3Atesting). Each one says what to check, step by step. Comment on the issue to say you are on it.
- **Free testing:** use DeeCure for a few days as you normally would. Open every page you understand, and look for the problems in the next step.

## Step 3: what counts as a problem

| You see | Example |
| --- | --- |
| A wrong number | DeeCure says 20 GB free, but `df -h` says 50 GB |
| Something empty or missing | A panel stays empty, or your second disk is not listed |
| An error | A red message, "agent unreachable", or the window closes |
| A wrong warning | A warning about something that is fine, or no warning when something is really wrong |
| Slow or heavy | A page takes long to open, or DeeCure uses a lot of CPU or memory |
| Confusing text | You read it twice and still do not know what it means or what to do |

When a number looks wrong, compare it with another tool, such as `df -h` (disks), `free -h` (memory), `lsblk` (drives), `ss -tulpn` (open ports), or `sensors` (temperatures). Put both numbers in your report.

## Step 4: report it

1. Search the [open issues](https://github.com/krisskad0/DeeCure/issues) first. If someone reported the same problem, add a comment with your details instead.
2. Open a [bug report](https://github.com/krisskad0/DeeCure/issues/new?template=bug_report.yml). One problem per report.
3. Fill in:
   - what you did, what you expected, and what happened instead;
   - the steps, so we can do the same;
   - your DeeCure version (`deecure --version`) and Ubuntu version (`lsb_release -ds`), and your desktop (GNOME, KDE, and so on);
   - a screenshot, if you can see the problem;
   - the agent log: `journalctl --user -u deecure-agent -n 100`.
4. For how DeeCure works on your hardware in general, use the [hardware report](https://github.com/krisskad0/DeeCure/issues/new?template=hardware_report.yml) instead.

## Keep private things private

Reports are public. Before you post a log, a screenshot, or command output:

- Remove your user name, computer name, IP addresses, Wi-Fi names, and serial numbers.
- Do not post the full `deecure status --json` without reading it first: it describes your whole computer.
- Report security problems privately, never in a public issue: [SECURITY.md](../../SECURITY.md).

## After you report

1. A maintainer reads it and adds labels. A new issue starts as `status: needs-triage`.
2. If we need more facts, the issue gets `status: needs-info`. Please answer: an issue without an answer for 30 days is closed.
3. When the problem is clear, it becomes `status: ready`, and someone fixes it.
4. The fix is listed in the [changelog](../../CHANGELOG.md) of the next release, and DeeCure updates itself.

## Remove DeeCure

```bash
sudo sh install.sh --uninstall
```

Other ways: [Install › Uninstall](install.md#uninstall).
