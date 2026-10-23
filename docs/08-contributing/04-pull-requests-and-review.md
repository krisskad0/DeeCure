# Pull requests and review

[Docs](../README.md) › [Contributing](README.md)

Every change to `main` is a pull request that passes CI, is reviewed, and is squash merged by a maintainer.

## A good pull request

- **One topic.** Small pull requests are reviewed faster and break less.
- **A clear title** in Conventional Commits form, because it becomes the commit on `main`: `fix(power): show battery health when the design capacity is missing`. CI checks the form.
- **The template filled in:** what changed and why, how it was tested, `Closes #123`.
- **Screenshots** for anything you can see in the app (light and dark), and the terminal output for CLI changes.
- **Tests** for new behavior and for every bug fix.
- **Docs** updated in `docs/` when behavior changes. Add a line to the `Unreleased` section of `CHANGELOG.md` for anything users notice.
- **Signed-off commits** (`git commit -s`). CI checks every commit.
- **Draft** while you are still working; mark it **Ready for review** when it is done.

## Checks that must pass

A pull request can merge only when all of these are green ([.github/workflows](../../.github/workflows)):

| Check | What it runs |
| --- | --- |
| `Lint and test` | `make lint` and `make test`: formatting, clippy, Rust tests, real-machine fixtures |
| `Web UI (test, type-check, build)` | UI tests, the generated-spec check, type-check, build |
| `Performance budget` | `make perf` and `make db-size` |
| `Build and test the .deb` | `make deb`, install and purge in a container, the root helper with real D-Bus and polkit |
| `Sign-off and title` | Every commit has a `Signed-off-by` line, and the title follows Conventional Commits |

The branch must also be up to date with `main` before merging (GitHub shows **Update branch**).

Also run on every pull request, and read by reviewers (not yet required, because a new advisory for an old dependency can appear on any day): **Dependencies** (`cargo-deny`, `npm audit`) and **CodeQL**. A new finding in code the pull request touches must be fixed before merging.

## Review

- At least **one approval** from a maintainer, and from the **code owner** of every area the pull request touches ([.github/CODEOWNERS](../../.github/CODEOWNERS)).
- A new push after an approval needs a fresh approval, and every review conversation must be resolved.
- Reviewers look at: does it do what the issue asked, is it tested, is it safe (no new root rights, no data sent anywhere, no shell commands built from input), is the user text plain English, and are the docs updated.
- Be specific and kind. Say what must change and what is only a suggestion (prefix it with "nit:").
- Authors answer every comment, with a change or a reason.

### Extra rules for sensitive areas

These areas run as root, ship system files, run in CI with secrets, or install software on people's computers. Changes need **two approving reviews** from maintainers, and the pull request gets the label `needs-two-reviews`:

| Area | Paths |
| --- | --- |
| Root helper | `crates/deecure-helper/`, `packaging/helper/` (polkit, D-Bus, the systemd unit) |
| Fix runner | `crates/deecure-fix/`, `crates/deecure-agent/src/fixes.rs` |
| Installer and packaging | `packaging/install.sh`, `packaging/debian/`, `scripts/build-*.sh` |
| CI and releases | `.github/workflows/`, `scripts/bump-version.sh` |

For the helper, also go through the [helper review checklist](../02-architecture/security-model.md#helper-review-checklist) and paste it into the pull request.

## Merging

- **Only maintainers merge**, and only with **Squash and merge**. Merge commits and rebase merges are turned off, so `main` stays a straight line with one commit per pull request.
- The squash commit keeps the pull request title and number, and the author's name. Its message keeps every `Signed-off-by` line.
- Breaking changes get the label `breaking-change`, a `!` in the title (`feat(cli)!: …`), and a note in the CHANGELOG.
- The branch is deleted after the merge.

## If something goes wrong after a merge

- `main` broken: a maintainer reverts the pull request at once (**Revert** button on GitHub), then the author fixes it in a new pull request. No force push.
- A security problem found: follow [SECURITY.md](../../SECURITY.md); do not discuss it in public until the fix is released.

---

← Previous: [Branches and releases](03-branches-and-releases.md) · [Contributing](README.md) · Next: [Roles and permissions](05-roles-and-permissions.md) →
