# Repository settings (for the owner)

[Docs](../README.md) › [Contributing](README.md)

> **Status:** applied on 2026-10-23 through the GitHub API (`gh`, as `krisskad0`). Everything in sections 1 to 5 is on. Use this page to check the settings, to set them up again on a copy of the repository, or to change them later.

The rules in this folder are enforced by GitHub settings and **rulesets**. Rulesets cannot be turned on by a commit, so the owner sets them up once, in the browser. It takes about 15 minutes. Everything below is under the repository's **Settings** tab.

## 1. Import the rulesets

**Settings › Rules › Rulesets › New ruleset › Import a ruleset**, then pick each file from [.github/rulesets](../../.github/rulesets) (download them first) and click **Create**:

| File | Protects | Rules | Who may bypass |
| --- | --- | --- | --- |
| `main.json` | `main` | No deletion, no force push, straight history, a pull request with 1 approval and the code owner's, stale approvals dismissed, conversations resolved, **squash merge only**, all 6 required checks green and up to date | Repository admins |
| `release-branches.json` | `release/*` | The same as `main` | Repository admins |
| `release-branch-creation.json` | `release/*` | Only maintainers and admins may create one | Admins, maintainers |
| `tag-creation.json` | `v*` tags | Only maintainers and admins may create a release tag (which starts a release) | Admins, maintainers |
| `tags-locked.json` | `v*` tags | Nobody may move or delete a release tag | Nobody |

**About the admin bypass on `main`:** it stays on **Always**, for two uses only (agreed 2026-11-07, when the first outside contributions arrived):

- The owner pushes docs-only changes straight to `main` (the files CI treats as docs only, see [Branches](03-branches-and-releases.md#the-branches)).
- The owner merges a pull request (their own, or one they pushed fixes to) after every check has passed. With one maintainer, nobody else can approve it.

Code changes never go straight to `main`. `install.sh` is served from `main` and reaches users at once, so CI must pass before the merge, not after the push. When a second maintainer joins, change the admin role in **Bypass list** from **Always** to **For pull requests only**, and let that maintainer approve the owner's pull requests.

**If a tag was made by mistake:** `tags-locked.json` has no bypass on purpose. To remove a tag, the owner turns that ruleset to **Disabled**, deletes the tag, and turns it back to **Active**, and writes down why in an issue.

## 2. Pull request settings

**Settings › General › Pull Requests:**

- [ ] Allow merge commits: **off**
- [ ] Allow squash merging: **on**, default message **Pull request title and commit details** (this keeps every `Signed-off-by` line)
- [ ] Allow rebase merging: **off**
- [ ] Always suggest updating pull request branches: **on**
- [ ] Automatically delete head branches: **on**
- [ ] Allow auto-merge: **off** (a maintainer merges by hand)

- [ ] Require contributors to sign off on web-based commits: **on** (a file edited on the GitHub website gets the `Signed-off-by` line, so the sign-off check passes)

**Settings › General › Features:** Issues **on**, Discussions **on** (for questions and ideas), Wiki **off** (the docs live in `docs/`), Projects as you like.

## 3. Actions

**Settings › Actions › General:**

- [ ] Fork pull request workflows from outside collaborators: **Require approval for first-time contributors** (a new contributor's first CI run waits until a maintainer looks at the change; after that, their pushes run CI at once. Safe, because fork pull requests get no secrets and a read-only token)
- [ ] Workflow permissions: **Read repository contents and packages permissions** (the release workflow asks for more itself)
- [ ] Allow GitHub Actions to create and approve pull requests: **off**

Secrets (**Settings › Secrets and variables › Actions**): only `APT_SIGNING_KEY` (the signing subkey, see [APT repository](../06-packaging/apt-repository.md)). GitHub never gives secrets to pull requests from forks.

## 4. Security

**Settings › Code security:**

- [ ] Private vulnerability reporting: **on** (the [SECURITY.md](../../SECURITY.md) link depends on it)
- [ ] Dependabot alerts and security updates: **on** ([.github/dependabot.yml](../../.github/dependabot.yml) adds weekly grouped updates)
- [ ] Secret scanning and push protection: **on**

## 5. People and labels

- **Settings › Collaborators and teams:** add people only with the role they earned ([roles](05-roles-and-permissions.md)): Triage, Write (reviewers), Maintain (maintainers). Keep Admin to the owner. Update [MAINTAINERS.md](../../MAINTAINERS.md) and [CODEOWNERS](../../.github/CODEOWNERS) in the same week.
- **Labels:** with the `gh` command logged in (`gh auth login`), run `scripts/sync-labels.sh` once; run it again whenever [.github/labels.yml](../../.github/labels.yml) changes.
- **Two accounts?** The repository belongs to `krisskad0`, and commits show `krisskad1`. If both are the owner's, add `krisskad1` as a collaborator with **Admin**, and next to `@krisskad0` in CODEOWNERS.

## 6. Check that it works

1. Try `git push origin main` from an account without bypass: GitHub refuses it.
2. Open a test pull request with a title like "Test" and an unsigned commit: the **Sign-off and title** check fails, and **Merge** stays grey.
3. Fix both, wait for CI, approve: only **Squash and merge** is offered.
4. In **Settings › Rules › Rulesets**, all five rulesets show **Active**. (Do not test the tag rules by deleting a real release tag.)

---

← Previous: [Roles and permissions](05-roles-and-permissions.md) · [Contributing](README.md)
