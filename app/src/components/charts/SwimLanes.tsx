// SPDX-License-Identifier: Apache-2.0
// 19 Swim-lane schedule, next 24 hours (docs/03-ui-design/03-components/19-swim-lane-schedule.md).

export interface Lane {
  name: string;
  /** When each run happens, in hours from now (0-24). */
  runs: number[];
  /** Very frequent jobs get small dots without a ring. */
  frequent?: boolean;
}

const pad2 = (n: number) => String(n).padStart(2, "0");

export function SwimLanes({ lanes, now }: { lanes: Lane[]; now: Date }) {
  const minutes = now.getHours() * 60 + now.getMinutes();
  // "Now", then every 00:00, 06:00, 12:00, and 18:00 in the next 24 hours.
  const marks = [{ at: 0, label: `Now ${pad2(now.getHours())}:${pad2(now.getMinutes())}` }];
  for (const hour of [0, 6, 12, 18]) {
    const ahead = (((hour * 60 - minutes) % 1440) + 1440) % 1440;
    if (ahead > 60 && ahead < 1440 - 60) marks.push({ at: ahead / 60, label: `${pad2(hour)}:00` });
  }
  return (
    <div className="lanes-scroll">
      <div className="lanes">
        {lanes.map((lane, i) => (
          // Two jobs can have the same short name, so the lane number is part of the key.
          <div className="lane" key={`${i}|${lane.name}`}>
            <span className="lane-name mono" title={lane.name}>
              {lane.name}
            </span>
            <span className="lane-track">
              {lane.runs.map((h, i) => (
                <i key={i} className={lane.frequent ? "run small" : "run"} style={{ left: `${(h / 24) * 100}%` }} />
              ))}
            </span>
          </div>
        ))}
        <div className="lane lane-axis">
          <span />
          <span className="lane-track">
            {marks.map((m) => (
              <span key={m.label} style={{ left: `${(m.at / 24) * 100}%` }}>
                {m.label}
              </span>
            ))}
          </span>
        </div>
      </div>
    </div>
  );
}
