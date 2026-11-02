// SPDX-License-Identifier: Apache-2.0
// A knowledge-base how-to article, folded open on click (docs/02-architecture/knowledge-base.md).

import { Icon } from "./Icon";

export interface HowToTopic {
  title: string;
  summary: string;
  steps: string[];
  notes?: string;
}

export function HowTo({ topic, label = "How to fix", open = false }: { topic?: HowToTopic; label?: string; open?: boolean }) {
  if (!topic) return null;
  return (
    <details className="howto" open={open}>
      <summary>
        <Icon name="chev" size={14} />
        <span>
          {label}: {topic.title}
        </span>
      </summary>
      <div className="howto-body">
        <p>{topic.summary}</p>
        <ol className="howto-steps">
          {topic.steps.map((step, i) => (
            <li key={step}>
              <span className="howto-n">{i + 1}</span>
              <span>{step}</span>
            </li>
          ))}
        </ol>
        {topic.notes && <p className="muted">{topic.notes}</p>}
      </div>
    </details>
  );
}
