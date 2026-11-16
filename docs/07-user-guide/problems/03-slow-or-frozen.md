# Ubuntu is slow or freezes

[Docs](../../README.md) › [User guide](../README.md) › [Problems and fixes](README.md)

A slow computer, one app eating the CPU, freezes from low memory, a slow start, and a disk that makes everything wait.

## Why is my Ubuntu computer so slow?

**Signs:** apps open slowly, typing lags, the mouse stutters.

**Why it happens:** one thing is the bottleneck: the CPU is busy, memory is short, the disk is overloaded, or the CPU is hot and slows itself down.

**Find it in DeeCure:**

1. Open **Diagnostics › Performance Check** and run it while the computer feels slow.
2. It names the main cause in plain English. For example: "The CPU is very hot and slows itself down to cool off. Check the fans and air vents."
3. It also lists the top processes behind it.

**Fix it with DeeCure:** follow the cause it names. Each one is covered below: [one app uses all the CPU](#one-app-uses-100-cpu-on-ubuntu), [memory is short](#ubuntu-freezes-and-apps-close-by-themselves-out-of-memory), [the disk is the bottleneck](#everything-waits-for-the-disk-on-ubuntu), or [heat](04-heat-and-fans.md).

**By hand:** `top -o %CPU`, `free -h`, and `cat /proc/pressure/{cpu,memory,io}`.

**Limits:** when nothing is slow during the check, it says "No bottleneck found". Run it again while the problem happens.

## One app uses 100% CPU on Ubuntu

**Signs:** the fan spins up, the laptop gets warm, and one app or background task stays at the top of the list. Common examples are the file indexer (`tracker-miner-fs`), a stuck browser tab, or `snapd` after an update.

**Find it in DeeCure:**

- **Processes › Top Consumers** shows the apps and processes using the most CPU and memory.
- Click a row to see what started it (an app, a service, or the terminal), the package it belongs to, and its threads.
- **Performance › CPU** shows the CPU use over time, so you can see when it started.

**Fix it with DeeCure:** click the process in **Processes**, then **End process…**. If it is a service that keeps coming back, see [a service keeps failing or restarting](08-services-jobs-and-time.md#a-service-keeps-restarting-on-ubuntu).

**By hand:** `top -o %CPU`, then `kill PID`.

## Ubuntu freezes, and apps close by themselves (out of memory)

**Signs:** the whole desktop freezes for a minute, then Chrome, VS Code, or the whole session closes.

**Why it happens:** memory runs out, the system swaps to disk, and then Ubuntu closes apps to recover. On 8 GB machines this happens often.

**Find it in DeeCure:**

- DeeCure warns **"The computer is short of memory"**, **"Swap is almost full"**, and **"Apps were stopped because memory ran out"**.
- **Performance › Memory** shows what really fills the memory: apps, file cache that can be freed, shared memory, and the kernel.
- It also shows memory per app and per service over 24 hours, 7 days, and 30 days.
- In Expert, it shows which processes the system would stop first when memory runs out.

**Fix it with DeeCure:**

1. Find the app using the most memory, and close it, or end it in **Processes**.
2. **Optimize** offers **turn on zswap** (compressed swap) when swapping stayed high for 10 minutes or more in the last 24 hours. zswap keeps swapped memory compressed in RAM, which is much faster than the disk.

**By hand:**

```bash
free -h
cat /proc/pressure/memory
journalctl -u systemd-oomd --since today    # which apps were closed
```

## Ubuntu takes too long to start

**Signs:** a long wait between the logo and the login screen, or between login and a usable desktop.

**Find it in DeeCure:**

- **Operating System › Boot** shows the boot time per phase, the boot time trend, the slowest services, the critical chain (what boot waited for), and whether boot waits for the network.
- **Services › Startup Applications** lists the apps that start when you log in.

**Fix it with DeeCure:** turn off start-up apps you do not need in **Services › Startup Applications**. Check the slowest services in Operating System › Boot.

**By hand:**

```bash
systemd-analyze
systemd-analyze blame | head
systemd-analyze critical-chain
```

## Everything waits for the disk on Ubuntu

**Signs:** the computer feels stuck while copying files or installing updates, and the disk light stays on.

**Find it in DeeCure:**

- DeeCure warns **"Everything is waiting for the disk."**
- **Performance › Disk I/O** shows reads and writes per app and per process.
- **Performance › Pressure (PSI)** shows how long everything waited for the disk.

**Fix it with DeeCure:**

1. Find the app writing the most in Performance › Disk I/O, and pause or close it.
2. For a hard disk (not an SSD), **Optimize** can offer **Use a better scheduler for hard disk**.

**By hand:** `cat /proc/pressure/io` and `sudo iotop`.

## Ubuntu is slow only on battery, or under load

The CPU may be slowing itself down because of heat or power limits. See [CPU throttling](04-heat-and-fans.md#cpu-throttling-on-ubuntu-the-computer-slows-down-under-load).

---

How a fix works, and how to undo it: see [Fix a problem](../using-deecure.md#3-fix-a-problem). Every change DeeCure makes is listed in **Activity**, with **Undo** where it can be undone.
