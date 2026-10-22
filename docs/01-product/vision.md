# Vision, users and principles

[Docs](../README.md) › [Product](README.md)

## The idea in one line

SysDoc is a **family doctor for your Linux computer**. It checks everything, explains what it finds in plain English, and fixes problems safely with one click and an undo button.

Think of it this way:

- **Cockpit** is like a hospital control panel. It is powerful, but you need to be a nurse to use it.
- **SysDoc** is like a good family doctor. It says "your disk is almost full, here is why, and here is the safe fix."

## Who it is for

| Person | Situation | What they need from SysDoc |
| --- | --- | --- |
| **Riya, student, new to Ubuntu** | Laptop feels slow, battery dies fast, scared of the terminal | One health score, simple answers, safe one-click fixes |
| **Arjun, developer** | Docker and old kernels filling the disk, many cron jobs, VPNs | Fast full picture, cleanup, scheduled-jobs calendar, Expert mode |
| **Meena, small office admin** | Looks after 5 shared Ubuntu PCs, no IT team | Clear warnings, security checks, history of "what changed" |

Start with Riya and Arjun. Meena (many machines) comes in a later phase.

## Product principles

These five rules decide every design question.

1. **Explain, don't just show.** Every item has a one-line meaning in plain English.
2. **Safe by default.** Every fix shows the exact command first, and keeps an undo point where possible.
3. **Honest.** If something cannot be checked, SysDoc says "your hardware does not report this", not a fake value.
4. **Private.** Everything stays on the computer. Nothing goes to the internet unless the user turns it on.
5. **Light.** It should never be the reason the PC is slow.
