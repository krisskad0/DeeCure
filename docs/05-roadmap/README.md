# Roadmap

[Docs](../README.md)

Durations are rough estimates for a team of 2–3 engineers.

| Phase | What ships | Rough time |
| --- | --- | --- |
| **0. Foundations** | Agent skeleton, SQLite, 5 collectors, CLI `status`, fixture tests | 3–4 weeks |
| **1. MVP — see and understand** | All Yes-status insights from the category list, home screen, category pages, KB for common items, health score, jobs calendar | 8–10 weeks |
| **2. Fix safely** | Helper + polkit, Admin-status insights, fix engine with preview, verify, and undo, first 20 fixes | 6–8 weeks |
| **3. History and trends** | What-changed timeline, trends, predictions ("disk full in N days"), notifications, weekly report | 5–6 weeks |
| **4. Ask SysDoc** | Local AI assistant with read-only tools, "Why did my PC restart?" | 4–6 weeks |
| **5. Reach** | Fedora and Arch adapters, KDE support, server web mode, multi-PC view for small offices | Ongoing |

**Launch the public beta after Phase 2.** Before that, the app can show problems but cannot fix them, and "safe fixes" is the main promise.

## Phase files

Build in this order. Each file lists the goal, the tasks in order, the specs to read, and a "Done when" check.

| Phase | File | Ends with |
| --- | --- | --- |
| 0 | [phase-0-foundations.md](phase-0-foundations.md) | `sysdoc status` works from an installed `.deb` |
| 1 | [phase-1-mvp.md](phase-1-mvp.md) | The desktop app shows every read-only insight |
| 2 | [phase-2-fix-safely.md](phase-2-fix-safely.md) | Safe fixes with undo, signed APT repo, **public beta** |
| 3 | [phase-3-history.md](phase-3-history.md) | "What changed" timeline, trends, notifications |
| 4 | [phase-4-assistant.md](phase-4-assistant.md) | Local AI answers questions with sources |
| 5 | [phase-5-reach.md](phase-5-reach.md) | More distros, KDE, servers, many PCs |

## How to work through a phase

1. Turn each task into a GitHub issue. Link the spec file in the issue.
2. Do the tasks in order. Later tasks need earlier ones.
3. If the spec is wrong or missing, fix the doc first, in the same pull request.
4. A phase is finished only when every "Done when" check passes.
