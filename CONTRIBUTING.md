# Contributing to DeeCure

Thank you for helping. This page explains how to contribute.

## Ways to help

| You know… | You can… | Start here |
| --- | --- | --- |
| Linux, YAML | Add checks and plain-English explanations | [Writing rules and KB entries](docs/04-development/writing-rules-and-kb.md) |
| Rust | Work on the agent, helper, or CLI | [Getting started](docs/04-development/getting-started.md) |
| React, TypeScript | Build screens and components | [UI design](docs/03-ui-design/README.md) |
| Your own laptop | Share test data from your hardware | [Testing › Adding a fixture](docs/04-development/testing.md#adding-a-fixture) |
| Writing | Improve docs and the user guide | [Docs](docs/README.md) |

## Workflow

1. Find or open an issue first. For big changes, agree on the approach in the issue.
2. Fork the repo and create a branch: `feat/disk-forecast`, `fix/battery-health`, `docs/install-guide`.
3. Keep pull requests small. One topic per pull request.
4. If the change affects behavior, update the spec in `docs/` in the same pull request.
5. Make sure `make lint` and `make test` pass.
6. Open the pull request and fill in the template.

## Commit messages

We use [Conventional Commits](https://www.conventionalcommits.org/):

```text
feat(agent): add battery wear collector
fix(cli): show free space in GB
docs(roadmap): split phase 2 tasks
```

Types: `feat`, `fix`, `docs`, `test`, `refactor`, `perf`, `build`, `ci`, `chore`.

## Sign your commits (DCO)

Add a `Signed-off-by` line to every commit with `git commit -s`. It means you agree to the [Developer Certificate of Origin](https://developercertificate.org/): you wrote the code, or you have the right to submit it.

## Code style

**Rust**

- `cargo fmt` and `cargo clippy -- -D warnings` must pass.
- No `unwrap()` or `expect()` in agent or helper code outside tests. Return errors.
- Every public function has a short doc comment.

**TypeScript and React**

- TypeScript `strict` mode. ESLint and Prettier must pass.
- Use the design tokens. No hard-coded colors or sizes.
- Components follow the spec in [docs/03-ui-design/03-components](docs/03-ui-design/03-components/README.md).

**Text shown to users**

- Plain English, short sentences, no blame.
- Follow the [writing style](docs/04-development/writing-rules-and-kb.md#writing-style).

## Extra rules for the root helper

`crates/deecure-helper/`, `packaging/polkit/`, and `packaging/dbus/` run with or control root rights.

- Changes here need **two** approving reviews.
- Follow the [helper review checklist](docs/02-architecture/security-model.md#helper-review-checklist).

## Code of conduct

Everyone in this project follows our [Code of Conduct](CODE_OF_CONDUCT.md).

## License

By contributing, you agree that your work is licensed under the [Apache License 2.0](LICENSE).
