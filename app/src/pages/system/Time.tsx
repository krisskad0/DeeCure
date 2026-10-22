// SPDX-License-Identifier: Apache-2.0
// System › Time & Locale (docs/03-ui-design/05-pages/05-system.md), with real data.

import { useEffect, useState } from "react";
import { getInventory } from "../../api";
import { KeyValueList } from "../../components/KeyValueList";
import { Panel } from "../../components/Panel";
import { StatusBadge } from "../../components/StatusBadge";
import { useLive } from "../../useLive";
import { NoData } from "../NoData";

const loadOs = () => getInventory("os");

/** "IST, +0530" for a time zone. */
function zoneName(zone: string, at: Date): string {
  try {
    const short = new Intl.DateTimeFormat("en-GB", { timeZone: zone, timeZoneName: "short" }).formatToParts(at).find((p) => p.type === "timeZoneName")?.value;
    const offset = new Intl.DateTimeFormat("en-GB", { timeZone: zone, timeZoneName: "longOffset" }).formatToParts(at).find((p) => p.type === "timeZoneName")?.value;
    return [short, offset?.replace("GMT", "").replace(":", "") || "+0000"].filter(Boolean).join(", ");
  } catch {
    return "";
  }
}

/** Only a few countries use imperial units; the locale's territory tells. */
function units(locale?: string): string {
  const territory = locale?.match(/_([A-Z]{2})/)?.[1];
  return territory && ["US", "LR", "MM"].includes(territory) ? "Imperial · °F" : "Metric · °C";
}

export function SystemTime() {
  const { data: os, error } = useLive(loadOs, 60_000);
  const [now, setNow] = useState(new Date());
  useEffect(() => {
    const timer = setInterval(() => setNow(new Date()), 1000);
    return () => clearInterval(timer);
  }, []);
  if (!os) {
    return (
      <Panel title="Date and time" span={12}>
        <NoData error={error} waiting="Loading…" />
      </Panel>
    );
  }
  return (
    <>
      <Panel title="Date and time" source="/etc/timezone · /run/systemd/timesync" span={6} flush>
        <KeyValueList
          rows={[
            ["Local time", now.toLocaleString(undefined, { weekday: "short", day: "numeric", month: "short", year: "numeric", hour: "2-digit", minute: "2-digit", second: "2-digit" })],
            ["Time zone", os.timezone ? `${os.timezone} (${zoneName(os.timezone, now)})` : "—"],
            ["NTP synchronized", <StatusBadge key="n" variant={os.ntp_synchronized ? "ok" : "warn"} label={os.ntp_synchronized ? "Yes" : "No"} />],
            ["Time server", os.time_server],
            ["RTC in local time", os.rtc_local ? <StatusBadge key="r" variant="warn" label="Yes" /> : "No"],
          ]}
        />
      </Panel>
      <Panel title="Locale" source="/etc/default/locale · /etc/default/keyboard" span={6} flush>
        <KeyValueList
          rows={[
            ["System locale", os.locale ? <code key="l">{os.locale}</code> : "—"],
            ["Keyboard layout", os.keyboard ?? "—"],
            ["Units", units(os.locale)],
          ]}
        />
      </Panel>
    </>
  );
}
