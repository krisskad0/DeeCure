// SPDX-License-Identifier: Apache-2.0
// What DeeCure changed, with Undo where possible: Activity › DeeCure Actions and
// Backups › Restore Points.

import { useState } from "react";
import { type FixRun, getFixRuns, toAppError, undoFix } from "../api";
import { Button } from "../components/Button";
import { type Column, DataTable } from "../components/DataTable";
import { StatusBadge, type StatusVariant } from "../components/StatusBadge";
import { when } from "../metrics";
import { NoData } from "../pages/NoData";
import { useLive } from "../useLive";
import { useToast } from "./Toast";

const loadRuns = () => Promise.resolve().then(() => getFixRuns());
const STATUS: Record<FixRun["status"], [StatusVariant, string]> = { succeeded: ["ok", "Succeeded"], failed: ["crit", "Failed"], undone: ["off", "Undone"] };

/** True when the run can still be taken back. */
export const undoable = (r: FixRun) => r.status !== "undone" && r.steps.some((s) => s.ok && s.undo);

export function FixHistory({ restorePoints = false }: { restorePoints?: boolean }) {
  const { data, error } = useLive(loadRuns, 15_000);
  const [busy, setBusy] = useState<number | null>(null);
  const [extra, setExtra] = useState<Record<number, FixRun["status"]>>({});
  const toast = useToast();
  if (!data) return <NoData error={error} waiting="Loading…" />;
  const runs = data.map((r) => (extra[r.id] ? { ...r, status: extra[r.id] } : r)).filter((r) => !restorePoints || r.steps.some((s) => s.undo));
  const undo = async (r: FixRun) => {
    setBusy(r.id);
    try {
      await undoFix(r.id);
      setExtra((e) => ({ ...e, [r.id]: "undone" }));
      toast(`Undone · ${r.title}`);
    } catch (e) {
      toast(`Could not undo: ${toAppError(e).message}`);
    }
    setBusy(null);
  };
  const columns: Column<FixRun>[] = [
    { key: "time", label: "Time", text: (r) => String(r.started_at), render: (r) => when(r.started_at) },
    { key: "action", label: "Action", text: (r) => r.title },
    { key: "target", label: "Target", text: (r) => r.subject ?? r.steps.map((s) => s.title).join(", "), render: (r) => (r.subject ? <code>{r.subject}</code> : r.steps.map((s) => s.title).join(", ")), wrap: true },
    { key: "result", label: "Result", text: (r) => STATUS[r.status][1], render: (r) => <StatusBadge variant={STATUS[r.status][0]} label={STATUS[r.status][1]} /> },
    {
      key: "undo",
      label: "Undo",
      text: () => "",
      render: (r) => (undoable(r) ? <Button small disabled={busy !== null} onClick={() => void undo(r)}>{busy === r.id ? "Undoing…" : "Undo"}</Button> : <span className="muted">{r.status === "undone" ? "Done" : "Not reversible"}</span>),
    },
  ];
  return (
    <DataTable
      rows={runs}
      columns={columns}
      rowKey={(r) => String(r.id)}
      pageSize={12}
      empty={restorePoints ? "No restore points yet. Reversible fixes save one here." : "DeeCure has not changed anything on this computer yet."}
    />
  );
}
