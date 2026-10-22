# 0005. Separate root helper on D-Bus, guarded by polkit

- Status: Accepted (from the implementation plan)
- Date: 2026-10-22

## Context

Some checks and all real fixes need root rights: reading disk health, reading the firewall, cleaning the APT cache, restarting a service. Running the whole app as root is unsafe. Asking for `sudo` in a terminal is unfriendly.

## Decision

- Put every root task in a small, separate program: `deecure-helper`.
- It offers a **fixed list of actions** on the D-Bus system bus. (D-Bus is the standard Linux message bus that programs use to talk to each other.) It never runs a free-form command.
- D-Bus starts it only when needed. It exits after 2 minutes of no use.
- Each action has its own polkit rule. (polkit is the Ubuntu service that shows the normal password box.) Read actions keep the approval for a few minutes. Risky write actions ask every time.
- Every call is checked and written to an audit log.

## Why

- Only a few hundred lines run as root, so they are easy to review.
- Users see the normal Ubuntu password box, not a terminal.
- **Options not picked:** running the agent as root (too much code with root rights), `pkexec` with shell scripts (hard to validate inputs), `sudo` rules (hard for beginners).

## Consequences

- Adding a new admin action means changing helper code, the polkit policy, and tests. That change needs two reviewers.
- Details: [helper.md](../helper.md) and [security-model.md](../security-model.md).
