// SPDX-License-Identifier: Apache-2.0
// Expert network sections: Wi-Fi verdicts, TCP health, DNS and routes, and TCP traffic per app.

import { act, cleanup, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { NetDeep, TcpTraffic } from "../../api";
import { REFRESH_EVENT } from "../../units";
import { DnsRoutesExpert, InterfacesExpert, TrafficExpert, perApp, signalVerdict } from "./NetworkExpert";

let step = 0;
let resolved = true;
const deep = (): NetDeep => ({
  at: 1_000_000 + step * 10_000,
  wifi: [{ interface: "wlp45s0", ssid: "Home", channel: 36, frequency_mhz: 5180, band: "5 GHz", rate_mbps: 270, signal_dbm: -72, power_save: true, security: "WPA2" }],
  interfaces: [{ name: "wlp45s0", rx_errors: 0, tx_errors: 0, rx_dropped: 10, tx_dropped: 111, collisions: 0, carrier_changes: 30 }],
  counters: { "Tcp.RetransSegs": 100 + step * 30, "Tcp.OutSegs": 10_000 + step * 1000, "Tcp.AttemptFails": 157, "Tcp.EstabResets": 1337, "Udp.InErrors": 1470, "Udp.RcvbufErrors": 1470 },
  dns: resolved ? [{ link: "wlp45s0", current: "192.168.1.1", servers: ["192.168.1.1"], default_route: true, dnssec: "no/unsupported", dns_over_tls: false }] : undefined,
  routes: [{ destination: "default", gateway: "192.168.1.1", device: "wlp45s0", metric: 600, down: false, family: 4 }],
  extra_rules: 2,
});
const tcp = (): TcpTraffic => ({
  at: 1_000_000 + step * 10_000,
  connections: [
    { local: "a:1", remote: "b:443", program: "chrome", pid: 5, received: 1000 + step * 100_000, sent: 500, resent: 0 },
    { local: "a:2", remote: "c:22", received: 50, sent: 50, resent: 0 },
  ],
});

vi.mock("../../api", () => ({
  getInventory: (kind: string) => Promise.resolve(kind === "net_deep" ? deep() : tcp()),
  toAppError: (e: unknown) => ({ kind: "other", message: String(e) }),
}));

afterEach(() => {
  cleanup();
  step = 0;
  resolved = true;
});

async function next() {
  step = 1;
  await act(async () => {
    window.dispatchEvent(new Event(REFRESH_EVENT));
  });
}

describe("Expert network sections", () => {
  it("rates a Wi-Fi signal plainly", () => {
    expect(signalVerdict(-55)?.text).toBe("Good");
    expect(signalVerdict(-72)?.variant).toBe("warn");
    expect(signalVerdict(-80)?.text).toBe("Weak");
    expect(signalVerdict(undefined)).toBeUndefined();
  });

  it("Wi-Fi link, TCP resends now, and UDP drops by slow programs", async () => {
    render(<InterfacesExpert />);
    await waitFor(() => expect(screen.getByText("Fair: calls may stutter")).toBeTruthy());
    expect(screen.getByText("On (saves battery, can add delay)")).toBeTruthy();
    expect(screen.getByText(/1,470 because a program read too slowly/)).toBeTruthy();
    expect(screen.getByText("1.00")).toBeTruthy();
    await next();
    await waitFor(() => expect(screen.getByText("3.00")).toBeTruthy());
  });

  it("DNS, routes, VPN rules, and no systemd-resolved", async () => {
    render(<DnsRoutesExpert />);
    await waitFor(() => expect(screen.getByText("internet through wlp45s0")).toBeTruthy());
    expect(screen.getByText(/2 extra routing rules/)).toBeTruthy();
    expect(screen.getByText("everything (IPv4)")).toBeTruthy();
    cleanup();
    resolved = false;
    render(<DnsRoutesExpert />);
    await waitFor(() => expect(screen.getByText(/systemd-resolved is not running/)).toBeTruthy());
  });

  it("TCP traffic per app: rates from two readings, others grouped", async () => {
    const before = { ...tcp() };
    step = 1;
    const apps = perApp(tcp(), before);
    expect(apps[0].program).toBe("chrome");
    expect(apps[0].downNow).toBe(10_000);
    expect(apps.find((a) => a.program === "Other users' programs")?.connections).toBe(1);
    step = 0;
    render(<TrafficExpert />);
    await waitFor(() => expect(screen.getByText("chrome")).toBeTruthy());
    expect(screen.getByText("TCP only")).toBeTruthy();
  });
});
