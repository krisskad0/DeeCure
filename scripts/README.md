# scripts

Developer scripts. Run each script with `--help`, or without arguments, to see how to use it.

| Script | What it does |
| --- | --- |
| `dev-agent-units.sh` | Install or remove systemd user units that run the agent from your debug build |
| `capture-fixture.py` | Save this machine's data as a test fixture, with private data removed |
| `build-deb.sh` | Build the `.deb` into `target/deb/` and run `lintian` (used by `make deb`) |
| `test-deb.sh` | Install, use, and purge the `.deb` on a throwaway system (used by `make deb-test` and CI) |
| `bump-version.sh` *(planned)* | Set one version in every manifest |
| `dev-install-helper.sh` *(planned)* | Install dev copies of the helper's D-Bus and polkit files |
| `dev-uninstall-helper.sh` *(planned)* | Remove them again |
