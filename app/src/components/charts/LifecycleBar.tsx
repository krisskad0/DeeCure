// SPDX-License-Identifier: Apache-2.0
// 20 Lifecycle bar (docs/03-ui-design/03-components/20-lifecycle-bar.md).

export function LifecycleBar(props: { percent: number; start: string; end: string; marker?: string }) {
  const at = `${Math.max(0, Math.min(100, props.percent))}%`;
  return (
    <div className="life">
      <div className="life-track">
        <span className="life-fill" style={{ width: at }} />
        <span className="life-marker" style={{ left: at }}>
          <b>{props.marker ?? "Today"}</b>
        </span>
      </div>
      <div className="life-axis">
        <span>{props.start}</span>
        <span>{props.end}</span>
      </div>
    </div>
  );
}
