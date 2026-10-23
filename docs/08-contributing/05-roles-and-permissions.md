# Roles and permissions

[Docs](../README.md) › [Contributing](README.md)

Who can do what. Rights follow trust that is earned by good work over time; nobody starts with write access.

## Roles

| Role | GitHub role | Can | Cannot |
| --- | --- | --- | --- |
| **Contributor** | None (works in a fork) | Open issues and pull requests, review and comment on anything | Push to the repository, merge, label |
| **Triager** | Triage | Label, assign, close and reopen issues, mark duplicates | Push, merge, change settings |
| **Reviewer** | Write | Approve pull requests in their areas (listed in CODEOWNERS), push work branches to the main repository | Merge into protected branches, create tags or release branches, change settings |
| **Maintainer** | Maintain | Merge pull requests, manage milestones, revert a broken change | Change protection rules, delete the repository, manage secrets |
| **Release manager** | Maintain, listed as a ruleset bypass for tags | Create `v*` tags and `release/*` branches, publish releases | Change protection rules |
| **Owner** | Admin | Everything, including settings, rulesets, secrets, and the APT signing key | Bypass review on purpose except in an emergency; every bypass is written in the pull request or a commit message |

The current people are in [MAINTAINERS.md](../../MAINTAINERS.md).

## What is restricted, and how

The rules are enforced by GitHub, not only written here ([06-repository-settings.md](06-repository-settings.md)):

| Action | Who | Enforced by |
| --- | --- | --- |
| Push to `main` or `release/*` | Nobody (all changes through pull requests) | Ruleset: pull request required |
| Force push, rewrite history, or delete `main` or `release/*` | Nobody | Ruleset: block force pushes, restrict deletions |
| Merge a pull request | Maintainers, after CI and the reviews pass | Ruleset: required checks and reviews; repository roles |
| Merge with a merge commit or a rebase | Nobody | Settings: squash merge only |
| Create a `release/*` branch | Release manager, owner | Ruleset: restrict creations |
| Create, move, or delete a `v*` tag (start a release) | Release manager, owner (create only; never move or delete) | Tag ruleset |
| Change workflows, the helper, the installer | Anyone may propose; two maintainer reviews needed | CODEOWNERS and review rules |
| Run CI on a pull request from a new contributor | After a maintainer approves the run | Actions setting: approval for outside contributors |
| Use repository secrets (APT signing key) | Only the release workflow, on tags | Secrets are never given to pull requests from forks |
| Change settings, rulesets, secrets | Owner | GitHub admin role |

## Becoming a reviewer or maintainer

- **Triager:** after several useful issue reports or reviews. Ask a maintainer, or one will ask you.
- **Reviewer:** after about ten good pull requests merged in one area, and reviews that helped others. A maintainer proposes it; the owner agrees.
- **Maintainer:** after months of steady reviews and good judgment, especially about safety. All maintainers agree.
- **Stepping back** is fine at any time; rights are removed after 6 months without activity, with thanks, and can come back.

## Decisions

- Day-to-day decisions are made in issues and pull requests, by the reviewers of that area.
- Bigger technical choices get a short decision record in [docs/02-architecture/decisions](../02-architecture/decisions/README.md), proposed in a pull request.
- When people disagree, maintainers aim for agreement; if that fails, the owner decides, and writes down why.
- See [GOVERNANCE.md](../../GOVERNANCE.md).

---

← Previous: [Pull requests and review](04-pull-requests-and-review.md) · [Contributing](README.md) · Next: [Repository settings](06-repository-settings.md) →
