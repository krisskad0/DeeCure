// SPDX-License-Identifier: Apache-2.0
// Command palette (docs/03-ui-design/02-shell/overlays.md): Ctrl/Cmd+K, search everything, jump.
// Results: every page and sub-page, and the services the agent knows. Packages and jobs
// join when their collectors exist. Beginner lists only its own pages (menuPages in
// visible.ts) and no services, and says that Advanced finds more.

import { useEffect, useMemo, useRef, useState } from "react";
import { type UnitState, getSnapshot } from "../api";
import { Icon } from "../components/Icon";
import { href } from "../route";
import type { PageSpec } from "../spec/pages";
import { atLeast, useUi } from "../ui";
import { menuPages } from "../visible";

interface Entry {
  label: string;
  kind: "Page" | "Service" | "Dev";
  target: string;
}

const MAX_RESULTS = 40;

function pageEntries(pages: PageSpec[]): Entry[] {
  return pages.flatMap((page) =>
    page.tabs.length === 1
      ? [{ label: page.label, kind: "Page" as const, target: href(page.id, page.tabs[0].id) }]
      : page.tabs.map((tab) => ({ label: `${page.label} › ${tab.label}`, kind: "Page" as const, target: href(page.id, tab.id) })),
  );
}

const DEV_ENTRIES: Entry[] = import.meta.env.DEV ? [{ label: "Component gallery", kind: "Dev", target: "#/gallery" }] : [];

export function CommandPalette({ onClose }: { onClose: () => void }) {
  const { mode } = useUi();
  const full = atLeast(mode, "advanced");
  const [query, setQuery] = useState("");
  const [services, setServices] = useState<Entry[]>([]);
  const [highlight, setHighlight] = useState(0);
  const input = useRef<HTMLInputElement>(null);
  const list = useRef<HTMLUListElement>(null);

  useEffect(() => {
    input.current?.focus();
    if (!full) {
      setServices([]);
      return;
    }
    let alive = true;
    getSnapshot<UnitState[]>("services").then(
      (snapshot) =>
        alive &&
        setServices(
          (snapshot?.data ?? []).map((unit) => ({ label: unit.name, kind: "Service", target: href("services", "system") })),
        ),
      () => alive && setServices([]),
    );
    return () => {
      alive = false;
    };
  }, [full]);

  const pages = useMemo(() => pageEntries(menuPages(mode)), [mode]);
  const results = useMemo(() => {
    const needle = query.trim().toLowerCase();
    return [...DEV_ENTRIES, ...pages, ...services].filter((e) => e.label.toLowerCase().includes(needle)).slice(0, MAX_RESULTS);
  }, [query, pages, services]);

  useEffect(() => setHighlight(0), [query]);
  useEffect(() => {
    list.current?.children[highlight]?.scrollIntoView({ block: "nearest" });
  }, [highlight]);

  const open = (entry: Entry | undefined) => {
    if (!entry) return;
    window.location.hash = entry.target;
    onClose();
  };

  const onKey = (event: React.KeyboardEvent) => {
    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      const step = event.key === "ArrowDown" ? 1 : -1;
      setHighlight((h) => (results.length === 0 ? 0 : (h + step + results.length) % results.length));
    } else if (event.key === "Enter") {
      open(results[highlight]);
    } else if (event.key === "Escape") {
      onClose();
    }
  };

  return (
    <>
      <div className="scrim scrim-palette" onClick={onClose} />
      <div className="palette" role="dialog" aria-label="Search">
        <div className="palette-input">
          <Icon name="search" size={18} />
          <input
            ref={input}
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={onKey}
            placeholder="Search pages and services…"
            aria-label="Search pages and services"
          />
          <kbd>Esc</kbd>
        </div>
        <ul className="palette-results" ref={list} role="listbox">
          {results.length === 0 && <li className="palette-empty muted">{full ? "No matches" : "No matches. Advanced shows every page and service."}</li>}
          {results.map((entry, i) => (
            <li key={`${entry.kind}|${entry.label}`} role="option" aria-selected={i === highlight}>
              <button
                type="button"
                className={i === highlight ? "highlight" : ""}
                onMouseEnter={() => setHighlight(i)}
                onClick={() => open(entry)}
              >
                <span>{entry.label}</span>
                <span className="kind">{entry.kind}</span>
              </button>
            </li>
          ))}
        </ul>
      </div>
    </>
  );
}
