// SPDX-License-Identifier: Apache-2.0
// The access level (docs/01-product/user-journey.md#first-launch-3-short-screens).
// Basic: DeeCure never asks for the password; admin read-outs and admin fix steps are off.
// Full: they are on, and polkit asks for the password each time one is used. Full needs
// the admin helper to be installed; without it the level is Basic, whatever was saved.

import { useHelperInstalled } from "./components/AdminRead";
import { usePref } from "./prefs";

export type AccessLevel = "basic" | "full";

export interface Access {
  /** What the person chose. */
  chosen: AccessLevel;
  /** True when admin actions may be used now: Full chosen and the helper installed. */
  full: boolean;
  /** `null` while checking. */
  helper: boolean | null;
  set: (level: AccessLevel) => void;
}

export function useAccess(): Access {
  const [chosen, set] = usePref<AccessLevel>("access", "basic");
  const helper = useHelperInstalled();
  return { chosen, full: chosen === "full" && helper === true, helper, set };
}

/** The two choices, with Full turned off (and saying why) when the helper is missing. */
export function accessOptions(helper: boolean | null) {
  return [
    { value: "basic" as const, title: "Basic", description: "No password · everything your account can read · fixes that need no admin rights" },
    {
      value: "full" as const,
      title: "Full",
      description:
        helper === false
          ? "Needs the DeeCure admin helper, which comes with the deecure package from DeeCure's repository"
          : "Admin helper · disk health, firewall rules, all users' jobs, admin fixes · your password is asked each time",
      disabled: helper !== true,
    },
  ];
}
