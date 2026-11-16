// SPDX-License-Identifier: Apache-2.0
// Security › Firewall (docs/03-ui-design/05-pages/18-security.md), with real data.
// It was Network › Firewall until 2026-11-16; Security › Overview checks "Firewall enabled".

import { getInventory } from "../../api";
import { AdminRead } from "../../components/AdminRead";
import { AlertBanner } from "../../components/AlertBanner";
import { Button } from "../../components/Button";
import { LogLine } from "../../components/CodeBlock";
import { KpiRow } from "../../components/KpiTile";
import { Panel } from "../../components/Panel";
import { useFix } from "../../shell/FixDrawer";
import { useLive } from "../../useLive";
import { NoData } from "../NoData";

const loadDetails = () => getInventory("net_details");

export function SecurityFirewall() {
  const { data: d, error } = useLive(loadDetails, 30_000);
  const openFix = useFix();
  if (!d) {
    return (
      <Panel title="Firewall" span={12}>
        <NoData error={error} waiting="Loading…" />
      </Panel>
    );
  }
  const fw = d.firewall;
  const policy = (p?: string) => (p === "DROP" || p === "REJECT" ? "Deny" : p === "ACCEPT" ? "Allow" : (p ?? "-"));
  return (
    <>
      <KpiRow
        tiles={[
          { label: "Status", value: !fw.installed ? "Not installed" : fw.enabled ? "Active" : "Off", status: fw.installed && !fw.enabled ? "warn" : "ok", subtext: "ufw" },
          { label: "Incoming", value: fw.enabled ? policy(fw.incoming) : "Allow", subtext: fw.enabled ? "default" : "firewall off" },
          { label: "Outgoing", value: fw.enabled ? policy(fw.outgoing) : "Allow", subtext: "default" },
          { label: "Rules", value: fw.rules ? String(fw.rules.length) : "-", subtext: fw.rules ? "" : "need Full access to read" },
        ]}
      />
      <Panel title="Rules" source="/etc/ufw/user.rules" span={12}>
        {fw.rules ? (
          fw.rules.length ? <ul className="plain-list">{fw.rules.map((r) => <li key={r}><code>{r}</code></li>)}</ul> : <div className="empty muted">No rules.</div>
        ) : (
          <AdminRead what="firewall_rules" label="Show the rules (asks for your password)">
            {(text) => <LogLine>{text}</LogLine>}
          </AdminRead>
        )}
      </Panel>
      {fw.installed && !fw.enabled && (
        <AlertBanner
          tone="warn"
          title="The firewall is off"
          text="Any program that listens on every network can be reached by other computers on the same network, such as public Wi-Fi."
          actions={<Button small variant="primary" onClick={() => openFix("security.enable_firewall")}>Turn on…</Button>}
        />
      )}
    </>
  );
}
