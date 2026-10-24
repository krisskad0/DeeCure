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

## What is built

```
agent (user service)        desktop app / CLI (in the session)        deecure-helper (root)
fix_plan ──► Plan  ───────► preview, untick steps
                             apply: user steps run here
                                    admin steps ─── D-Bus + polkit ──► check, run, audit
fix_record ◄── FixRun ◄───── results (with undo calls)
  saves it, re-checks the rules now
fix_runs / fix_undone ◄───── Activity › DeeCure Actions, Backups › Restore Points, `deecure undo`
```

- **Plan** (`crates/deecure-agent/src/fixes.rs`): built from live data, so sizes and items are current. 47 fixes (`deecure_core::fix::ACTIONS`): free up space (and each part of it), remove packages no longer needed or leftover settings, install security or all updates, turn on the firewall, remove old kernels, restart / turn off / turn on a service, turn a startup app off or on, end a process, change the power mode, power off cleanly at 5%, and the power controls: battery charge limits and charge mode, CPU turbo, the highest CPU speed on battery, the PCIe power saving policy and the suspend type (each tried for this session first, then kept), what closing the lid does, a device's wake-up and power saving, the keyboard backlight, a radio, and Wi-Fi power saving; and for Optimize: turbo on battery, the log size cap, snap versions kept, unused Flatpak runtimes, the file-watch limit, zswap, a hard disk scheduler, finishing a broken package install, installing a driver, rebuilding DKMS drivers, and firmware updates (fwupd, which asks for the password itself); and DeeCure's own update and automatic updates ([Updates](updates.md)).
- **Steps** name a fixed `Call`. Ten run as the user (Trash, thumbnails, Docker images and build cache, startup apps, ending one of your own processes, power mode, keyboard backlight through UPower, radios through rfkill, Wi-Fi power saving through NetworkManager); the rest go to the helper.
- **Why the app runs the steps, not the agent:** polkit shows its password box in the person's session. The agent is a systemd user service outside the session, so its calls could not be approved.
- **Protect:** steps that change settings return an undo call (the firewall, services, startup apps, the power mode, UPower.conf's old values, and every power control: the old value is read before it changes).
- **Verify:** after `fix_record`, the agent runs its collectors and rules at once; the drawer then says whether the finding is gone.
- **Undo** runs the undo calls, last step first, and marks the run undone.
- **CLI:** `deecure fix <action> [target]` (preview, then asks), `deecure fix --list`, `deecure history`, `deecure undo <n>`.

