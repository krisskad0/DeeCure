// SPDX-License-Identifier: Apache-2.0
// Sudo rules and SSH server settings, with answers shaped like the helper's.

import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { SshServerView, SudoRulesView, sudoMeaning } from "./AdminAccounts";

afterEach(cleanup);

const rule = (who: string, no_password = false, all_commands = false) => ({ file: "/etc/sudoers", line: `${who} ALL=(ALL) ALL`, who, no_password, all_commands });

describe("sudo rules", () => {
  it("explains each rule", () => {
    expect(sudoMeaning(rule("Defaults")).text).toBe("Setting");
    expect(sudoMeaning(rule("@includedir")).text).toBe("Reads more files");
    expect(sudoMeaning(rule("ci", true, true))).toEqual({ text: "Everything, no password", variant: "warn" });
    expect(sudoMeaning(rule("%sudo", false, true)).text).toBe("Everything, with password");
  });

  it("counts admin rules and names the risky ones", () => {
    render(<SudoRulesView data={[rule("Defaults"), rule("%sudo", false, true), rule("ci", true, true)]} />);
    expect(screen.getByText("2")).toBeTruthy();
    expect(screen.getByText("1: ci")).toBeTruthy();
  });
});

describe("SSH server", () => {
  it("lists weak settings and the real values", () => {
    render(<SshServerView data={{ installed: true, settings: { permitrootlogin: "yes", passwordauthentication: "yes" }, weak: [{ level: "warning", text: "root can sign in with a password" }] }} />);
    expect(screen.getByText("root can sign in with a password")).toBeTruthy();
    expect(screen.getByText("permitrootlogin")).toBeTruthy();
  });

  it("says when nothing is weak, or no server is installed", () => {
    render(<SshServerView data={{ installed: true, settings: {}, weak: [] }} />);
    expect(screen.getByText("No weak settings found")).toBeTruthy();
    cleanup();
    render(<SshServerView data={{ installed: false }} />);
    expect(screen.getByText(/not installed/)).toBeTruthy();
  });
});
