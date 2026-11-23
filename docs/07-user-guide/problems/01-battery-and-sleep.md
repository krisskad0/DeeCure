# Battery and sleep problems on Ubuntu laptops

[Docs](../../README.md) › [User guide](../README.md) › [Problems and fixes](README.md)

Battery drain, a worn battery, a weak charger, and a laptop that wakes up in the bag. Each section says how DeeCure finds the cause and how to fix it.

Some details show only in the **Expert** view. Switch the view at the top of the DeeCure window.

## Ubuntu laptop battery dies overnight while asleep

**Signs:** you close the lid at 100%, and the next morning the battery is at 60% or empty. The laptop is warm in the bag.

**Why it happens:** many laptops sleep in a light mode called `s2idle`, where some parts stay awake. A device such as a mouse, a USB port, or the network card can also wake the laptop again and again. Sometimes the laptop never really goes to sleep.

**Find it in DeeCure:**

- **Power › Sleep** lists every suspend, with its result, how long it lasted, and how much battery it used.
- In Expert, it also shows the drain per hour while suspended, the sleep type in use and the types the laptop offers, what woke the computer, and which devices are allowed to wake it.

**Fix it with DeeCure:**

1. Open **Power › Sleep** and look at the battery used by each suspend and what woke the laptop.
2. If a device woke it, switch that device off in the list of devices that may wake the computer (Expert view).
3. If the laptop offers deep sleep, open **Power › Power Profile › Power controls** (Expert view) and set the suspend type to **deep**.
4. Risky power settings run for one session first. Click **Keep** to keep the setting.
5. When the laptop woke up 3 or more times in a week with the lid closed, **Optimize** names the last device that woke it and points you to Power › Sleep.

**By hand:**

```bash
cat /sys/power/mem_sleep          # the sleep type in [brackets] is the one in use
cat /proc/acpi/wakeup             # devices allowed to wake the laptop
journalctl -b -1 -u systemd-suspend.service   # what happened during the last suspend
```

**Limits:** some laptops offer only `s2idle`. DeeCure shows only the sleep types your laptop offers, so it cannot turn on deep sleep where the firmware does not support it.

## Ubuntu laptop wakes up from sleep by itself

**Signs:** the laptop is awake when you open the bag, or the screen lights up a few seconds after you close the lid.

**Why it happens:** a device (USB, network card, lid sensor, Bluetooth) sends a wake-up signal.

**Find it in DeeCure:** **Power › Sleep** (Expert view) shows what woke the computer from each suspend, how often each source fired, and every device allowed to wake it.

**Fix it with DeeCure:** in **Power › Sleep**, switch off waking for the device that keeps waking the laptop. DeeCure keeps the setting after a restart.

**By hand:** `cat /proc/acpi/wakeup` lists the devices, and `echo XHC | sudo tee /proc/acpi/wakeup` turns one device on or off (it toggles) until the next restart.

## Ubuntu battery drains fast: find which app is using the battery

**Signs:** the battery goes from 100% to 20% by lunch, even with light work.

**Why it happens:** one app or background task keeps the CPU busy, a browser tab plays video, a device has power saving off, or the power mode is Performance on battery.

**Find it in DeeCure:**

- **Power › Battery** shows battery use by app, like a phone's battery screen. This is an estimate, and DeeCure says so.
- It also shows the power draw in watts now, and the drain per hour by day (Expert view).
- **Processes › Top Consumers** shows the apps using the most CPU. In Expert, it also shows wake-ups per process.

**Fix it with DeeCure:**

1. Find the app in **Power › Battery**, then close it, or end it in **Processes**.
2. Open **Power › Power Profile** and use the Power saver mode on battery. In Expert, **Power controls › Power saver on battery** does this for you: Power saver turns on when you unplug, and your mode comes back when you plug in.
3. Open **Optimize**. It offers only what was measured on your laptop:
   - **Save power when you are away**, when the screen never turns off or the laptop never sleeps on battery.
   - **Turn off a service with no hardware**, such as ModemManager on a laptop without a mobile modem.
   - **Use Power saver on battery**, when the laptop ran on battery for an hour or more in the last 7 days. It also shows your real draw in watts and how long a full charge lasts.
   - **Turn off turbo on battery**, when the CPU often slowed down from heat on battery in the last 7 days.
   - **Power saving for a device**, when that device supports it and has it off.
   - **A different power mode**, when Performance runs on battery for a long time.
4. Lower the keyboard backlight in **Power › Power Profile › Power controls**.

**By hand:**

```bash
upower -i $(upower -e | grep -m1 BAT)   # charge, energy rate in watts, time left
sudo powertop                             # what uses power right now
```

**Limits:** DeeCure does not yet warn when the battery drains faster than your usual. You can compare the drain per hour by day yourself in Power › Battery.

## Laptop battery does not last like before: check battery health on Ubuntu

**Signs:** a full charge lasts less every month.

**Why it happens:** every battery loses capacity with charge cycles, heat, and long hours at 100%.

**Find it in DeeCure:**

- **Power › Battery** shows battery health in % of its design capacity, charge cycles, temperature, and a 30-day health trend.
- When the health falls, DeeCure warns: **"Battery has lost some of its capacity."**

**Fix it with DeeCure:**

1. Lost capacity cannot come back, but you can slow down further wear.
2. Open **Power › Power Profile › Power controls** (Expert view) and set a **charge limit**: Full 100%, Balanced 90%, Long life 80%, or your own start and stop values. It is kept after a restart.
3. **Optimize** suggests the 80% limit whenever the battery supports one and stops above 80%.
4. If the battery still charges past the limit, DeeCure warns "Battery charges past its limit". The firmware is not following the limit. On some laptops (Dell) the limit works only in the Custom charge mode. In any other mode the limit is not in use, and DeeCure shows it as "not in use" in Power controls. A limit set in DeeCure (from Optimize or Power controls) sets Custom by itself. A limit set in DeeCure is set again each time you plug in the charger. To pick another charge mode, use Power controls, not the firmware setup (BIOS): DeeCure sets a kept mode again.

**By hand:**

```bash
upower -i $(upower -e | grep -m1 BAT) | grep -E "energy-full|capacity"
cat /sys/class/power_supply/BAT0/charge_control_end_threshold   # the charge limit, when the laptop supports one
```

## Ubuntu laptop battery drains even when plugged in

**Signs:** the charger is connected, but the battery level still goes down.

**Why it happens:** the charger gives fewer watts than the laptop uses, often with a small USB-C phone charger.

**Find it in DeeCure:**

- DeeCure warns: **"The battery drains on the charger."**
- **Power › Battery** (Expert view) shows what a USB-C charger offers, in volts and amps, and what the laptop takes.

**Fix it with DeeCure:** DeeCure warns and shows the charger's watts. The fix is a stronger charger, made for your laptop.

**By hand:** `upower -d` shows whether the charger is online and the battery's energy rate.

## Laptop battery is too hot or reports a fault

**Signs:** the laptop is hot under the battery, or the battery stops charging.

**Find it in DeeCure:** DeeCure warns **"Battery is too hot"**, **"Battery reports a fault"**, or **"Battery ran empty"**, with the temperature and the battery's name.

**Fix it with DeeCure:** DeeCure warns and explains what to do. Let the laptop cool down on a hard surface, and check the battery with the maker when it reports a fault.

---

How a fix works, and how to undo it: see [Fix a problem](../using-deecure.md#3-fix-a-problem). Every change DeeCure makes is listed in **Activity**, with **Undo** where it can be undone.
