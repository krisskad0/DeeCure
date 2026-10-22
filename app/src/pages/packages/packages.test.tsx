// SPDX-License-Identifier: Apache-2.0
// Renders the Packages, Containers, and Backups pages with realistic answers and gaps.

import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { BackupsJobs, BackupsRestore, BackupsSnapshots } from "../backups/Backups";
import { ContainersImages, ContainersList, ContainersVms, ContainersVolumes, dockerBytes } from "../containers/Containers";
import { PackagesCleanup, PackagesInstalled, PackagesSources } from "./Packages";

const now = Date.now();
let docker = "running";
let dejaDup = { installed: false, scheduled: false };

vi.mock("../../api", () => ({
  getInventory: (kind: string) =>
    Promise.resolve(
      {
        packages: {
          packages: [
            { name: "htop", version: "3.3.0", source: "apt", size: 300_000, manual: true, summary: "interactive viewer", section: "utils" },
            { name: "libfoo", version: "1", source: "apt", size: 10_000, manual: false },
            { name: "firefox", version: "131.0.3", source: "snap", size: 286e6, manual: true },
          ],
          sources: [{ line: "http://archive.ubuntu.com/ubuntu noble main", kind: "Official", signed: true, own_key: true, enabled: true, file: "ubuntu.sources" }, { line: "http://example.com/repo ./", kind: "Third-party", signed: false, own_key: false, enabled: true, file: "x.list" }],
          removable: [{ action: "packages.autoremove", title: "Packages no longer needed", count: 48, size: 310e6, names: ["linux-headers-6.8.0-45"], undo: "reinstallable" }],
        },
        containers: {
          docker,
          containers: [
            { name: "pg-dev", image: "postgres:16", status: "Up 12 hours", state: "running", ports: ["0.0.0.0:5432->5432/tcp", ":::5432->5432/tcp"], cpu: "0.20%", memory: "142MiB" },
            { name: "old", image: "x", status: "Exited (0) 2 days ago", state: "exited", ports: [] },
          ],
          images: [{ repository: "postgres", tag: "16", size: "432MB", created: "3 weeks ago", containers: 1 }, { repository: "node", tag: "20", size: "1.1GB", created: "2 months ago", containers: 0 }],
          volumes: [{ name: "pgdata", size: "980MB", links: 1 }, { name: "orphan", size: "12MB", links: 0 }],
          vms: [],
        },
        backups: { deja_dup: dejaDup, timeshift: { installed: true, configured: true, mode: "rsync", schedule: ["Monthly"], snapshots: [{ created: now - 40 * 86_400_000, kind: "Monthly", comment: "before upgrade" }] } },
      }[kind],
    ),
  toAppError: (e: unknown) => ({ kind: "other", message: String(e) }),
}));

beforeEach(() => {
  docker = "running";
  dejaDup = { installed: false, scheduled: false };
});
afterEach(cleanup);

async function shows(Tab: () => React.JSX.Element, text: string | RegExp) {
  render(<Tab />);
  await waitFor(() => expect(screen.getAllByText(text).length).toBeGreaterThan(0));
  cleanup();
}

describe("Packages", () => {
  it("lists packages from every source and inspects one", async () => {
    render(<PackagesInstalled />);
    await waitFor(() => expect(screen.getByText("1 installed by you")).toBeTruthy());
    fireEvent.click(screen.getByText("htop"));
    expect(screen.getByRole("dialog", { name: "htop" })).toBeTruthy();
    expect(screen.getByText("utils")).toBeTruthy();
  });

  it("flags unchecked sources and lists cleanup from a simulation", async () => {
    await shows(PackagesSources, "1 source skips signature checks");
    await shows(PackagesCleanup, "Packages no longer needed");
  });
});

describe("Containers", () => {
  it("reads docker sizes", () => {
    expect(dockerBytes("3.77GB")).toBe(3.77e9);
    expect(dockerBytes("12MB")).toBe(12e6);
    expect(dockerBytes("n/a")).toBe(0);
  });

  it("warns about ports open to every network, and never offers volumes for deletion", async () => {
    await shows(ContainersList, "pg-dev is reachable from every network");
    await shows(ContainersImages, "Unused");
    await shows(ContainersVolumes, /never offers to delete them/);
    await shows(ContainersVms, /No virtual machines/);
  });

  it("is honest when Docker is not running", async () => {
    docker = "not running";
    await shows(ContainersList, /DeeCure does not start it/);
  });
});

describe("Backups", () => {
  it("shows the last backup age and warns about personal files", async () => {
    await shows(BackupsJobs, "Personal files have no backup");
    await shows(BackupsSnapshots, "before upgrade");
    await shows(BackupsRestore, /nothing to restore/);
  });
});
