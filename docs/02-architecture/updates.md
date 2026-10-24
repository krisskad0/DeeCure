# Updates

[Docs](../README.md) › [Architecture](README.md)

How DeeCure finds and installs its own new releases. The user sets it in **Settings › About › Updates**.

## Settings

| Setting | Choices | Default |
| --- | --- | --- |
| Check for updates | Every day, every week, never | Every day |
| Releases | Pre-releases too, stable only | Pre-releases too (there is no stable release yet) |
| Install updates | Ask me first, automatically | Ask me first |

"Never" means DeeCure does not contact the internet at all. The update check is the only connection DeeCure makes, and it sends nothing about the computer: it asks GitHub's public API for the release list of `krisskad0/DeeCure`.

## Checking (the agent, as the user)

`crates/deecure-agent/src/updates.rs`, every 30 minutes, starting 10 minutes after the agent starts (so a login never goes online at once; **Check now** works right away):

1. When a check is due (a day or a week after the last one; an hour after a failed one), it fetches `https://api.github.com/repos/krisskad0/DeeCure/releases` with `curl`, or `wget` when curl is missing (Ubuntu Desktop has wget, not always curl), HTTPS only, with a time limit.
2. It picks the newest release for the channel by version order (`0.1.0-alpha.3` < `0.1.0-alpha.10` < `0.1.0-beta.1` < `0.1.0`), leaving out drafts and releases without `SHA256SUMS` and a `deecure_*_amd64.deb` (`deecure_core::update::newest`).
3. When it is newer than the running version, the user gets one notification per version ("DeeCure X is available"), outside quiet hours. Not when automatic updates are on, or when DeeCure comes from an APT repository (then Software Updater brings updates).
4. After an update, the first run of the new agent says "DeeCure was updated to X" once.

Requests: `update_status`, `set_update_settings`, `check_updates` ([protocol](ipc-protocol.md)).

## Installing (the root helper)

`InstallUpdate { version }` (polkit action `install-update`, password each time), in `crates/deecure-helper/src/update.rs`, does what `install.sh` does:

1. The version must be a version (`0.1.0-alpha.4`) and newer than the installed package (`dpkg --compare-versions`).
2. Downloads go only to DeeCure's release page, `https://github.com/krisskad0/DeeCure/releases/download/v<version>/`, built from the checked version; never a URL from outside. HTTPS only (`curl --proto =https --tlsv1.2`, or `wget --https-only`).
3. `SHA256SUMS` first. Each package must be listed there with a safe file name (no path), and its checksum must match.
4. Each package's own `Package`, `Version`, and `Architecture` (`dpkg-deb -f`) must be the expected ones.
5. `deecure`, and `deecure-desktop` only where it is installed. apt installs them with the Ubuntu libraries they need (`apt-get -y --no-install-recommends install`). The downloads are removed afterwards.

The checksums come from the same release as the packages, so they prove a download is whole, not who made it. That is the same trust as the one-command install; signed releases (decision #12) will add the signature check.

The upgrade's maintainer scripts restart the agent for everyone signed in; the helper itself and the update timer are not restarted, so the running update is never cut off. The open app keeps working and says to open it again.

## Automatic updates

- Turning them on runs `SetAutoUpdate { on, check, channel }` (polkit action `set-auto-update`, password once), which writes `/etc/deecure/update.json`. Off removes it. A change of how often or which releases while they are on asks again, so the root file always matches.
- `deecure-update.timer` runs daily, at a random time within 4 hours, and catches up after the computer was off. Its service runs only when `/etc/deecure/update.json` exists, and is installed enabled but does nothing until then.
- `deecure-helper --auto-update` uses the file only when root alone can change it, waits a day or a week after its last attempt, skips a metered connection ("waiting: the connection is metered"), finds the newest release the same way as the agent, and installs it with the same checks.
- Each result goes to the audit log and `/run/deecure/update-result.json`, which Settings shows as "Last automatic update".

## Not yet

- Signed releases and the APT repository (decision #12). Once DeeCure comes from its APT repository, Software Updater installs updates and the Install button is hidden.
- Architectures other than amd64: releases only build amd64 today.

---

[Index](README.md) · [Root helper](helper.md) · [Packaging](../06-packaging/README.md)
