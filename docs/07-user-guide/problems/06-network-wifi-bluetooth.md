# Wi-Fi, internet, and Bluetooth problems on Ubuntu

[Docs](../../README.md) › [User guide](../README.md) › [Problems and fixes](README.md)

No internet, weak or dropping Wi-Fi, a slow network, which app uses the internet, and Bluetooth.

## Ubuntu Wi-Fi connected but no internet

**Signs:** the Wi-Fi icon shows connected, but websites do not load.

**Why it happens:** the problem is past your computer: the router, the gateway, or DNS (the service that turns names into addresses).

**Find it in DeeCure:**

- DeeCure warns **"Connected, but the internet does not answer"**, or **"Not connected to a network"**.
- **Diagnostics › Network Test** checks each step in order (the link, the gateway, DNS, the internet) and says where it breaks.
- **Network › DNS & VPN** shows which DNS server answers, how fast, and any VPN routes.

**Fix it with DeeCure:** run **Diagnostics › Network Test** and follow the step it names. When it is the router or the provider, DeeCure says so, so you do not waste time on the laptop.

**By hand:**

```bash
ping -c3 1.1.1.1              # does the internet answer?
resolvectl query ubuntu.com   # does DNS work?
ip route                      # which gateway is used
```

## Ubuntu Wi-Fi is weak or keeps disconnecting

**Signs:** Wi-Fi drops every few minutes, after sleep, or far from the router.

**Why it happens:** a weak signal, Wi-Fi power saving that puts the card to sleep, or a driver problem after a kernel update.

**Find it in DeeCure:**

- DeeCure warns **"The Wi-Fi signal is weak"** with the signal in dBm, and warns when a network link has errors.
- **Network › Interfaces** shows the signal and noise in dBm, the band and channel, the link speed, how often the link went down, and errors and dropped packets.

**Fix it with DeeCure:**

1. In **Power › Power Profile › Power controls** (Expert view), turn **Wi-Fi power saving** off. It is kept after a restart.
2. If the signal is weak, move closer to the router or change the band (5 GHz is faster, 2.4 GHz reaches further).

**By hand:**

```bash
nmcli dev wifi                # signal and channel
iw dev                        # adapter name
# Wi-Fi power saving off (2 = off):
printf '[connection]\nwifi.powersave = 2\n' | sudo tee /etc/NetworkManager/conf.d/wifi-powersave.conf
sudo systemctl restart NetworkManager
```

**Limits:** some Wi-Fi drops come from driver bugs in a new kernel. No app can fix those. Try the older kernel from the boot menu, and wait for a fixed update.

## Ubuntu internet is slow or loses packets

**Find it in DeeCure:**

- **Diagnostics › Network Test** says when "the network works, but it is slow or loses packets".
- **Network › Interfaces** shows errors, dropped packets, and TCP resends.

**Fix it with DeeCure:** use the result to decide between the Wi-Fi signal (see above), the cable, or the provider.

## Which app is using my internet on Ubuntu?

**Find it in DeeCure:** **Network › Traffic** shows the download and upload of each app, now and over 24 hours, 7 days, and 30 days.

**Fix it with DeeCure:** close the app, or end it in **Processes**.

**By hand:** `sudo nethogs`

## Bluetooth is not working on Ubuntu

**Signs:** headphones will not connect, or connect but do not play, often after an update or after sleep.

**Find it in DeeCure:**

- **Hardware › All Parts** shows the Bluetooth adapters, and the Bluetooth devices that are connected.
- **Power › Power Profile › Power controls** (Expert view) shows whether the Bluetooth radio is on.

**Fix it with DeeCure:** if the Bluetooth radio is off, turn it on in **Power › Power Profile › Power controls** (Expert view).

**By hand:**

```bash
rfkill list                     # is Bluetooth blocked?
bluetoothctl show               # is the adapter powered?
systemctl status bluetooth
# Remove the device, then pair it again:
bluetoothctl remove AA:BB:CC:DD:EE:FF
```

**Limits:** DeeCure does not yet find why a Bluetooth connection fails. It shows the adapter, the devices, and firmware crashes (see [device firmware crashed](10-drivers-kernel-and-firmware.md#wi-fi-or-bluetooth-firmware-crashed-on-ubuntu)).

---

How a fix works, and how to undo it: see [Fix a problem](../using-deecure.md#3-fix-a-problem). Every change DeeCure makes is listed in **Activity**, with **Undo** where it can be undone.
