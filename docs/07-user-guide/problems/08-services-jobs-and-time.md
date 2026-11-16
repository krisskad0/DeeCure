# Services, scheduled jobs, printing, and the clock on Ubuntu

[Docs](../../README.md) › [User guide](../README.md) › [Problems and fixes](README.md)

Failed services, restart loops, scheduled jobs that did not run, a stuck printer, and a wrong clock.

## A service failed on Ubuntu

**Signs:** something stopped working in the background (printing, sharing, a database, a VPN), or a "failed" line appears during boot.

**Find it in DeeCure:**

- DeeCure warns that a service has stopped working, and names it.
- **Services › System Services** shows every service with its state, CPU time, memory, and restarts.
- **Logs › Journal** shows the service's recent log lines.

**Fix it with DeeCure:** click **Fix** on the warning in **Overview › Issues** to restart the service, or **Restart** in **Services › System Services**. DeeCure checks again after the restart.

**By hand:**

```bash
systemctl --failed
journalctl -u NAME -b
sudo systemctl restart NAME
```

## A service keeps restarting on Ubuntu

**Signs:** a service starts, crashes, and starts again, often using CPU and filling the log.

**Find it in DeeCure:** DeeCure warns that the service keeps restarting, with the number of restarts. **Services › System Services** shows its restarts and its unit file, including local overrides.

**Fix it with DeeCure:** read its log in **Logs › Journal** to see why it crashes. Stop or restart it from **Services**.

**By hand:** `systemctl status NAME` and `journalctl -u NAME -n 50`.

## A cron job or timer did not run on Ubuntu

**Signs:** a backup, a script, or a clean-up that should run every night did not.

**Find it in DeeCure:**

- DeeCure warns that a scheduled job failed, and names it.
- **Scheduled Jobs › All Jobs** lists cron jobs, systemd timers, anacron, and `at` jobs, with every run of the last 7 days and its result.
- **Scheduled Jobs › Calendar** shows when each job runs next.

**Fix it with DeeCure:** click **Fix** on the warning to run the job's service again, then check its result in Scheduled Jobs.

**By hand:**

```bash
systemctl list-timers
crontab -l
journalctl -u cron --since today
```

## Ubuntu clock is wrong, or time sync is off

**Signs:** the clock is a few minutes off, or websites show certificate errors.

**Find it in DeeCure:** DeeCure warns **"The clock is not in sync"** or **"Time sync is off"**. **Operating System › Kernel** (Expert view) shows the time server and the clock offset.

**Fix it with DeeCure:** DeeCure warns and explains. Turn time sync on by hand.

**By hand:** `timedatectl`, then `sudo timedatectl set-ntp true`.

## Ubuntu printer is stuck or not printing

**Signs:** jobs wait in the queue forever.

**Find it in DeeCure:** **Devices › Ports & Devices** (Expert view) shows each printer's queue, its state, and its last error.

**Fix it with DeeCure:** restart the print service (`cups`) in **Services › System Services**.

**By hand:** `lpstat -t`, then `sudo systemctl restart cups`.

---

How a fix works, and how to undo it: see [Fix a problem](../using-deecure.md#3-fix-a-problem). Every change DeeCure makes is listed in **Activity**, with **Undo** where it can be undone.
