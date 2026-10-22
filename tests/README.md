# tests

Cross-crate tests and test data. Unit tests live next to the code in each crate.

| Folder | What |
| --- | --- |
| `fixtures/` | Saved `/proc`, `/sys`, disk space, and service data from one machine per folder, plus the exact findings DeeCure must report. Add one with `scripts/capture-fixture.py`. |
| `vm/` | Scripts that build Ubuntu VMs, install the package, and run checks |

Docs: [Testing](../docs/04-development/testing.md)
