# SysDoc

**A family doctor for your Linux computer.** SysDoc checks everything, explains what it finds in plain English, and fixes problems safely with one click and an undo button.

> **Status:** design stage. There is no code or release yet. "SysDoc" is a working name.

## What it does

- **One health score** for your whole computer, with the top problems first.
- **Plain-English answers:** "Your main disk is almost full. Here is why."
- **Safe fixes:** see the exact command first, approve once, undo when possible.
- **"Why is my PC slow?"** A 10-second check that gives one clear answer.
- **What changed:** a timeline of updates, config edits, and device events.
- **Private:** everything stays on your computer. No account. Nothing is sent anywhere unless you turn it on.
- **Optional local AI:** ask questions like "Why did my laptop restart last night?"

## Install

Not released yet. From the public beta, installing will look like this on Ubuntu 24.04 and 26.04:

```bash
curl -fsSL https://sysdoc.github.io/install.sh | sudo bash
```

Or download the `.deb` from Releases and double-click it. Updates come through Software Updater like any other app. Full guide: [docs/07-user-guide/install.md](docs/07-user-guide/install.md).

## Try the prototype

Open [docs/03-ui-design/prototype/sysdoc-dashboard.html](docs/03-ui-design/prototype/sysdoc-dashboard.html) in a browser. It is the clickable UI with sample data.

## Documentation

Start at [docs/README.md](docs/README.md).

| Folder | What's inside |
| --- | --- |
| [01-product](docs/01-product/README.md) | Vision, users, user journey, metrics, risks, open decisions |
| [02-architecture](docs/02-architecture/README.md) | Components, security model, decision records |
| [03-ui-design](docs/03-ui-design/README.md) | Design tokens, shell, 24 components, 22 pages |
| [04-development](docs/04-development/README.md) | Setup, repo layout, writing rules, testing |
| [05-roadmap](docs/05-roadmap/README.md) | Phase-by-phase build plan with task lists |
| [06-packaging](docs/06-packaging/README.md) | `.deb`, APT repository, releases |
| [07-user-guide](docs/07-user-guide/README.md) | Install, update, uninstall |

## Built with

Rust (agent, root helper, CLI) · Tauri v2 + React + TypeScript (desktop app) · SQLite · YAML rules and knowledge base · optional Ollama for the local AI.

## Repository layout

```
app/          Desktop app (Tauri + React)
crates/       Rust workspace: sysdoc-core, sysdoc-agent, sysdoc-helper, sysdoc-cli
rules/        Checks and fix actions (YAML)
kb/           Plain-English knowledge base (YAML)
packaging/    Debian package, desktop files, D-Bus, polkit, systemd, APT repo
scripts/      Developer scripts
tests/        Fixtures and VM tests
docs/         Documentation
```

Details: [docs/04-development/project-structure.md](docs/04-development/project-structure.md).

## Contributing

You can help without writing Rust: add checks and plain-English explanations in YAML. See [CONTRIBUTING.md](CONTRIBUTING.md).

## Security

Please report security problems privately. See [SECURITY.md](SECURITY.md).

## License

[Apache License 2.0](LICENSE) (proposed; see [ADR 0004](docs/02-architecture/decisions/0004-license-apache-2.0.md)).
