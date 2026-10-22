# DeeCure documentation

Start here. Each folder covers one area. Each file covers one topic and stays short.

> **Status:** design stage. No code yet.

## Reading order

Read the folders in number order. Then build the app by following the roadmap.

| Step | Folder | Read it to… |
| --- | --- | --- |
| 1 | [01-product](01-product/README.md) | Know what we build, for whom, and why |
| 2 | [02-architecture](02-architecture/README.md) | Know the parts of the app and how they talk |
| 3 | [03-ui-design](03-ui-design/README.md) | Build the UI: tokens, shell, components, behavior, pages |
| 4 | [04-development](04-development/README.md) | Set up your machine, learn the repo layout, write rules and tests |
| 5 | [05-roadmap](05-roadmap/README.md) | Build the app phase by phase, task by task |
| 6 | [06-packaging](06-packaging/README.md) | Ship the `.deb`, the APT repository, and releases |
| 7 | [07-user-guide](07-user-guide/README.md) | Help end users install and use DeeCure |

## How to build the app with these docs

1. Read [01-product](01-product/README.md) and [02-architecture](02-architecture/README.md) once.
2. Open [05-roadmap](05-roadmap/README.md) and start at Phase 0.
3. Each phase lists its tasks in order. Each task links to the exact spec file.
4. Turn each task into a GitHub issue. Close the issue when the "Done when" check passes.

## Try the prototype

Open [deecure-dashboard.html](03-ui-design/prototype/deecure-dashboard.html) in a browser. It is the clickable UI, with sample data. The UI spec in [03-ui-design](03-ui-design/README.md) was written from it.

## Rules for these docs

- One topic per file. Aim for under 200 lines.
- Plain English. Short sentences. Explain every technical word the first time.
- Every file starts with a breadcrumb link back to its index.
- Numbers in folder and file names show the build order.
- Change the spec first, then the code. A pull request that changes behavior also updates the doc.
- Draw diagrams in Mermaid inside Markdown. GitHub shows them.
- Record big technical choices as short decision records in [02-architecture/decisions](02-architecture/decisions/README.md).

## Before the first public release

Some choices are still open: name, license, GitHub organization, contact emails, and the signing key. See [Open decisions](01-product/open-decisions.md).

## Archive

[archive/](archive/README.md) holds the two original single-file specs. Their content now lives in the folders above.
