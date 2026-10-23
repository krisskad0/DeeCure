<!--
Thank you! The title becomes the commit on main, so write it like one:
  fix(power): show battery health when the design capacity is missing
Guide: docs/08-contributing/04-pull-requests-and-review.md
-->

## What does this change, and why?

<!-- One or two sentences. Link the issue: Closes #123 -->

## How was it tested?

<!-- Commands you ran, machines or VMs you tried. Screenshots (light and dark) for anything visible in the app. -->

## Checklist

- [ ] The title follows Conventional Commits (`type(area): what changed`)
- [ ] Every commit is signed off (`git commit -s`)
- [ ] `make lint`, `make test`, and `make web` pass
- [ ] Tests added or updated (a bug fix has a test that fails without it)
- [ ] Docs in `docs/` updated if behavior changed, and a line in `CHANGELOG.md` (Unreleased) if users notice it
- [ ] User-facing text is plain English and follows the writing style
- [ ] **Sensitive area** (helper, fix runner, installer, packaging, CI)? Label `needs-two-reviews`; for the helper, the review checklist is pasted below
- [ ] **Breaking change?** `!` in the title, label `breaking-change`, and a CHANGELOG note
