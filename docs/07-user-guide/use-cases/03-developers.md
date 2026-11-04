# DeeCure for developers, data scientists, and engineers

[Docs](../../README.md) › [User guide](../README.md) › [Who DeeCure is for](README.md)

**For:** software developers, DevOps and platform engineers, data scientists, and ML engineers who work on an Ubuntu laptop or workstation.

## What usually goes wrong

- A build or a training run is slow, and `top` does not say why.
- Docker quietly eats 30 GB of disk.
- A container publishes a database port to the whole network, past the firewall.
- The IDE or a file watcher hits the file-watch limit.
- Something was killed for memory in the middle of the work.

DeeCure answers all of this in one place, with the data source shown next to every number in the **Expert** view.

## How DeeCure makes it easy

| You want to… | Open | What you get |
| --- | --- | --- |
| See why the machine stalls | **Performance › Pressure (PSI)** and **Diagnostics › Performance Check** | Real stall time for CPU, memory, and disk, and the app behind it, not just "100% busy" |
| See real memory per app | **Performance › Memory** and **Processes** | What really fills RAM (cache shown apart), memory per app and per service, swap, and apps killed for memory |
| See which app uses CPU over time | **Performance › CPU** | The busiest apps and services over 24 hours, 7 days, or 30 days, also apps you closed since |
| See disk and network use per app | **Performance › Disk I/O** and **Network › Traffic** | Disk reads and writes per process, and download and upload per app (TCP only, and said so) |
| Keep containers in check | **Containers** | Docker containers, images, volumes, and virtual machines. A warning when a published port bypasses the UFW firewall. |
| Get disk space back | **Storage › Cleanup** | Unused Docker images and build cache, old snap versions, caches, and logs, each with its size |
| Find open ports | **Network › Open Ports** and **Connections** | Every listening port, its program, and whether the firewall lets it in |
| Read services and jobs | **Services** and **Scheduled Jobs** | System and user services, startup apps, a calendar of cron jobs and systemd timers |
| Read the kernel | **System › Kernel** | Installed kernels with the fallback, loaded modules, limits, and kernel settings with the file that set them |
| Check the hardware link speed | **Hardware › Peripherals › PCIe links** (Expert view) | Whether the SSD, the graphics card, and other chips run at their full PCIe speed |
| Read logs without `journalctl` flags | **Logs** | The journal, errors grouped with explanations, crashes, and every boot |

![DeeCure memory page on Linux: what fills RAM as one bar with cache striped, a verdict, usage over 24 hours, where memory goes, and the apps and services using the most](../images/deecure-expert-memory-map.png)

## Improvements a developer machine often needs

The **Optimize** page offers these only when a measurement shows they are needed:

- **Raise the file-watch limit** when more than 80% of file watches are in use.
- **Turn on compressed swap (zswap)** when swap-ins and hard page faults stay high for 10 minutes.
- **Prune unused Docker images or build cache** when they pass 2 GB.
- **Stop "wait for network" from slowing boot** when nothing on the machine needs the network before sign-in.
- **Remove old kernels** when more than the running one and a fallback are installed.

Each one shows a preview of its exact commands first, and most can be undone.

## Warnings that save your work

- **An app was killed for memory** in the last 24 hours.
- **Swap almost full** while less than 5% of memory is free.
- **Service keeps restarting**: 10 or more restarts since boot.
- **Everything waits for the disk**: disk pressure stays high for 5 minutes.

## From the terminal

```bash
deecure status --json          # health and problems, for scripts
deecure history --since 2d     # what changed: packages, config files, devices
deecure fix --list             # every fix, then preview one with: deecure fix <name>
```

## Your first 10 minutes

1. Switch to the **Expert** view.
2. Open **Containers** and **Network › Open Ports**. Close anything you did not mean to expose.
3. Open **Performance › Memory**. See what really uses your RAM.
4. Open **Optimize** and click **Scan**.

---

Previous: [Laptop owners](02-laptop-owners.md) · Next: [Gamers and creators](04-gamers-and-creators.md) · [All roles](README.md)
