# Packages (`packages`)

[Docs](../../README.md) › [UI design](../README.md) › [Pages](README.md)

- Group: Software & Data · Icon: `pkg` · Status badge: {warn:Warning} (rule in [Page status rules](../04-behavior/page-status.md))
- Sub-pages: Installed, Sources, Cleanup, Permissions

## Packages › Installed: `packages/installed`

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| APT | 1,842 |  | - | 212 installed by you | - | - |
| Snap | 14 |  | - | 3.9 GB | - | - |
| Flatpak | 6 |  | - | 2.2 GB | - | - |
| AppImage | 2 |  | - | found in ~/Downloads | - | - |
| Language packages | 29 |  | - | pip · npm · cargo | - | - |
| Updates | 12 |  | {warn:Warning} |   | - | `system/updates` |

2. **Donut chart**: "Disk usage by source" · span 4
- Center: "12.4" / "GB" · unit "GB"

| Segment | Value | Color | Display value |
| --- | --- | --- | --- |
| APT | 5.8 | slot 1 | - |
| Snap | 3.9 | slot 2 | - |
| Flatpak | 2.2 | slot 3 | - |
| AppImage | 0.1 | slot 4 | 0.1 GB |
| Other | 0.4 | slot 5 | - |

3. **Data table**: "Packages" · span 8
- Search: "Search packages"
- Filter chips on column 3: All / apt / snap / flatpak / AppImage
- Page size 10
- Total count shown: 1,893
- Inspect: row click opens the row-inspect drawer with actions "Remove", "Show files"
- Table ID: `pkgs`

| Name | Version | Source | Size [n] | Installed by | Last used |
| --- | --- | --- | --- | --- | --- |
| firefox | `131.0.3` | {info:snap} | 286 MB | System | Today |
| code | `1.94.2` | {info:snap} | 340 MB | User | Today |
| google-chrome-stable | `130.0.6723.58` | {off:apt · 3rd-party} | 352 MB | User | Today |
| telegram-desktop | `5.6.1` | {info:snap} | 182 MB | User | Today |
| docker-ce | `27.3.1` | {off:apt · 3rd-party} | 108 MB | User | Today |
| zoom-client | `6.2.3` | {info:snap} | 452 MB | User | 6 days ago |
| com.spotify.Client | `1.2.45` | {ok:flatpak} | 310 MB | User | Yesterday |
| com.obsproject.Studio | `30.2.3` | {ok:flatpak} | 480 MB | User | 9 days ago |
| virtualbox | `7.0.16` | {off:apt} | 190 MB | User | 12 days ago |
| vlc | `3.0.20` | {off:apt} | 12 MB | User | 3 days ago |
| vlc | `3.0.21` | {info:snap} | 320 MB | User | Never |
| gimp | `2.10.36` | {off:apt} | 98 MB | User | 124 days ago {warn:Unused} |
| shotwell | `0.32.6` | {off:apt} | 8 MB | System | 201 days ago {warn:Unused} |
| git | `2.43.0` | {off:apt} | 42 MB | User | Today |
| python3 | `3.12.3` | {off:apt} | - | System | Today |
| libreoffice-writer | `24.2.6` | {off:apt} | 64 MB | System | 18 days ago |
| balenaEtcher | `1.19.21` | {warn:AppImage} | 98 MB | User | 42 days ago |
| dropbox | `2024.04.17` | {off:apt · 3rd-party} | 38 MB | User | Today |
| httpie | `3.2.2` | `pip` | 2 MB | User | 30 days ago |
| typescript | `5.6.3` | `npm` | 22 MB | User | 2 days ago |
| ripgrep | `14.1.1` | `cargo` | 6 MB | User | Today |

## Packages › Sources: `packages/sources`

1. **Data table**: "Software sources" · span 12
- Source chip (Expert): `/etc/apt/sources.list.d`

| Repository | Type | Signed | Packages [n] | Status |
| --- | --- | --- | --- | --- |
| `archive.ubuntu.com noble main restricted universe multiverse` | Official | ✓ | 1,689 | {ok:OK} |
| `security.ubuntu.com noble-security` | Official | ✓ | 312 | {ok:OK} |
| `download.docker.com/linux/ubuntu noble stable` | Third-party | ✓ | 6 | {ok:OK} |
| `dl.google.com/linux/chrome/deb stable` | Third-party | ✓ | 1 | {ok:OK} |
| `packages.microsoft.com/repos/code stable` | Third-party | ✓ | 0 | {info:Unused} |
| `ppa:git-core/ppa` | PPA | ✓ | 4 | {ok:OK} |
| `ppa:deadsnakes/ppa` | PPA | ✓ | 0 | {info:Unused} |
| `ppa:obsproject/obs-studio` | PPA | ✓ | 0 | {info:Unused} |

## Packages › Cleanup: `packages/cleanup`

1. **Data table**: "Reclaimable" · span 12

| Item | Count [n] | Size [n] | Undo | (action) |
| --- | --- | --- | --- | --- |
| Packages no longer needed (autoremove) | 48 | 310 MB | {info:Reinstallable} | [Remove](act:toast:48 packages removed · 310 MB freed.) |
| Disabled snap revisions | 6 | 8.7 GB | {warn:Irreversible} | [Clean](act:fix:storage) |
| Duplicate app: VLC (apt + snap) | 1 | 320 MB | {info:Reinstallable} | [Review](act:toast:Keep one VLC to save 320 MB.) |
| Unused Flatpak runtimes | 2 | 640 MB | {info:Regenerates} | [Remove](act:toast:flatpak uninstall --unused) |
| Apps not used in 90+ days | 2 | 106 MB | {off:Review} | [Review](go:packages/installed) |
| Old kernel 6.8.0-41 | 1 | 320 MB | {info:Reinstallable} | [Remove](act:toast:Old kernel queued for removal.) |

## Packages › Permissions: `packages/permissions`

1. **Data table**: "Sandboxed app permissions" · span 12
- Source chip (Expert): `snap connections · flatpak info --show-permissions`

| App | Source | Camera | Microphone | Home folder | Network | Location |
| --- | --- | --- | --- | --- | --- | --- |
| zoom-client | {info:snap} | ✓ | ✓ | ✓ | ✓ | - |
| code | {info:snap} | - | - | ✓ | ✓ | - |
| telegram-desktop | {info:snap} | ✓ | ✓ | ✓ | ✓ | - |
| firefox | {info:snap} | ✓ | ✓ | ✓ | ✓ | ✓ |
| com.spotify.Client | {ok:flatpak} | - | - | - | ✓ | - |
| com.obsproject.Studio | {ok:flatpak} | ✓ | ✓ | ✓ | ✓ | - |

---

← Previous: [Containers](12-containers.md) · [Index](README.md) · Next: [Storage](14-storage.md) →
