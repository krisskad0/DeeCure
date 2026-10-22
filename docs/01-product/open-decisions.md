# Open decisions

[Docs](../README.md) › [Product](README.md)

1. ~~**Final name:** check trademark and package-name availability.~~ **Decided 2026-10-23: DeeCure** (package and command `deecure`, repo [krisskad0/DeeCure](https://github.com/krisskad0/DeeCure)). On that day `deecure` was free as a Debian/Ubuntu package, Launchpad team, GitHub name, Snap, crates.io and npm name, and as `.com`, `.app`, `.dev` and `.org` domains. Still to do: create the Launchpad team `deecure`, register `deecure.com`, and run a legal trademark search before launch.
2. **License:** open source (GPL or Apache 2.0) builds trust for a root-level tool; decide before the first public commit.
3. **Business model:** free desktop app, with paid multi-PC and office features later.
4. **Languages:** English first; Hindi and other languages through the KB format later.
5. ~~**Rust vs Go:** pick based on what the team already knows well.~~ **Decided 2026-10-23: Rust** ([ADR 0001](../02-architecture/decisions/0001-rust-for-agent-helper-cli.md)).

## Added during repo setup (Oct 2026)

These came up while setting up the repo. Each one has a proposed default, so work can start now. Close them before the first public release.

| # | Decision | Proposed default | Where it is used |
| --- | --- | --- | --- |
| 6 | GitHub organization and app ID | `io.github.krisskad0.DeeCure` (matches the GitHub repo). Use `com.deecure.DeeCure` instead if you register `deecure.com`. Decide before the first release: the app ID must never change after that. | `.desktop` file, AppStream, D-Bus, polkit |
| 7 | License (closes #2) | Apache-2.0, matching the prototype's About page | [ADR 0004](../02-architecture/decisions/0004-license-apache-2.0.md), `LICENSE` |
| 8 | ~~Rust vs Go (closes #5)~~ | **Decided: Rust** | [ADR 0001](../02-architecture/decisions/0001-rust-for-agent-helper-cli.md) |
| 9 | Supported systems | Ubuntu 24.04 LTS and 26.04 LTS on amd64 first, then arm64, then Debian 13 | [Packaging](../06-packaging/README.md) |
| 10 | Charts in the real app | Custom SVG components that match the spec, or a library such as uPlot for long time series | [UI design](../03-ui-design/README.md) |
| 11 | Contact emails | One address for security, one for the code of conduct | `SECURITY.md`, `CODE_OF_CONDUCT.md` |
| 12 | APT signing key | Who owns it and where the offline primary key is kept | [APT repository](../06-packaging/apt-repository.md) |
| 13 | Missing insights catalog | Add `insights-catalog.md` (was `linux-system-insights.md`) | [Product](README.md), [Roadmap](../05-roadmap/README.md) |
| 14 | Onboarding and first-scan screens | Design them. They are not in the prototype. | [Phase 1](../05-roadmap/phase-1-mvp.md) |
