// SPDX-License-Identifier: Apache-2.0
// Renders the Network, Accounts, Security, and Privacy pages with realistic answers.

import { cleanup, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { AccountsGroups, AccountsLogins, AccountsSessions, AccountsSsh, AccountsUsers } from "../accounts/Accounts";
import { NetworkConnections, NetworkDns, NetworkFirewall, NetworkInterfaces, NetworkPorts, NetworkTraffic } from "../network/Network";
import { PrivacyAccess, PrivacyPermissions, PrivacySettings } from "../privacy/Privacy";
import { SecurityHardening, SecurityIntegrity, SecurityMalware, SecurityOverview, SecurityVulns } from "./Security";

const now = Date.now();
const DATA: Record<string, unknown> = {
  net_details: {
    links: [{ name: "wlp0s20f3", ipv4: ["192.168.1.24/24"], ipv6: [], rx: 4e9, tx: 6e8 }],
    gateway: "192.168.1.1",
    wifi: { interface: "wlp0s20f3", ssid: "HomeNet-5G", strength: 78, dbm: -52, frequency_mhz: 5180, bitrate_mbps: 866 },
    connectivity: "full",
    resolver: "systemd-resolved",
    dns_servers: ["192.168.1.1"],
    vpns: [{ name: "Office", kind: "wireguard", connected: false }],
    firewall: { installed: true, enabled: false, incoming: "DROP", outgoing: "ACCEPT" },
  },
  network: { interfaces: [{ name: "wlp0s20f3", kind: "Wi-Fi", state: "up" }], connections: { tcp_established: 1, tcp_listening: 2, udp: 1 } },
  sockets: [
    { proto: "tcp", local: "0.0.0.0:5432", local_port: 5432, remote: "*", state: "LISTEN", exposure: "all" },
    { proto: "tcp", local: "127.0.0.1:631", local_port: 631, remote: "*", state: "LISTEN", exposure: "local" },
    { proto: "tcp", local: "192.168.1.24:51432", local_port: 51432, remote: "142.250.183.14:443", state: "ESTAB", process: "chrome", exposure: "network" },
  ],
  accounts: {
    users: [{ name: "riya", full_name: "Riya", uid: 1000, groups: ["riya", "sudo", "docker"], admin: true, shell: "/bin/bash", home: "/home/riya", last_sign_in: now - 3_600_000 }],
    groups: [{ name: "docker", gid: 999, members: ["riya"], privilege: "Root-equivalent (Docker)" }],
    sessions: [{ id: "2", user: "riya", seat: "seat0", kind: "wayland", class: "user", state: "active", remote: false, since: now - 3_600_000 }],
    events: [{ at: now - 3_600_000, user: "riya", event: "sign-in", source: "tty2", success: true }, { at: now - 1_800_000, user: "riya", event: "sudo", source: "/usr/bin/apt update", success: false }],
    ssh: { server_installed: true, password_login: true, root_login: "prohibit-password", authorized_keys: 1, client_keys: [{ file: "id_ed25519", kind: "ED25519", bits: 256, fingerprint: "SHA256:k3FQw8abcdefghijklmnopqrstuv", created: now }] },
  },
  security: {
    hardening: [{ name: "Address space randomization (ASLR)", value: "kernel.randomize_va_space = 2", result: "pass", recommendation: "" }, { name: "Kernel pointers hidden", value: "kernel.kptr_restrict = 0", result: "fail", recommendation: "Set kernel.kptr_restrict = 1" }],
    apparmor: true,
    suid: [{ path: "/usr/bin/sudo", owner: "root", package: "sudo" }, { path: "/opt/tool/helper", owner: "root" }],
    scanners: ["clamav"],
    clamav_updated: now - 30 * 86_400_000,
    heuristics: [{ title: "No startup apps run from temporary or download folders", passed: false, detail: "Check: App" }],
    screen_lock: true,
  },
  privacy: {
    in_use: { microphone: ["Zoom"], camera: [], screen: [] },
    apps: [{ app: "firefox", kind: "snap", camera: true, microphone: true, location: false, files: true, network: true }],
    settings: [{ label: "Location services", description: "Apps can ask where you are", on: false }],
  },
  hardware: { machine: { secure_boot: true, boot_mode: "UEFI" } },
  storage: { encrypted: false, candidates: [] },
  os: { livepatch: "disabled" },
};

vi.mock("../../api", () => ({
  getInventory: (kind: string) => Promise.resolve(DATA[kind]),
  getSnapshot: () => Promise.resolve({ data: { pending: [{ name: "openssl", installed: "1", candidate: "2", origin: "noble-security", security: true }], automatic_security: true } }),
  getMetrics: () => Promise.resolve([]),
  toAppError: (e: unknown) => ({ kind: "other", message: String(e) }),
}));

afterEach(cleanup);

async function shows(Tab: () => React.JSX.Element, text: string | RegExp) {
  render(<Tab />);
  await waitFor(() => expect(screen.getAllByText(text).length).toBeGreaterThan(0));
  cleanup();
}

describe("Network", () => {
  it("every tab renders real values", async () => {
    await shows(NetworkInterfaces, "HomeNet-5G");
    await shows(NetworkConnections, "chrome");
    await shows(NetworkPorts, "Not filtered");
    await shows(NetworkFirewall, "The firewall is off");
    await shows(NetworkDns, "Office");
    await shows(NetworkTraffic, "Throughput");
  });
});

describe("Accounts", () => {
  it("every tab renders real values", async () => {
    await shows(AccountsUsers, "Administrator");
    await shows(AccountsGroups, "Root-equivalent (Docker)");
    await shows(AccountsSessions, "seat0");
    await shows(AccountsLogins, "1 failed sudo attempts in 30 days.");
    await shows(AccountsSsh, "SSH allows password sign-in");
  });
});

describe("Security", () => {
  it("overview counts checks and flags what to fix", async () => {
    await shows(SecurityOverview, "1 security updates waiting");
  });

  it("other tabs render", async () => {
    await shows(SecurityVulns, "openssl");
    await shows(SecurityHardening, "Could be safer");
    await shows(SecurityIntegrity, "No package");
    await shows(SecurityMalware, /Old signatures miss new malware/);
  });
});

describe("Privacy", () => {
  it("shows devices in use, permissions, and settings", async () => {
    await shows(PrivacyAccess, "In use by Zoom");
    await shows(PrivacyPermissions, "firefox");
    await shows(PrivacySettings, "Location services");
  });
});
