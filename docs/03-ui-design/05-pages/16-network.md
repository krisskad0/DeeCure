# Network (`network`)

[Docs](../../README.md) › [UI design](../README.md) › [Pages](README.md)

- Group: Network & Access · Icon: `wifi` · Status badge: {ok:Healthy} (rule in [Page status rules](../04-behavior/page-status.md))
- Sub-pages: Interfaces, Connections, Open Ports, Firewall, DNS & VPN, Traffic

## Network › Interfaces: `network/interfaces`

1. **Data table**: "Interfaces" · span 12
- Source chip (Expert): `ip -br addr · nmcli device`

| Interface | Type | State | IPv4 | MAC [x] | Speed | RX [n] | TX [n] |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `wlp0s20f3` | Wi-Fi | {ok:connected} | `192.168.1.24/24` | `3c:a9:f4:..:1b` | 866 Mb/s | 4.0 GB | 0.6 GB |
| `docker0` | Bridge | {ok:connected} | `172.17.0.1/16` | `02:42:..:9a` | - | 210 MB | 140 MB |
| `lo` | Loopback | {off:unmanaged} | `127.0.0.1/8` | - | - | 88 MB | 88 MB |
| `enx00e04c68` | Ethernet | {off:unavailable} | - | `00:e0:4c:..:68` | - | 0 | 0 |

2. **Key-value list**: "Wi-Fi" · span 6
- Source chip (Expert): `nmcli -f all dev wifi`

| Key | Value |
| --- | --- |
| Network | HomeNet-5G |
| Signal | [meter:ok 78% "−52 dBm"] |
| Band / channel | 5 GHz · 44 |
| Security | {ok:WPA2-PSK} |
| Link rate | 866 Mb/s |

3. **Key-value list**: "Connectivity" · span 6

| Key | Value |
| --- | --- |
| Internet | {ok:Online} |
| Gateway | `192.168.1.1` · 2 ms |
| Public IP | Hidden [Reveal](act:toast:Looking up the public IP contacts an outside service. Off in the prototype.) |
| IPv6 | Link-local only |
| Captive portal | None |

## Network › Connections: `network/connections`

1. **Data table**: "Active connections" · span 12
- Source chip (Expert): `ss -tunp`
- Search: "Filter connections"
- Filter chips on column 4: All / ESTAB / LISTEN / UNCONN
- Page size 12
- Total count shown: 68

| Proto | Local | Remote | State | Process |
| --- | --- | --- | --- | --- |
| tcp | `192.168.1.24:51432` | `142.250.183.14:443` | {ok:ESTAB} | chrome |
| tcp | `192.168.1.24:51438` | `142.250.77.110:443` | {ok:ESTAB} | chrome |
| tcp | `192.168.1.24:42210` | `149.154.167.51:443` | {ok:ESTAB} | telegram-desktop |
| tcp | `192.168.1.24:39812` | `162.125.21.3:443` | {ok:ESTAB} | dropbox |
| tcp | `192.168.1.24:50114` | `13.107.6.158:443` | {ok:ESTAB} | code |
| tcp | `0.0.0.0:5432` | `*:*` | {info:LISTEN} | docker-proxy |
| tcp | `127.0.0.1:6379` | `*:*` | {info:LISTEN} | docker-proxy |
| tcp | `127.0.0.1:631` | `*:*` | {info:LISTEN} | cupsd |
| udp | `127.0.0.53:53` | `*:*` | {off:UNCONN} | systemd-resolved |
| udp | `0.0.0.0:5353` | `*:*` | {off:UNCONN} | avahi-daemon |
| tcp | `192.168.1.24:51900` | `185.125.190.20:443` | {off:TIME-WAIT} | - |

## Network › Open Ports: `network/ports`

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Listening | 5 |  | - |   | - | - |
| Local only | 3 |  | - |   | - | - |
| Network-exposed | 2 |  | {warn:Warning} |   | - | - |
| Reachable from internet | Unknown |  | - | Needs outside test | - | - |

2. **Data table**: "Listening ports" · span 12
- Source chip (Expert): `ss -tulpn`

| Port | Protocol | Bind address | Exposure | Process | Firewall |
| --- | --- | --- | --- | --- | --- |
| `5432` | TCP | `0.0.0.0` | {warn:LAN} | docker-proxy (pg-dev) | {warn:Bypassed} |
| `5353` | UDP | `0.0.0.0` | {info:LAN} | avahi-daemon | {ok:Allowed (mDNS)} |
| `6379` | TCP | `127.0.0.1` | {off:Local} | docker-proxy (redis-dev) | - |
| `631` | TCP | `127.0.0.1` | {off:Local} | cupsd | - |
| `53` | UDP | `127.0.0.53` | {off:Local} | systemd-resolved | - |

3. **Alert banner**: "External reachability is not tested" · span 12
- State: `info` · Title: "External reachability is not tested" · Text: "Whether the router forwards a port can only be checked from outside your network." · Buttons: [Run external test](act:toast:Opt-in: this sends your public IP to a test server. Off in the prototype.)

## Network › Firewall: `network/firewall`

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Status | Active |  | - | ufw | - | - |
| Incoming | Deny |  | - | default | - | - |
| Outgoing | Allow |  | - | default | - | - |
| Rules | 2 |  | - |   | - | - |
| Blocked today | 37 |  | - | packets | - | - |

2. **Data table**: "Rules" · span 12
- Source chip (Expert): `sudo ufw status numbered`
- Empty text: "Requires Full access"

| # [n] | To | Action | From | Comment |
| --- | --- | --- | --- | --- |
| 1 | `5353/udp` | {ok:ALLOW IN} | Anywhere | mDNS |
| 2 | `1714:1764/tcp` | {ok:ALLOW IN} | `192.168.1.0/24` | GSConnect |

3. **Alert banner**: "Docker rules are not managed by ufw" · span 12
- State: `warn` · Title: "Docker rules are not managed by ufw" · Text: "Port 5432 (pg-dev) is reachable from the LAN even though ufw denies incoming traffic." · Buttons: [View container](go:containers/containers)

## Network › DNS & VPN: `network/dns`

1. **Key-value list**: "DNS" · span 6
- Source chip (Expert): `resolvectl status`

| Key | Value |
| --- | --- |
| Resolver | systemd-resolved |
| Servers | `192.168.1.1` |
| Search domain | - |
| DNSSEC | No |
| DNS over TLS | No |
| /etc/hosts entries | 3 |

2. **Data table**: "VPN connections" · span 6
- Source chip (Expert): `nmcli connection show`

| Name | Type | State | (action) |
| --- | --- | --- | --- |
| Office | WireGuard | {off:Disconnected} | [Connect](act:toast:Connecting to Office (WireGuard)…) |
| Tailscale | tailscale | {ok:Connected} |   |

## Network › Traffic: `network/traffic`

1. **Line chart**: "Throughput" · span 12
- Source chip (Expert): `/proc/net/dev`
- Height 220px · Y axis 0 to auto · unit "MB/s"
- Series: Download (slot 1, gen[9, 0.8, 0.6, 0.8, 0, 6, 1.2]); Upload (slot 2, gen[10, 0.2, 0.2, 0.3, 0, 3, 0.3])
- X axis: time (current range)

2. **Data table**: "Traffic by application" · span 12
- Subtitle: "today"
- Source chip (Expert): `nethogs (admin)`
- Empty text: "Requires Full access"

| Application | Downloaded [n] | Uploaded [n] | Share |
| --- | --- | --- | --- |
| chrome | 2.6 GB | 210 MB | [meter 62%] |
| snapd | 620 MB | 4 MB | [meter 15%] |
| dropbox | 310 MB | 280 MB | [meter 13%] |
| telegram-desktop | 180 MB | 40 MB | [meter 5%] |
| code | 90 MB | 12 MB | [meter 2%] |

---

← Previous: [Backups](15-backups.md) · [Index](README.md) · Next: [Accounts](17-accounts.md) →
