# rules

Checks (rules) and fix actions, as YAML. A rule turns system data into a finding. A fix action is one safe step that solves it.

```
rules/
├── <category>/<name>.yaml          # rule, id: <category>.<name>
└── fixes/<category>/<name>.yaml    # fix action
```

Categories match the app pages: `storage`, `security`, `power`, `services`, `packages`, and so on.

- Format: [Rules engine](../docs/02-architecture/rules-engine.md)
- How to add one: [Writing rules and KB entries](../docs/04-development/writing-rules-and-kb.md)
