# Admin helper (deecure-helper)

[Docs](../README.md) › [Architecture](README.md)

The helper is the only part that runs as root, so it is kept small and strict.

1. It is **started by D-Bus only when needed**, and exits after 2 minutes of no use.
2. It offers a **fixed list of actions**. It never runs a free-form command. Examples:
   - `ReadSmart(device)`, `ReadFirewall()`, `ListAllCrontabs()`, `ReadShadowStatus()`
   - `CleanAptCache()`, `VacuumJournal(size)`, `RemoveSnapRevision(name, rev)`
   - `RestartService(unit)`, `DisableService(unit)`, `EnableFirewall()`, `RemoveOldKernels(keep=2)`
3. Each action has a **polkit rule**. Read actions use `auth_admin_keep`, so one password covers a few minutes. Risky write actions ask every time.
4. The helper checks every input (for example, a service name must exist) and writes every call to an audit log.

## What is built

The code is `crates/deecure-helper`. It is installed as `/usr/libexec/deecure/deecure-helper` and never started by the package: D-Bus starts it on the first call (`io.github.krisskad0.DeeCure.Helper`, object `/io/github/krisskad0/DeeCure/Helper`, interface `io.github.krisskad0.DeeCure.Helper1`).

| Method | What it does |
| --- | --- |
| `Run(call)` | One admin fix step: a JSON fix call (`deecure_core::fix::Call`). Unknown operations cannot be read; user-level operations are refused. |
| `Read(what, arg)` | An admin read-out: `smart` (arg: a disk name such as `nvme0n1`), `firewall_rules`, `crontabs`, `accounts` (password status, never hashes), `memory_modules`, `serial` |
| `Version` (property) | The helper's version, no approval needed |

| Admin operation | Runs | Checks |
| --- | --- | --- |
| `clean_apt_cache` | `apt-get clean` | - |
| `vacuum_journal` | `journalctl --vacuum-size=<N>M` | 50-10000 MB |
| `remove_snap_revision` | `snap remove <name> --revision=<rev>` | Name and revision pattern; it must be a **disabled** revision |
| `autoremove` | `apt-get -y autoremove` | - |
| `purge_residual` | `apt-get -y purge -- <packages>` | Package names; each must be in the "config files only" state |
| `upgrade_packages` | `apt-get -y install --only-upgrade -- <packages>` | Package names; each must be installed |
| `set_firewall` | `ufw --force enable` / `ufw disable` | ufw installed |
| `restart_service` | `systemctl restart -- <unit>` | Unit pattern; it must exist |
| `set_service_enabled` | `systemctl enable/disable --now -- <unit>` | As above; essential units (D-Bus, logind, journald, udev, GDM, polkit, NetworkManager) can never be turned off |
| `remove_kernels` | `apt-get -y purge -- linux-image-…` | Version pattern; never the running or newest kernel; at least two are always kept |
| `set_battery_action` | Edits `/etc/UPower/UPower.conf`, then `systemctl try-restart upower` | PowerOff, Hibernate, or HybridSleep at 1-20%. If a later step fails, the file is put back. Returns the old values for undo. |
| `set_charge_limits` | Writes `/sys/class/power_supply/<battery>/charge_control_{start,end}_threshold` | Battery name pattern (`BAT0`, `BATT`, `CMB0`) and `type` is `Battery`; stop 20-100%; start below stop, and only when the battery has a start limit; written in an order that never puts start at or above stop. Returns the old limits for undo. |
| `set_charge_mode` | Writes `…/<battery>/charge_types` | The mode must be one the battery lists. Returns the old mode. |
| `set_turbo` | Writes `/sys/devices/system/cpu/intel_pstate/no_turbo` or `…/cpufreq/boost` | One of the two must exist. Returns the old state. |
| `set_device_wakeup` | Writes `/sys/devices/<device>/power/wakeup` | Path pattern (letters, digits, `:._-/`, no `..`, not absolute); it must resolve inside `/sys/devices` after links and have the file. The lid, power and sleep buttons, the built-in keyboard (i8042, serio), and USB keyboards are never stopped from waking the computer. Returns the old value. |
| `set_device_power_saving` | Writes `/sys/devices/<device>/power/control` (`auto` or `on`) | As above; keyboards, mice and other input (USB class 03), sound devices (USB class 01, PCI class 04) never get power saving. Returns the old value. |

The five power calls write one value to one checked file each; they run no program. All of them go through a test root in the unit tests (`crates/deecure-helper/src/power.rs`), never the real `/sys`: a container shares the host's kernel, so a container test could change the real computer.

### Kept after restart

Linux forgets charge limits, turbo, and device power settings at every start. The helper keeps them (`crates/deecure-helper/src/saved.rs`):

- After a power call works, it is saved in `/etc/deecure/power.json` (root only can change it), one entry per setting: `charge_limits:<battery>`, `charge_mode:<battery>`, `turbo`, `wakeup:<device>`, `power_saving:<device>`. Each entry has the change, the value before the first change, and the device's vendor and product (`idVendor:idProduct` or `vendor:device`) or the battery's model.
- Setting a value back to the one before the first change removes its entry; the last one removes the file. No other call is ever saved.
- At boot, `deecure-power.service` (enabled by the package, skipped while the file does not exist) runs `deecure-helper --apply-saved`. It uses the file only when it is a plain file owned by root that no one else can write. Every entry goes through the same checks as a change from the app; a device setting is skipped when a different device is in that place. Batteries and devices that are not there yet are waited for up to 20 seconds. The unit is `Type=exec`, so this never holds up the boot, and it runs with no extra powers, no network, and a read-only system except `/sys`, `/run/deecure`, and `/var/log/deecure`.
- Each result (setting, change tried, done or why not) goes to `/run/deecure/power-boot.json` and the audit log. The app shows the list with this start's result in Power › Power Profile.
- Purging the package removes `/etc/deecure/power.json`, `/run/deecure`, and `/var/log/deecure`.

How every call is handled:

1. The caller's user ID is looked up on the bus.
2. polkit is asked, with user interaction allowed (the password box). Each operation has its own polkit action, `io.github.krisskad0.deecure.<operation>` (policy: `packaging/helper/`). Reads and cache cleaning keep the approval for a few minutes; changes ask every time.
3. Inputs are checked against strict patterns **and** the live system.
4. Programs run with a fixed argument list and a clean environment (`PATH=/usr/sbin:/usr/bin:/sbin:/bin`, `LC_ALL=C`), never through a shell. One write at a time.
5. A JSON line goes to `/var/log/deecure/helper.log`: time, caller, operation, inputs, and result.
6. After 2 minutes without calls (and no write running), the helper exits.

## Tests

- Unit tests: command building, every refusal (shell characters, paths, missing units, essential units, the running kernel, wrong package states), the UPower edit, and literal arguments.
- `make helper-test` (also in CI): a clean Ubuntu container with a real system D-Bus and polkit. An approved user cleans the apt cache; a bad unit name, an unknown operation, and a user-level call are refused; an unapproved user is denied; a failed step puts `UPower.conf` back; the audit log records everything and never a password hash; `--apply-saved` refuses a settings file others can write and a run without root, and reports a battery that is not there without writing anything.
- Not yet: apply → verify → undo of every fix in a full VM, and the security review by two people (Phase 2 tasks 7 and 9).

