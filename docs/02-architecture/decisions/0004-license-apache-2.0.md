# 0004. Apache-2.0 license

- Status: Proposed (open decision 2: GPL or Apache 2.0)
- Date: 2026-10-22

## Context

SysDoc runs a root-level helper. Users trust it more when anyone can read the code. The license must be chosen before the first public commit.

## Decision

Use the Apache License 2.0 for all code, rules, and knowledge-base files. The `LICENSE` file at the repo root holds the full text.

## Why

- The prototype's About page already says Apache-2.0.
- It is a well-known open-source license with an explicit patent grant.
- Companies can use and package it easily, which helps adoption.
- **Option not picked:** GPL-3.0. It forces every changed copy to stay open source. Pick it if that matters more than easy adoption by companies. If you switch, replace `LICENSE` and update this record before going public.

## Consequences

- Every source file may carry a short SPDX header: `// SPDX-License-Identifier: Apache-2.0`.
- `packaging/debian/copyright` must list the license in Debian's machine-readable format.
- Third-party dependencies must use licenses compatible with Apache-2.0. Check them in CI (for example with `cargo-deny`).
