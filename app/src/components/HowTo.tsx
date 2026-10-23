// SPDX-License-Identifier: Apache-2.0
// A knowledge-base how-to article, folded open on click (docs/02-architecture/knowledge-base.md).

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
        {label}: {topic.title}
      </summary>
      <p>{topic.summary}</p>
      <ol>
        {topic.steps.map((step) => (
          <li key={step}>{step}</li>
        ))}
      </ol>
      {topic.notes && <p className="muted">{topic.notes}</p>}
    </details>
  );
}
