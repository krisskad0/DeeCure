# Governance

How DeeCure is run, in short. Details: [docs/08-contributing](docs/08-contributing/README.md).

- **Open:** all work happens in public issues and pull requests, except security reports ([SECURITY.md](SECURITY.md)).
- **Roles:** contributors, triagers, reviewers, maintainers, and the owner ([roles and permissions](docs/08-contributing/05-roles-and-permissions.md)). Current maintainers: [MAINTAINERS.md](MAINTAINERS.md).
- **Changes:** every change to `main` is a reviewed pull request with green CI, squash merged by a maintainer. `main`, release branches, and release tags are protected by GitHub rulesets ([repository settings](docs/08-contributing/06-repository-settings.md)).
- **Safety first:** code that runs as root, installs software, or runs in CI with secrets needs two maintainer reviews.
- **Decisions:** made by the reviewers of an area; bigger technical choices are written down as decision records in [docs/02-architecture/decisions](docs/02-architecture/decisions/README.md). When maintainers cannot agree, the owner decides and writes down why.
- **Releases:** only the release manager or the owner creates release tags; tags are never moved or deleted ([branches and releases](docs/08-contributing/03-branches-and-releases.md)).
- **Conduct:** everyone follows the [Code of Conduct](CODE_OF_CONDUCT.md). Maintainers enforce it.
- **Changing these rules:** by a pull request to this file, approved by the owner.
