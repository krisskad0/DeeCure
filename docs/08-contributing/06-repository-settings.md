# Repository settings (for the owner)

[Docs](../README.md) › [Contributing](README.md)

The rules in this folder are enforced by GitHub settings and **rulesets**. Rulesets cannot be turned on by a commit, so the owner sets them up once, in the browser. It takes about 15 minutes. Everything below is under the repository's **Settings** tab.

## 1. Import the rulesets

**Settings › Rules › Rulesets › New ruleset › Import a ruleset**, then pick each file from [.github/rulesets](../../.github/rulesets) (download them first) and click **Create**:

| File | Protects | Rules | Who may bypass |
| --- | --- | --- | --- |
| `main.json` | `main` | No deletion, no force push, straight history, a pull request with 1 approval and the code owner's, stale approvals dismissed, conversations resolved, **squash merge only**, all 5 CI checks green and up to date | Repository admins |
| `release-branches.json` | `release/*` | The same as `main` | Repository admins |
| `release-branch-creation.json` | `release/*` | Only maintainers and admins may create one | Admins, maintainers |
| `tag-creation.json` | `v*` tags | Only maintainers and admins may create a release tag (which starts a release) | Admins, maintainers |
| `tags-locked.json` | `v*` tags | Nobody may move or delete a release tag | Nobody |

**About the admin bypass on `main`:** it lets the owner keep pushing small changes straight to `main` while the project is young (the current way of working). When other people contribute regularly, tighten it: open the ruleset, and in **Bypass list** change the admin role from **Always** to **For pull requests only**. Then the owner also works through pull requests, and can merge their own only in an emergency.

**If a tag was made by mistake:** `tags-locked.json` has no bypass on purpose. To remove a tag, the owner turns that ruleset to **Disabled**, deletes the tag, and turns it back to **Active**, and writes down why in an issue.

## 2. Pull request settings

**Settings › General › Pull Requests:**

- [ ] Allow merge commits: **off**
- [ ] Allow squash merging: **on**, default message **Pull request title and commit details** (this keeps every `Signed-off-by` line)
- [ ] Allow rebase merging: **off**
- [ ] Always suggest updating pull request branches: **on**
- [ ] Automatically delete head branches: **on**
- [ ] Allow auto-merge: **off** (a maintainer merges by hand)

**Settings › General › Features:** Issues **on**, Discussions **on** (for questions and ideas), Wiki **off** (the docs live in `docs/`), Projects as you like.

## 3. Actions

**Settings › Actions › General:**

- [ ] Fork pull request workflows from outside collaborators: **Require approval for all outside collaborators** (new contributors' CI runs only after a maintainer looks at the change)
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
