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
| `services.unit_failed` | A service failed (warning) |
| `power.battery_worn` | Battery health is under 80% (info) or 60% (warning) |

- Format: [Rules engine](../docs/02-architecture/rules-engine.md#rule-file-reference)
- How to add one: [Writing rules and KB entries](../docs/04-development/writing-rules-and-kb.md)
