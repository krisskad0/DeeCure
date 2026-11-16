# Disk space problems on Ubuntu

[Docs](../../README.md) › [User guide](../README.md) › [Problems and fixes](README.md)

A full disk, a disk that fills up fast, and the hidden places that eat space: old snaps, logs, old kernels, caches, and Docker.

Every clean-up shows its exact steps and sizes before it runs. Steps that need admin rights ask for your password once.

## Ubuntu says the disk is full or low on space

**Signs:** "Low disk space" notifications, updates fail, apps cannot save files.

**Why it happens:** space goes to places most people never look at: old snap versions, the system log, the APT cache, old kernels, the trash, thumbnails, and Docker.

**Find it in DeeCure:**

- DeeCure warns **"Main disk is low on space"**, or that another disk is low on space.
- **Storage › Overview** shows used and free space for each disk.
- **Storage › Disk Usage** shows what fills the disk: your home folder, apps, and the system.
- **Storage › Cleanup** lists what is safe to clean, with its size, and says whether each clean-up can be undone.

**Fix it with DeeCure:**

1. Open **Overview › Issues** and click **Fix** on the low-space warning.
2. DeeCure lists each clean-up step with its size. Untick any step you do not want.
3. Click **Apply**. DeeCure checks the free space again when it is done.

**By hand:**

```bash
df -h /                                   # free space on the main disk
sudo du -xh --max-depth=1 / | sort -h     # the biggest folders
```

## Ubuntu disk filling up fast: when will it be full?

**Signs:** you freed space last month, and it is almost full again.

**Why it happens:** something writes a little every day: logs, caches, downloads, snaps, or containers.

**Find it in DeeCure:**

- DeeCure looks at 14 days of disk use and warns early: **"/ will be full in about 12 days."**
- **Storage › Overview** shows a 30-day usage trend with the low-space line.

**Fix it with DeeCure:** click **Fix** on the warning in **Overview › Issues**. It offers the same safe clean-up steps as above, with their sizes. Then watch the trend in Storage › Overview.

**By hand:** `df -h` every few days, and `sudo du -xh --max-depth=2 /var | sort -h | tail` to see what grows.

## Old snap versions take gigabytes on Ubuntu

**Signs:** `/var/lib/snapd` is several gigabytes, even with few snap apps.

**Why it happens:** Ubuntu keeps older versions of every snap after each update, so it can roll back.

**Find it in DeeCure:** **Apps & Updates** shows old snap versions and the space each one uses. **Storage › Cleanup** shows the total.

**Fix it with DeeCure:**

1. Open **Optimize** and click **Preview** on **Remove disabled snap revisions**, then **Apply**.
2. Optimize can also set Ubuntu to keep fewer old versions from now on.

**By hand:**

```bash
snap list --all | awk '/disabled/{print $1, $3}'     # the old versions
sudo snap remove NAME --revision=REV                  # remove one
sudo snap set system refresh.retain=2                 # keep fewer in future
```

## Ubuntu system logs too big: /var/log/journal uses gigabytes

**Signs:** `/var/log` takes several gigabytes.

**Why it happens:** the system log (the journal) keeps growing until it hits its own large limit.

**Find it in DeeCure:** **Storage › Cleanup** shows how much space the log uses.

**Fix it with DeeCure:**

1. Shrink the log once in **Storage › Cleanup**, or through **Optimize**.
2. If it grows over 1 GB again, **Optimize** offers to set a size cap, so it stays small.

**By hand:**

```bash
journalctl --disk-usage
sudo journalctl --vacuum-size=200M
```

## /boot is full on Ubuntu, and updates fail

**Signs:** kernel updates fail with "No space left on device" for `/boot`.

**Why it happens:** old kernels pile up in a small `/boot` partition.

**Find it in DeeCure:**

- DeeCure warns **"/boot is almost full."**
- **Operating System › Kernel** lists the installed kernels and marks the one in use and the fallback.

**Fix it with DeeCure:** click **Fix** on the warning, or **Remove old…** in Operating System › Kernel. DeeCure always keeps the kernel in use and one fallback.

**By hand:** `sudo apt autoremove --purge`

## Clean the APT cache, trash, thumbnails, and leftover packages on Ubuntu

**Signs:** no single big folder, but many small leftovers add up.

**Find it in DeeCure:** **Storage › Cleanup** lists each one with its size.

**Fix it with DeeCure:** **Optimize** offers each clean-up only when it frees real space:

- Clean the APT package cache.
- Empty the trash, and clear old thumbnails.
- Remove packages that nothing needs any more.
- Remove the settings left behind by removed packages.
- Remove Flatpak runtimes that no app uses.

**By hand:**

```bash
sudo apt clean
sudo apt autoremove --purge
sudo apt purge '~c'              # settings left by removed packages
flatpak uninstall --unused
```

## Docker uses too much disk space on Ubuntu

Docker images and build cache can grow to tens of gigabytes. See [Docker uses too much disk space](09-developers-and-containers.md#docker-uses-too-much-disk-space).

## Disk has free space but Ubuntu says it is full (inodes)

**Signs:** `df -h` shows free space, but you get "No space left on device".

**Why it happens:** the disk has run out of file slots (inodes), usually because of millions of tiny files.

**Find it in DeeCure:**

- DeeCure warns **"/ is running out of file slots."**
- **Storage › Filesystems** shows the file slots used and left for each disk.

**Fix it with DeeCure:** DeeCure warns and explains. Remove the folder with the millions of small files by hand. Caches and mail queues are common causes.

**By hand:**

```bash
df -i
sudo du --inodes -x / 2>/dev/null | sort -n | tail
```

---

How a fix works, and how to undo it: see [Fix a problem](../using-deecure.md#3-fix-a-problem). Every change DeeCure makes is listed in **Activity**, with **Undo** where it can be undone. Deleted files cannot come back, and DeeCure says so before you apply.
