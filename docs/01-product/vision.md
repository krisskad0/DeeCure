# Vision, users and principles

[Docs](../README.md) › [Product](README.md)

## The idea in one line

DeeCure is **the control center for your Linux computer**: it understands every part of it in depth, keeps it healthy, warns you before something breaks, and fixes problems safely, with a preview and an undo button.

## The goal

One app that gives a person **full control of their own computer**, in five steps:

| Step | What DeeCure does | Where |
| --- | --- | --- |
| **Understand** | Shows everything: hardware down to each sensor's limit and each drive's link speed, the operating system, apps, services, storage, network, security, and what changed | Every page; the deep data in [Expert coverage](expert-coverage.md) |
| **Keep healthy** | Watches the whole machine every minute and keeps a history | The agent; [Health score](../02-architecture/rules-engine.md) |
| **Warn early** | Warns about harm (heat, stalls, errors, wear, risk), judged against each part's own limits, only when it lasts | [Health warnings](health-warnings.md) |
| **Fix** | Fixes problems and offers improvements that a measurement shows are needed, each with a preview and undo | [Optimize](optimize-catalog.md), the [fix engine](../02-architecture/fix-engine.md) |
| **Control** | Sets battery, power, sleep, and device settings directly, and keeps them after restart | Power › Power Profile and Power › Sleep |

Think of it this way:

- **Cockpit** is like a hospital control panel. It is powerful, but you need to be a nurse to use it.
- **DeeCure** is like a good family doctor who also hands you a full X-ray of your computer. It says "your disk is almost full, here is why, and here is the safe fix."

## Who it is for

| Person | Situation | What they need from DeeCure |
| --- | --- | --- |
| **Riya, student, new to Ubuntu** | Laptop feels slow, battery dies fast, scared of the terminal | One health score, simple answers, safe one-click fixes |
| **Arjun, developer** | Docker and old kernels filling the disk, many cron jobs, VPNs | Fast full picture, cleanup, scheduled-jobs calendar, Advanced and Expert levels |
| **Meena, small office admin** | Looks after 5 shared Ubuntu PCs, no IT team | Clear warnings, security checks, history of "what changed" |

Start with Riya and Arjun. Meena (many machines) comes in a later phase.

## Product principles

These rules decide every design question.

1. **Explain, don't just show.** Every item has a one-line meaning in plain English.
2. **Safe by default.** Every fix shows the exact command first, and keeps an undo point where possible. Risky settings are tried for one session before they are kept.
3. **Honest.** If something cannot be checked, DeeCure says "your hardware does not report this", not a fake value.
4. **Private.** Everything stays on the computer. Nothing goes to the internet unless the user turns it on.
5. **Light.** It should never be the reason the PC is slow. Deep data is read only while its page is open.
6. **Warn about harm, not use.** A busy CPU or full RAM cache is the computer doing its job. DeeCure warns about stalls, errors, heat, wear, and risk, against the hardware's own limits, and only when a problem lasts. The user can turn any warning off or change its limit.
7. **Measured, not assumed.** An improvement is offered only when a measurement on this computer shows it is needed; common tips that do not help are never offered.
8. **Light for beginners, deep for experts.** Beginner keeps the menus short; Expert shows every detail ([View levels](view-levels.md)).
