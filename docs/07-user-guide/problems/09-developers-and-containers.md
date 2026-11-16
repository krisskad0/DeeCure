# Developer problems on Ubuntu: Docker, VS Code, ports, and limits

[Docs](../../README.md) › [User guide](../README.md) › [Problems and fixes](README.md)

Docker eating the disk, the VS Code file-watch limit, busy containers, ports, open files, memory leaks, and virtual machines.

Many of these details show only in the **Expert** view. Switch the view at the top of the DeeCure window.

## Docker uses too much disk space

**Signs:** the disk fills up while you build images. `/var/lib/docker` takes tens of gigabytes.

**Why it happens:** old images, the build cache, and stopped containers stay on disk until you remove them.

**Find it in DeeCure:** **Containers & VMs › Images and Volumes** shows the unused images and the build cache, with their sizes. It also shows the disk use of each container.

**Fix it with DeeCure:**

1. In **Containers & VMs › Images and Volumes**, click **Prune unused…** for the images, or **Remove unused…** for the build cache. **Optimize** also offers them when they free real space.
2. DeeCure shows the exact `docker` command and the space it frees before it runs.

**By hand:**

```bash
docker system df
docker builder prune
docker image prune -a
```

**Limits:** removed images are not kept. You download or build them again when you need them.

## VS Code: "System limit for number of file watchers reached" on Ubuntu

**Signs:** VS Code, a dev server, or a test watcher stops noticing file changes, or shows the error above.

**Why it happens:** Linux limits how many files all your apps can watch at once, and big projects use many.

**Find it in DeeCure:** **Operating System › Kernel** (Expert view) shows the file-watch limit, used against the maximum.

**Fix it with DeeCure:** **Optimize** shows "Your apps use N of LIMIT file watches" when the limit is close to full, and offers to **raise the limit**. It is kept after a restart, and you can undo it.

**By hand:**

```bash
cat /proc/sys/fs/inotify/max_user_watches
echo fs.inotify.max_user_watches=524288 | sudo tee /etc/sysctl.d/60-file-watches.conf
sudo sysctl --system
```

## A Docker container always uses high CPU

**Signs:** the fan runs and the laptop is slow while you develop, and a container is the cause.

**Find it in DeeCure:** **Containers & VMs › Containers** shows the CPU, memory, disk, and network use of each running container.

**Fix it with DeeCure:** stop or fix the container that uses the CPU (a loop, a watcher, or a busy database).

**By hand:** `docker stats --no-stream`

**Limits:** DeeCure shows the CPU per container, but it does not yet warn when a container stays busy for a long time.

## A Docker container is open to the internet

**Signs:** a database or dev server started with `-p 5432:5432` can be reached from every network, not only from your computer.

**Find it in DeeCure:**

- DeeCure warns **"A container is open to every network"**.
- **Network › Open Ports** shows every open port, the process behind it, and who can reach it.

**Fix it with DeeCure:** DeeCure warns and explains. Publish the port on `127.0.0.1` only.

**By hand:** use `-p 127.0.0.1:5432:5432` in `docker run`, or `"127.0.0.1:5432:5432"` in Compose.

## Port already in use: which process is listening on Ubuntu?

**Signs:** "address already in use" when you start a server.

**Find it in DeeCure:** **Network › Open Ports** lists every open port with its process and who can reach it. Click the process to see its details.

**Fix it with DeeCure:** end the process in **Processes**, or stop its service in **Services**.

**By hand:** `sudo ss -ltnp | grep :8080`

## "Too many open files" on Ubuntu

**Find it in DeeCure:**

- **Operating System › Kernel** (Expert view) shows the open files of the whole system, used against the maximum.
- The details of each process show its open files against its own limit.

**Fix it with DeeCure:** DeeCure shows which process is near its limit. Raise that app's limit by hand, or fix the leak.

**By hand:** `cat /proc/sys/fs/file-nr` and `ulimit -n`.

## My app leaks memory over time on Ubuntu

**Find it in DeeCure:** **Performance › Memory** shows the memory of each app and service over 24 hours, 7 days, and 30 days, at every view level. A line that keeps going up is a leak.

**Fix it with DeeCure:** restart the app or service, and use the chart to show when it started.

## Virtual machines on Ubuntu

**Find it in DeeCure:** **Containers & VMs › Virtual Machines** lists the virtual machines (through `virsh`). In Expert, **Hardware › Firmware** shows whether the CPU supports virtualization, whether it is turned on in the firmware, and whether KVM is ready.

**By hand:** `virsh list --all` and `kvm-ok`.

---

How a fix works, and how to undo it: see [Fix a problem](../using-deecure.md#3-fix-a-problem). Every change DeeCure makes is listed in **Activity**, with **Undo** where it can be undone.
