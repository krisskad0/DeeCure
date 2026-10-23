# rules

Checks (rules) and fix actions, as YAML. A rule turns system data into a finding. A fix action is one safe step that solves it.

```
rules/
├── <category>/<name>.yaml          # rule, id: <category>.<name>
└── fixes/<category>/<name>.yaml    # fix action
```

Categories match the app pages: `storage`, `security`, `power`, `services`, `packages`, and so on.

| Rule | Fires when |
| --- | --- |
| `storage.root_almost_full` | The main disk is 85% full (warning) or 93% (critical) |
| `hardware.cpu_hot` | The processor is at 90 °C (warning) or 100 °C (critical) |
| `network.offline` | No default route for IPv4 or IPv6 (info) |
| `packages.updates_available` | Updates other than security updates are waiting (info) |
| `accounts.extra_root` | An account other than root has user ID 0 (critical) |
| `storage.filling_up` | At the last 7 days' rate, a disk is full within 14 days (warning) or 3 days (critical) |
| `services.unit_failed` | A service failed (warning) |
| `power.battery_worn` | Battery health is under 80% (info) or 60% (warning) |

- Format: [Rules engine](../docs/02-architecture/rules-engine.md#rule-file-reference)
- How to add one: [Writing rules and KB entries](../docs/04-development/writing-rules-and-kb.md)
