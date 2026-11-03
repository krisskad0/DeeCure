// SPDX-License-Identifier: Apache-2.0
// Once after DeeCure was updated: "DeeCure was updated to X", its main change, and a link to
// everything that is new (Settings › About). The first start after installing shows nothing.

import { useEffect } from "react";
import { getUpdateStatus } from "../api";
import { AlertBanner } from "../components/AlertBanner";
import { Button } from "../components/Button";
import { noteGroups } from "../components/ReleaseNotes";
import { usePref } from "../prefs";
import { href } from "../route";
import { useLive } from "../useLive";

const load = () => getUpdateStatus();

export function UpdatedBanner() {
  const [seen, setSeen] = usePref<string>("seenVersion", "");
  const { data } = useLive(load, 600_000);
  // The first start remembers this version without a banner.
  useEffect(() => {
    if (data && seen === "") setSeen(data.current);
  }, [data, seen, setSeen]);
  if (!data || seen === "" || seen === data.current) return null;
  const first = data.current_notes ? noteGroups(data.current_notes)[0]?.points[0] : undefined;
  return (
    <AlertBanner
      tone="info"
      title={`DeeCure was updated to ${data.current}`}
      text={first ? `${first.replace(/[`*]/g, "")} See everything that is new in Settings › About.` : "See what is new in Settings › About."}
      actions={
        <>
          <Button
            small
            variant="primary"
            onClick={() => {
              setSeen(data.current);
              window.location.hash = href("settings", "about").replace(/^#/, "");
            }}
          >
            What is new
          </Button>
          <Button small onClick={() => setSeen(data.current)}>
            Dismiss
          </Button>
        </>
      }
    />
  );
}
