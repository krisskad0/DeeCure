// SPDX-License-Identifier: Apache-2.0
// Assistant › Chat (docs/03-ui-design/05-pages/21-assistant.md). The assistant is planned for
// Phase 4; until then this page says what it will do, and links to the pages that answer
// the suggested questions today.

import { LinkButton } from "../../components/Button";
import { KeyValueList } from "../../components/KeyValueList";
import { Panel } from "../../components/Panel";
import { StatusBadge } from "../../components/StatusBadge";
import { href } from "../../route";

const QUESTIONS: [string, string, string][] = [
  ["Why did my computer restart last night?", "logs", "boots"],
  ["Why is a program crashing?", "logs", "crashes"],
  ["Is my disk healthy?", "hardware", "storage"],
  ["What is filling my disk?", "storage", "usage"],
  ["Why is my computer slow?", "diagnostics", "performance"],
];

export function AssistantChat() {
  return (
    <>
      <Panel title="Ask DeeCure" subtitle="planned: local model, on this computer" span={8}>
        <div className="empty">
          <StatusBadge variant="off" label="Coming in a later version" />
          <p>The assistant will answer questions about this computer in plain words, using what DeeCure already collected. It will run on this computer, only read, and suggest fixes without running them.</p>
          <p className="muted">Until then, the pages on the right answer these questions directly.</p>
        </div>
      </Panel>
      <Panel title="Suggested questions" span={4} flush>
        <ul className="plain-list question-list">
          {QUESTIONS.map(([q, page, tab]) => (
            <li key={q}>
              <span>{q}</span>
              <LinkButton to={href(page, tab)}>Open</LinkButton>
            </li>
          ))}
        </ul>
      </Panel>
      <Panel title="Assistant" span={4} flush>
        <KeyValueList
          rows={[
            ["Model", "Local (planned)"],
            ["Data access", "Read-only"],
            ["Can run commands", "No · suggests fixes only"],
            ["Cloud fallback", <StatusBadge key="c" variant="off" label="Off" />],
          ]}
        />
      </Panel>
    </>
  );
}
