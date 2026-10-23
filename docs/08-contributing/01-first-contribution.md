# Your first contribution

[Docs](../README.md) › [Contributing](README.md)

From nothing to a merged pull request, in eight steps. Ask in the issue if you get stuck: nobody expects you to know everything.

## 1. Pick something

- Good first tasks have the label [`good first issue`](https://github.com/krisskad0/DeeCure/labels/good%20first%20issue).
- You can help without writing Rust: a new check is a small YAML file in `rules/`, and a plain-English explanation is a YAML file in `kb/` ([Writing rules and KB entries](../04-development/writing-rules-and-kb.md)).
- Comment on the issue ("I would like to work on this") so two people do not do the same work. A maintainer assigns it to you.
- **Bigger than a small fix?** Open an issue first and agree on the approach. A large pull request without an agreed plan may be closed.

## 2. Fork and clone

On GitHub, click **Fork**. Then:

```bash
git clone https://github.com/<your-name>/DeeCure.git
cd DeeCure
git remote add upstream https://github.com/krisskad0/DeeCure.git
```

Set up the tools once: [Getting started](../04-development/getting-started.md).

## 3. Create a branch

Always start from the newest `main`:

```bash
git fetch upstream
git switch -c fix/battery-health upstream/main
```

Branch names: [03-branches-and-releases.md](03-branches-and-releases.md#branch-names).

## 4. Make the change

- Keep it to one topic. Two unrelated fixes are two pull requests.
- Follow the code style in [CONTRIBUTING.md](../../CONTRIBUTING.md#code-style).
- Add or update tests. A bug fix comes with a test that fails without the fix.
- If behavior changes, update the spec in `docs/` in the same pull request.
- Text people see is plain English: short sentences, no blame, no jargon without an explanation.

## 5. Test it

```bash
make fmt          # format Rust code
make lint         # formatting and lints, as CI checks them
make test         # Rust tests, including real-machine fixtures
make web          # UI tests, type-check, and build
```

Never test fixes or admin actions on your own computer: the container tests do that safely (`make deb-test`, `make helper-test`).

## 6. Commit with a sign-off

```bash
git add -A
git commit -s -m "fix(power): show battery health when the design capacity is missing"
```

- `-s` adds the `Signed-off-by` line: you confirm the [Developer Certificate of Origin](https://developercertificate.org/), which says you have the right to submit the work. CI checks it.
- The message follows [Conventional Commits](https://www.conventionalcommits.org/): `type(area): what changed`. Types: `feat`, `fix`, `docs`, `test`, `refactor`, `perf`, `build`, `ci`, `chore`.

Forgot the sign-off? `git commit --amend -s` for the last commit, or `git rebase --signoff upstream/main` for all of them, then push with `--force-with-lease` to **your own branch**.

## 7. Open the pull request

```bash
git push -u origin fix/battery-health
```

On GitHub, open a pull request into `krisskad0/DeeCure` `main`. Fill in the template: what changed, how you tested it, and `Closes #123` for the issue. The title becomes the commit message on `main`, so write it like one.

## 8. Review and merge

- CI runs automatically. For a first-time contributor, a maintainer approves the run first (a GitHub safety rule for forks).
- A reviewer may ask for changes. Push more commits to the same branch; do not open a new pull request.
- When CI is green and a maintainer approves, a maintainer **squash merges** it. Your commits become one commit on `main`, with you as the author.
- Then delete your branch and update your fork:

```bash
git switch main && git pull upstream main && git push origin main
```

Thank you. Your name is in the history, and in the release notes when the change ships.

---

← [Contributing](README.md) · Next: [Issues and labels](02-issues-and-labels.md) →
