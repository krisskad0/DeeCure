# Heat, fan noise, and CPU throttling on Ubuntu

[Docs](../../README.md) › [User guide](../README.md) › [Problems and fixes](README.md)

A loud fan, a hot laptop, a CPU that slows itself down, and a computer that switches off from heat.

DeeCure compares each temperature with that part's **own** limits (the temperatures where it slows down and where it shuts down), not with one number for every computer.

## Ubuntu laptop fan is loud even when idle

**Signs:** the fan runs fast while you only read or type, and the laptop is hot.

**Why it happens:** a background task keeps the CPU busy (a file indexer, a browser tab, an update), the power mode is set to Performance, or dust blocks the air vents.

**Find it in DeeCure:**

- **Hardware › Sensors** shows each temperature against its own limit, the fan speeds, and heat slowdowns over 24 hours.
- **Processes › Top Consumers** shows which process keeps the CPU busy.
- DeeCure warns **"The processor is very hot"** or that a part is too hot, with the temperature.

**Fix it with DeeCure:**

1. End the busy process in **Processes**, when it is not needed.
2. In **Power › Power Profile**, use Balanced or Power saver.
3. In **Power › Power Profile › Power controls** (Expert view), turn turbo off. You can set it separately on battery and on mains.

**By hand:**

```bash
sensors                 # from the lm-sensors package
top -o %CPU
```

## CPU throttling on Ubuntu: the computer slows down under load

**Signs:** a build, a game, or a video export starts fast and then slows down. The laptop is slower on battery.

**Why it happens:** the CPU slows itself down when it is too hot, or when it reaches its power limit.

**Find it in DeeCure:**

- **Hardware › Sensors** shows heat slowdowns on the same chart as the temperature.
- In Expert, **Hardware › Processor** shows how many times and how long each core was slowed by heat, the CPU power in watts, and the power limits.

**Fix it with DeeCure:**

1. **Optimize** offers **Turn off turbo on battery** when the CPU slowed down from heat on battery in the last 7 days.
2. In **Power › Power Profile › Power controls** (Expert view), you can set the highest CPU speed on battery.
3. Clean the air vents if the slowdowns happen on mains too.

**By hand:** `cat /sys/devices/system/cpu/cpu*/thermal_throttle/core_throttle_count` (Intel).

## Ubuntu laptop fan stopped, or the cooling cannot keep up

**Signs:** the laptop gets very hot and stays hot, or you do not hear the fan at all.

**Find it in DeeCure:**

- DeeCure warns **"A fan stopped while the processor is hot"** or **"The cooling cannot keep up."**
- **Hardware › Sensors** shows each fan's speed and the cooling devices and their level.

**Fix it with DeeCure:** **Optimize** shows **Clean the fans, or check the cooling** when the cooling is not keeping up. This one is done by hand: clean the vents with air, or have the fan checked.

## Ubuntu computer switched off by itself from heat

**Signs:** the computer turned off with no warning, often during heavy work.

**Find it in DeeCure:**

- DeeCure warns **"The computer switched off at a critical temperature."**
- **Logs › Errors and crashes** shows the event and when it happened.

**Fix it with DeeCure:** DeeCure warns and explains. Check the fan and vents, and use the heat steps above.

**By hand:** `journalctl -b -1 -k | grep -i -E "critical|thermal"`

---

How a fix works, and how to undo it: see [Fix a problem](../using-deecure.md#3-fix-a-problem). Every change DeeCure makes is listed in **Activity**, with **Undo** where it can be undone.
