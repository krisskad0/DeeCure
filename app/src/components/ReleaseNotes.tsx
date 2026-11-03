// SPDX-License-Identifier: Apache-2.0
// A release's notes (its CHANGELOG.md section, from the GitHub release) as a short list:
// "### Added" and the like become small headings, "- " lines become points, and `code` and
// **bold** keep their look. The "How to update" part at the end is left out: the app has the
// Install button for that.

import type { ReactNode } from "react";

const HEADINGS: Record<string, string> = { Added: "New", Changed: "Changed", Fixed: "Fixed", Removed: "Removed", Security: "Security" };

/** Inline Markdown: `code`, **bold**, and [text](link) as plain text. */
function inline(text: string): ReactNode[] {
  const parts = text.replace(/\[([^\]]+)\]\([^)]+\)/g, "$1").split(/(`[^`]+`|\*\*[^*]+\*\*)/g);
  return parts.map((p, i) => {
    if (p.startsWith("`") && p.endsWith("`") && p.length > 1) return <code key={i}>{p.slice(1, -1)}</code>;
    if (p.startsWith("**") && p.endsWith("**") && p.length > 3) return <b key={i}>{p.slice(2, -2)}</b>;
    return p;
  });
}

export interface NoteGroup {
  title: string;
  points: string[];
}

/** The notes as groups of points, without "How to update" and what follows. */
export function noteGroups(notes: string): NoteGroup[] {
  const groups: NoteGroup[] = [];
  for (const raw of notes.split("\n")) {
    const line = raw.trim();
    const heading = line.match(/^#{2,4}\s+(.*)$/);
    if (heading) {
      if (/how to update/i.test(heading[1])) break;
      groups.push({ title: HEADINGS[heading[1]] ?? heading[1], points: [] });
      continue;
    }
    const point = line.match(/^[-*]\s+(.*)$/);
    if (point) {
      if (groups.length === 0) groups.push({ title: "", points: [] });
      groups[groups.length - 1].points.push(point[1]);
    }
  }
  return groups.filter((g) => g.points.length > 0);
}

/** The notes as a list; `max` shows only the first points, with the count of the rest. */
export function ReleaseNotes({ notes, max }: { notes: string; max?: number }) {
  const groups = noteGroups(notes);
  if (groups.length === 0) return <p className="muted">This release has no notes.</p>;
  let left = max ?? Number.POSITIVE_INFINITY;
  const total = groups.reduce((n, g) => n + g.points.length, 0);
  return (
    <div className="release-notes">
      {groups.map((g) => {
        if (left <= 0) return null;
        const points = g.points.slice(0, left);
        left -= points.length;
        return (
          <div key={g.title} className="rn-group">
            {g.title && <div className="kn-head">{g.title}</div>}
            <ul>
              {points.map((p) => (
                <li key={p}>{inline(p)}</li>
              ))}
            </ul>
          </div>
        );
      })}
      {max !== undefined && total > max && <p className="muted">and {total - max} more</p>}
    </div>
  );
}
