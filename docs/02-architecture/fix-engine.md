# Fix engine and undo

[Docs](../README.md) › [Architecture](README.md)

Every fix follows the same five steps:

1. **Plan:** list the steps, commands, admin needs, and undo type.
2. **Preview:** the user sees everything and can untick steps.
3. **Protect:** DeeCure saves an undo point first (see table).
4. **Apply:** it runs the steps (admin steps through the helper) and stops on the first failure.
5. **Verify:** it re-runs the rule. If the problem is still there, it says so honestly.

| Undo type | Meaning | Example |
| --- | --- | --- |
| **Full undo** | DeeCure can put it back exactly | Disable a service → re-enable; edit a config → restore the saved copy |
| **No undo needed** | The thing rebuilds itself | Clear apt cache, thumbnails |
| **No undo** | It is gone for good, and the user is warned clearly | Empty Trash, remove old snap versions |
| **System snapshot** | Used for big changes when Timeshift or Btrfs is present | Remove old kernels |
