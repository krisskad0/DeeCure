# packaging

Everything needed to install DeeCure on a system.

| Folder | What goes here |
| --- | --- |
| `debian/` | Debian packaging files. Copied to `debian/` at build time. |
| `desktop/` | `.desktop` file, AppStream metainfo, icons |
| `dbus/` | D-Bus activation and policy files for the helper |
| `polkit/` | polkit action policy: one action per helper action |
| `systemd/` | `deecure-agent.socket`, `deecure-agent.service`, `deecure-helper.service` |
| `apt-repo/` | `reprepro` config and publish scripts for the APT repository |
| `install.sh` | One-line installer *(planned)* |

`dbus/` and `polkit/` control root access. Changes there need two reviewers.

Docs: [docs/06-packaging](../docs/06-packaging/README.md)
