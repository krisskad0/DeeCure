# Ubuntu problems and how to fix them with DeeCure

[Docs](../../README.md) › [User guide](../README.md)

The problems Ubuntu users really hit, from everyday ones to deep system ones, and how DeeCure finds and fixes each of them. Every section also gives the command to do it by hand.

For Ubuntu 24.04 LTS and 26.04 LTS, on desktops, laptops, and servers.

## What the words in the "DeeCure" column mean

- **Finds and fixes:** DeeCure warns or shows the cause, and has a fix you can apply with one click. Each fix shows its exact steps first, asks for your password only for admin steps, checks again afterwards, and can be undone where that is possible.
- **Warns:** DeeCure warns early and explains what to do, and you do the fix by hand.
- **Shows:** DeeCure shows the facts you need, but it does not warn about this problem yet.

## Level 1: everyday problems

| Problem | DeeCure |
| --- | --- |
| [Laptop battery dies overnight while asleep](01-battery-and-sleep.md#ubuntu-laptop-battery-dies-overnight-while-asleep) | Finds and fixes |
| [Battery drains fast: which app is using it](01-battery-and-sleep.md#ubuntu-battery-drains-fast-find-which-app-is-using-the-battery) | Finds and fixes |
| [Battery does not last like before](01-battery-and-sleep.md#laptop-battery-does-not-last-like-before-check-battery-health-on-ubuntu) | Finds and fixes |
| [Laptop wakes up from sleep by itself](01-battery-and-sleep.md#ubuntu-laptop-wakes-up-from-sleep-by-itself) | Finds and fixes |
| [The disk is full or low on space](02-disk-space.md#ubuntu-says-the-disk-is-full-or-low-on-space) | Finds and fixes |
| [The disk is filling up fast](02-disk-space.md#ubuntu-disk-filling-up-fast-when-will-it-be-full) | Finds and fixes |
| [The computer is slow](03-slow-or-frozen.md#why-is-my-ubuntu-computer-so-slow) | Finds and fixes |
| [One app uses 100% CPU](03-slow-or-frozen.md#one-app-uses-100-cpu-on-ubuntu) | Finds and fixes |
| [Ubuntu takes too long to start](03-slow-or-frozen.md#ubuntu-takes-too-long-to-start) | Finds and fixes |
| [The fan is loud even when idle](04-heat-and-fans.md#ubuntu-laptop-fan-is-loud-even-when-idle) | Finds and fixes |
| [Security updates are waiting](05-updates-and-apt.md#security-updates-are-waiting-on-ubuntu) | Finds and fixes |
| [Ubuntu says "restart required"](05-updates-and-apt.md#ubuntu-says-restart-required-after-updates) | Warns |
| [Wi-Fi connected but no internet](06-network-wifi-bluetooth.md#ubuntu-wi-fi-connected-but-no-internet) | Warns, and tests where it breaks |
| [Wi-Fi is weak or keeps disconnecting](06-network-wifi-bluetooth.md#ubuntu-wi-fi-is-weak-or-keeps-disconnecting) | Finds and fixes |
| [Which app is using my internet](06-network-wifi-bluetooth.md#which-app-is-using-my-internet-on-ubuntu) | Shows |
| [Bluetooth is not working](06-network-wifi-bluetooth.md#bluetooth-is-not-working-on-ubuntu) | Shows |
| [Is the firewall on?](07-security-and-privacy.md#is-the-ubuntu-firewall-on-turn-on-the-firewall) | Finds and fixes |
| [Which apps use my camera and microphone](07-security-and-privacy.md#which-apps-use-my-camera-and-microphone-on-ubuntu) | Shows |

## Level 2: power-user problems

| Problem | DeeCure |
| --- | --- |
| ["dpkg was interrupted": apt is broken](05-updates-and-apt.md#dpkg-was-interrupted-apt-is-broken-on-ubuntu) | Finds and fixes |
| [The upgrade to 26.04 stopped half way](05-updates-and-apt.md#ubuntu-upgrade-to-2604-stopped-half-way) | Finds and fixes the stopped installs |
| [Automatic security updates stopped](05-updates-and-apt.md#ubuntu-automatic-security-updates-stopped-working) | Warns |
| [/boot is full, and updates fail](02-disk-space.md#boot-is-full-on-ubuntu-and-updates-fail) | Finds and fixes |
| [Old snap versions take gigabytes](02-disk-space.md#old-snap-versions-take-gigabytes-on-ubuntu) | Finds and fixes |
| [System logs use gigabytes](02-disk-space.md#ubuntu-system-logs-too-big-varlogjournal-uses-gigabytes) | Finds and fixes |
| [APT cache, trash, thumbnails, and leftover packages](02-disk-space.md#clean-the-apt-cache-trash-thumbnails-and-leftover-packages-on-ubuntu) | Finds and fixes |
| [Ubuntu freezes, and apps close (out of memory)](03-slow-or-frozen.md#ubuntu-freezes-and-apps-close-by-themselves-out-of-memory) | Finds and fixes |
| [Everything waits for the disk](03-slow-or-frozen.md#everything-waits-for-the-disk-on-ubuntu) | Finds and fixes |
| [The CPU slows down under load (throttling)](04-heat-and-fans.md#cpu-throttling-on-ubuntu-the-computer-slows-down-under-load) | Finds and fixes |
| [The battery drains even when plugged in](01-battery-and-sleep.md#ubuntu-laptop-battery-drains-even-when-plugged-in) | Warns |
| [The internet is slow or loses packets](06-network-wifi-bluetooth.md#ubuntu-internet-is-slow-or-loses-packets) | Tests where it breaks |
| [A service failed](08-services-jobs-and-time.md#a-service-failed-on-ubuntu) | Finds and fixes |
| [A service keeps restarting](08-services-jobs-and-time.md#a-service-keeps-restarting-on-ubuntu) | Warns, with a restart button |
| [A cron job or timer did not run](08-services-jobs-and-time.md#a-cron-job-or-timer-did-not-run-on-ubuntu) | Finds and fixes |
| [The clock is wrong, or time sync is off](08-services-jobs-and-time.md#ubuntu-clock-is-wrong-or-time-sync-is-off) | Warns |
| [The printer is stuck](08-services-jobs-and-time.md#ubuntu-printer-is-stuck-or-not-printing) | Shows, with a restart for the print service |

## Level 3: developer and server problems

| Problem | DeeCure |
| --- | --- |
| [Docker uses too much disk space](09-developers-and-containers.md#docker-uses-too-much-disk-space) | Finds and fixes |
| [VS Code: file watcher limit reached](09-developers-and-containers.md#vs-code-system-limit-for-number-of-file-watchers-reached-on-ubuntu) | Finds and fixes |
| [A Docker container always uses high CPU](09-developers-and-containers.md#a-docker-container-always-uses-high-cpu) | Shows |
| [A Docker container is open to the internet](09-developers-and-containers.md#a-docker-container-is-open-to-the-internet) | Warns |
| [Port already in use: which process is listening](09-developers-and-containers.md#port-already-in-use-which-process-is-listening-on-ubuntu) | Shows |
| ["Too many open files"](09-developers-and-containers.md#too-many-open-files-on-ubuntu) | Shows |
| [My app leaks memory over time](09-developers-and-containers.md#my-app-leaks-memory-over-time-on-ubuntu) | Shows |
| [Virtual machines](09-developers-and-containers.md#virtual-machines-on-ubuntu) | Shows |
| [SSH lets root sign in with a password](07-security-and-privacy.md#ssh-lets-root-sign-in-with-a-password) | Warns |
| [Another account has admin (sudo) rights](07-security-and-privacy.md#another-account-has-admin-sudo-rights-on-ubuntu) | Warns |
| [AppArmor is off](07-security-and-privacy.md#apparmor-is-off-on-ubuntu) | Warns |
| [Were my system files changed or damaged?](05-updates-and-apt.md#were-my-ubuntu-system-files-changed-or-damaged) | Shows, on request |
| [Ubuntu sends my package list every week](07-security-and-privacy.md#ubuntu-sends-my-list-of-installed-packages-every-week) | Warns |
| [Free space, but "No space left on device" (inodes)](02-disk-space.md#disk-has-free-space-but-ubuntu-says-it-is-full-inodes) | Warns |

## Level 4: drivers, kernel, firmware, and hardware

| Problem | DeeCure |
| --- | --- |
| [NVIDIA black screen after a kernel update](10-drivers-kernel-and-firmware.md#nvidia-black-screen-or-missing-driver-after-a-kernel-update-on-ubuntu) | Finds and fixes |
| [Ubuntu is not using my graphics card](10-drivers-kernel-and-firmware.md#ubuntu-is-not-using-my-graphics-card-software-rendering) | Finds and fixes |
| [The second graphics card drains the battery](10-drivers-kernel-and-firmware.md#second-graphics-card-drains-the-battery-on-ubuntu) | Finds and fixes |
| [Firmware and BIOS updates](10-drivers-kernel-and-firmware.md#firmware-and-bios-updates-on-ubuntu) | Finds and fixes |
| [A device has no driver](10-drivers-kernel-and-firmware.md#a-device-has-no-driver-on-ubuntu) | Finds and fixes, when Ubuntu has the driver |
| [The kernel crashed](10-drivers-kernel-and-firmware.md#the-ubuntu-kernel-crashed) | Warns |
| [The screen froze: the graphics card hung](10-drivers-kernel-and-firmware.md#the-screen-froze-for-a-moment-graphics-card-hung-on-ubuntu) | Warns |
| [Wi-Fi or Bluetooth firmware crashed](10-drivers-kernel-and-firmware.md#wi-fi-or-bluetooth-firmware-crashed-on-ubuntu) | Warns |
| [Secure Boot is off](07-security-and-privacy.md#secure-boot-is-off-on-ubuntu) | Warns |
| [A CPU security flaw is not protected](07-security-and-privacy.md#a-cpu-security-flaw-is-not-protected-on-ubuntu) | Warns |
| [Is my SSD dying?](11-hardware-health.md#is-my-ssd-dying-check-ssd-health-on-ubuntu) | Warns early |
| [The disk became read-only](11-hardware-health.md#ubuntu-disk-became-read-only-or-has-filesystem-errors) | Warns |
| [Is my RAM failing?](11-hardware-health.md#is-my-ram-failing-memory-errors-on-ubuntu) | Warns |
| [Is my RAM running at full speed?](11-hardware-health.md#is-my-ram-running-at-full-speed-on-ubuntu) | Shows |
| [The NVMe SSD or GPU is slower than it should be](11-hardware-health.md#my-nvme-ssd-or-gpu-is-slower-than-it-should-be-pcie-link) | Shows, and warns about link errors |
| [A USB 3 drive is slow](11-hardware-health.md#my-usb-3-drive-is-slow-on-ubuntu) | Shows |
| [A USB port turned off](11-hardware-health.md#a-usb-port-turned-off-on-ubuntu) | Warns |
| [What a USB-C charger or dock supports](11-hardware-health.md#what-does-my-usb-c-charger-or-dock-support-on-ubuntu) | Shows |
| [The fan stopped, or the cooling cannot keep up](04-heat-and-fans.md#ubuntu-laptop-fan-stopped-or-the-cooling-cannot-keep-up) | Warns |
| [The computer switched off from heat](04-heat-and-fans.md#ubuntu-computer-switched-off-by-itself-from-heat) | Warns |
| [The battery is too hot or reports a fault](01-battery-and-sleep.md#laptop-battery-is-too-hot-or-reports-a-fault) | Warns |

## The pages, by topic

| Page | What it covers |
| --- | --- |
| [01-battery-and-sleep.md](01-battery-and-sleep.md) | Battery drain, sleep, wake-ups, battery health, chargers |
| [02-disk-space.md](02-disk-space.md) | A full disk, a disk that fills fast, snaps, logs, old kernels, caches, inodes |
| [03-slow-or-frozen.md](03-slow-or-frozen.md) | A slow computer, busy apps, out-of-memory freezes, slow boot, disk waits |
| [04-heat-and-fans.md](04-heat-and-fans.md) | Fan noise, heat, CPU throttling, heat shutdowns |
| [05-updates-and-apt.md](05-updates-and-apt.md) | Broken apt, stopped upgrades, security and automatic updates, changed files |
| [06-network-wifi-bluetooth.md](06-network-wifi-bluetooth.md) | No internet, Wi-Fi drops, slow network, traffic per app, Bluetooth |
| [07-security-and-privacy.md](07-security-and-privacy.md) | Firewall, AppArmor, SSH, admin accounts, Secure Boot, CPU flaws, privacy |
| [08-services-jobs-and-time.md](08-services-jobs-and-time.md) | Failed services, restart loops, scheduled jobs, the clock, printers |
| [09-developers-and-containers.md](09-developers-and-containers.md) | Docker, VS Code file watchers, ports, open files, memory leaks, VMs |
| [10-drivers-kernel-and-firmware.md](10-drivers-kernel-and-firmware.md) | NVIDIA and DKMS, graphics drivers, kernel and firmware crashes, firmware updates |
| [11-hardware-health.md](11-hardware-health.md) | SSD health, read-only disks, memory errors, PCIe and USB speed, USB-C |

Not covered yet: DeeCure does not yet warn when the battery drains faster than your usual, when a container stays busy for a long time, or why a Bluetooth connection fails.
