# Admin helper (sysdoc-helper)

[Docs](../README.md) › [Architecture](README.md)

The helper is the only part that runs as root, so it is kept small and strict.

1. It is **started by D-Bus only when needed**, and exits after 2 minutes of no use.
2. It offers a **fixed list of actions**. It never runs a free-form command. Examples:
   - `ReadSmart(device)`, `ReadFirewall()`, `ListAllCrontabs()`, `ReadShadowStatus()`
   - `CleanAptCache()`, `VacuumJournal(size)`, `RemoveSnapRevision(name, rev)`
   - `RestartService(unit)`, `DisableService(unit)`, `EnableFirewall()`, `RemoveOldKernels(keep=2)`
3. Each action has a **polkit rule**. Read actions use `auth_admin_keep`, so one password covers a few minutes. Risky write actions ask every time.
4. The helper checks every input (for example, a service name must exist) and writes every call to an audit log.
