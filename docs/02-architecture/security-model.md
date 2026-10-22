# Security and privacy of DeeCure itself

[Docs](../README.md) › [Architecture](README.md)

1. **No free-form root commands.** The helper accepts only listed actions with checked inputs.
2. **No network by default.** The agent opens no ports. The UI talks over a local socket that only this user can access.
3. **Signed updates.** Packages and knowledge-base updates are signed.
4. **Audit log.** Every admin action is logged with time, user, and result.
5. **Telemetry off by default.** If added later, it is opt-in, anonymous, and documented.
6. **Code review rule.** Any change to the helper needs two reviewers.

## Trust boundaries

A trust boundary is a place where data moves from a less trusted part to a more trusted part. Each one needs a guard.

| Boundary | What could go wrong | Guard |
| --- | --- | --- |
| Other programs → agent socket | Another user reads this user's data | Socket in the private runtime folder, mode `0600`, caller user ID check |
| Agent → helper (D-Bus) | Any local program asks the helper to act | D-Bus policy limits the interface. polkit checks the real caller for each action. Strict input checks. Audit log. |
| Rules and KB files → fix engine | A bad rule starts a harmful fix | Rules can only name fix action IDs. Fix actions map to fixed helper calls. Rules ship inside the signed package. |
| Logs and AI output → user | Text in a log tricks the AI ("prompt injection") | AI tools are read-only. The AI can only suggest a fix ID. The user still sees the preview and approves. |
| Internet → package | Someone changes a package on the way | Signed APT repository (`Signed-By` keyring) and signed checksums on releases |
| Dependencies → build | A bad library update | Lockfiles, `cargo-deny` and `cargo-audit` in CI, `npm audit`, review of new dependencies |

## Helper review checklist

Use this list when reviewing any change to `deecure-helper`.

- [ ] No shell. Run programs with a fixed argument list, never `sh -c`.
- [ ] Every input is checked against a strict pattern or a live list (for example, the unit really exists).
- [ ] The action has its own polkit action ID with the right level (`auth_admin_keep` for reads, `auth_admin` for writes).
- [ ] The call is written to the audit log with time, user, inputs, and result.
- [ ] Errors return a clear message. They never leak file contents the caller could not read.
- [ ] The action has apply → verify → undo tests in a VM.
- [ ] Two reviewers approved the pull request.

## Reporting a security problem

See [SECURITY.md](../../SECURITY.md) at the repo root.
