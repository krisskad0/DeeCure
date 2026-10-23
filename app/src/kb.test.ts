// SPDX-License-Identifier: Apache-2.0

import { describe, expect, it } from "vitest";
import type { Kb } from "./api";
import { kbError, kbPort, kbService, kbTopic } from "./kb";

const KB: Kb = {
  services: [{ name: "cups.service", plain: "Printing service." }],
  ports: [{ port: 22, proto: "tcp", name: "SSH", plain: "Remote sign-in.", risk_if_open: "high" }],
  errors: [
    { match: "EXT4-fs error", plain: "Disk damage.", severity: "critical", next_step: "Back up." },
    { match: "nvme.*timeout", plain: "Slow SSD.", severity: "critical", next_step: "Back up." },
    { match: "bad[regex", plain: "x", severity: "info", next_step: "x" },
  ],
  packages: [],
  topics: [{ id: "topics/storage/disk-full", title: "Free up disk space", summary: "s", steps: ["Empty the Trash."] }],
  rules: { "storage.root_almost_full": "topics/storage/disk-full" },
};

describe("knowledge base lookups", () => {
  it("finds entries by name, port, and rule", () => {
    expect(kbService(KB, "cups.service")?.plain).toBe("Printing service.");
    expect(kbPort(KB, 22, "tcp")?.name).toBe("SSH");
    expect(kbPort(KB, 22, "udp")).toBeUndefined();
    expect(kbTopic(KB, "storage.root_almost_full")?.title).toBe("Free up disk space");
    expect(kbTopic(KB, "other.rule")).toBeUndefined();
    expect(kbService(null, "cups.service")).toBeUndefined();
  });

  it("matches error text plainly or as a pattern, and survives bad patterns", () => {
    expect(kbError(KB, "EXT4-fs error (device dm-1): bad block")?.plain).toBe("Disk damage.");
    expect(kbError(KB, "nvme nvme0: I/O 12 QID 3 timeout, aborting")?.plain).toBe("Slow SSD.");
    expect(kbError(KB, "all good")).toBeUndefined();
  });
});
