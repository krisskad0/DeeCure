# Contributing

[Docs](../README.md)

How people work together on DeeCure: from a first contribution to merging and releasing. Read the files in order; each one is short.

| Step | File | For | What it covers |
| --- | --- | --- | --- |
| 1 | [01-first-contribution.md](01-first-contribution.md) | Everyone | Fork, branch, change, test, and open a pull request, step by step |
| 2 | [02-issues-and-labels.md](02-issues-and-labels.md) | Everyone | How to report a bug or ask for a feature, and how issues move from new to done |
| 3 | [03-branches-and-releases.md](03-branches-and-releases.md) | Everyone | The branch model, branch names, release branches, tags, and hotfixes |
| 4 | [04-pull-requests-and-review.md](04-pull-requests-and-review.md) | Everyone | What a good pull request looks like, the checks it must pass, review rules, and how it is merged |
| 5 | [05-roles-and-permissions.md](05-roles-and-permissions.md) | Everyone | Who can do what: contributors, triagers, reviewers, maintainers, and the release manager |
| 6 | [06-repository-settings.md](06-repository-settings.md) | Maintainers | The GitHub settings and rulesets that protect `main`, release branches, and tags |

## The short version

1. Look for an issue, or open one first for anything bigger than a small fix.
2. Fork the repository, create a branch such as `fix/battery-health`, and make the change.
3. Run `make lint test web` and sign off your commits (`git commit -s`).
4. Open a pull request into `main`. CI must pass and a maintainer must approve.
5. A maintainer merges it with **squash merge**. Nobody pushes to `main` directly.

The front door for new people is [CONTRIBUTING.md](../../CONTRIBUTING.md); it links here.

## Ground rules

- **`main` is always releasable.** Every change arrives through a reviewed pull request with green CI.
- **Root-level code is special.** Anything that runs as root or ships system files needs two reviews ([04](04-pull-requests-and-review.md#extra-rules-for-sensitive-areas)).
- **Docs change with code.** A pull request that changes behavior updates the spec in `docs/` too.
- **Be kind.** Everyone follows the [Code of Conduct](../../CODE_OF_CONDUCT.md).

---

← Previous: [User guide](../07-user-guide/README.md) · [Docs index](../README.md)
