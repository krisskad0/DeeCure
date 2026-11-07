#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Draw the README's top animation: how deep DeeCure looks, as an animated SVG.

Eight layers nested one inside the other, from the apps down to the hardware. A scan walks
down them; what it finds appears on the right as plain sentences, each with its path down
the layers, next to live readings. CSS animation only (no script), so GitHub plays it.
With "reduce motion" it shows the finished scan.

  docs/07-user-guide/images/deecure-deep-look-light.svg
  docs/07-user-guide/images/deecure-deep-look-dark.svg

Usage: scripts/gen-deep-look.py           write both files
       scripts/gen-deep-look.py --check   fail if the files are out of date
"""

import random
import sys
from pathlib import Path

OUT = Path(__file__).resolve().parent.parent / "docs/07-user-guide/images"

# The site's own colours (site/style.css).
THEMES = {
    "light": dict(bg="#f6f7f9", panel="#ffffff", panel2="#f1f3f7", line="#e2e5ea", line2="#cfd4dc",
                  ink="#111620", ink2="#465061", muted="#6b7486", accent="#2563c9", accent_ink="#ffffff",
                  accent_soft="#e8f0fc", good="#14742b", good_soft="#e4f4e6", warn="#8a5900",
                  warn_soft="#fdf2da", depth=0.03),
    "dark": dict(bg="#0b0d11", panel="#14171c", panel2="#191d23", line="#252a32", line2="#343b46",
                 ink="#e8ebef", ink2="#b4bccb", muted="#8790a0", accent="#5f9df0", accent_ink="#0b0d11",
                 accent_soft="#16243a", good="#5fcf6a", good_soft="#12261a", warn="#f6c457",
                 warn_soft="#2c2411", depth=0.035),
}

# The layers, from the surface down. The check counts are the site's (#deep); warn = a finding.
LAYERS = [
    ("Apps and processes", "CPU, memory, battery, and network per app · /proc, cgroups", 14, False),
    ("Services, start-up, and jobs", "Failed units, restart loops, boot chain, timers · systemd", 14, False),
    ("Packages and updates", "Security updates, broken installs, checksums · apt, dpkg", 14, False),
    ("Network", "Wi-Fi, DNS, every open port, link errors · /proc/net", 14, False),
    ("Security and backups", "Firewall, AppArmor, SSH, admin accounts · /etc/ufw", 14, True),
    ("Storage and filesystems", "Space, days until full, ext4 errors · statvfs", 14, True),
    ("Kernel, drivers, firmware", "Crashes, taint, Secure Boot · fwupd", 17, False),
    ("Hardware", "Sensors, SSD, battery, memory, PCIe", 16, False),
]
TOTAL = sum(n for _, _, n, _ in LAYERS)
SCORE = 92

# What the user sees: (layer index, warn, sentence, path down the layers, has a Fix button)
FINDS = [
    (4, True, "The firewall is off", "Computer › Security › Firewall › /etc/ufw", True),
    (5, True, "Main disk full in about 12 days", "Computer › Storage › / (ext4) › 14-day trend", False),
    (7, False, "SSD: 92% of its life left", "Computer › Hardware › nvme0 › SMART", False),
    (7, False, "Battery health 84%", "Computer › Hardware › BAT0 › 320 cycles", False),
]

# Timeline, in seconds. One loop is T long.
T = 15.0
S0, STEP, SCAN = 0.6, 0.95, 0.8
START = [S0 + i * STEP for i in range(len(LAYERS))]
DONE = [s + SCAN for s in START]
FINAL = DONE[-1] + 0.35
OUT_AT = 14.2  # everything fades out here, then the loop starts again

W, H = 1200, 720
SANS = '"Ubuntu Sans", Ubuntu, "Segoe UI", system-ui, -apple-system, "Helvetica Neue", Arial, sans-serif'
MONO = 'ui-monospace, "Ubuntu Mono", "DejaVu Sans Mono", Menlo, Consolas, monospace'


def mix(a, b, t):
    """Blend colour a towards b by t (0 to 1)."""
    pa = [int(a[i:i + 2], 16) for i in (1, 3, 5)]
    pb = [int(b[i:i + 2], 16) for i in (1, 3, 5)]
    return "#" + "".join(f"{round(x + (y - x) * t):02x}" for x, y in zip(pa, pb))


def esc(s):
    return s.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")


class Anim:
    """Collects one @keyframes per animated element, on the shared loop of T seconds."""

    def __init__(self):
        self.css = []
        self.n = 0

    def keys(self, points, extra=""):
        """points: [(seconds, "prop: value; ...")]. Returns the class to put on the element."""
        self.n += 1
        name = f"k{self.n}"
        last = -1.0
        frames = []
        for t, decl in sorted(points, key=lambda p: p[0]):
            t = max(t, last + 0.001)  # keyframes need rising times
            last = t
            frames.append(f"{min(t, T) / T * 100:.3f}%{{{decl}}}")
        self.css.append(f".{name}{{animation:{name} {T}s linear infinite{extra}}}"
                        f"@keyframes {name}{{{''.join(frames)}}}")
        return name

    def show(self, on, off=OUT_AT, fade=0.2, slide=0):
        """Visible from `on` to `off`; slides up by `slide` px while it fades in."""
        if slide:
            hid, vis = f"opacity:0;transform:translateY({slide}px)", "opacity:1;transform:translateY(0)"
        else:
            hid, vis = "opacity:0", "opacity:1"
        return self.keys([(0, hid), (on, hid), (on + fade, vis), (off, vis), (off + fade, hid), (T, hid)])

    def steps(self, times, values, prop, fmt, ease=0.35):
        """A property that moves to values[k] at times[k] and back to values[0] at the end."""
        pts = [(0, f"{prop}:{fmt(values[0])}")]
        cur = values[0]
        for t, v in zip(times, values[1:]):
            pts += [(t, f"{prop}:{fmt(cur)}"), (t + ease, f"{prop}:{fmt(v)}")]
            cur = v
        pts += [(OUT_AT, f"{prop}:{fmt(cur)}"), (OUT_AT + 0.5, f"{prop}:{fmt(values[0])}"),
                (T, f"{prop}:{fmt(values[0])}")]
        return self.keys(pts)


def cycle_texts(x, y, values, cls, anchor="start", period=4.0, name="c"):
    """Live values: each shows for an equal share of `period`, forever (its own short loop)."""
    out, css = [], []
    n = len(values)
    for i, v in enumerate(values):
        a, b = i / n * 100, (i + 1) / n * 100
        k = f"{name}{i}"
        css.append(f".{k}{{animation:{k} {period}s steps(1,end) infinite}}"
                   f"@keyframes {k}{{0%{{opacity:{1 if i == 0 else 0}}}{a:.2f}%{{opacity:1}}"
                   f"{b:.2f}%{{opacity:0}}100%{{opacity:{1 if i == 0 else 0}}}}}")
        op = "1" if i == 0 else "0"
        out.append(f'<text class="{cls} {k}" x="{x}" y="{y}" text-anchor="{anchor}" opacity="{op}">{esc(v)}</text>')
    return "".join(out), "".join(css)


def spark(rng, x, y, w, h, name, period):
    """A live line that scrolls left forever: one random wave drawn twice, moved by one width."""
    n = 12
    ys = [y + rng.uniform(0.15, 0.95) * h for _ in range(n)]
    ys.append(ys[0])
    pts = [(x + i * w / n, ys[i]) for i in range(n + 1)]
    pts += [(px + w, py) for px, py in pts[1:]]
    d = "M" + " L".join(f"{px:.1f} {py:.1f}" for px, py in pts)
    area = d + f" L{x + 2 * w:.1f} {y + h} L{x} {y + h}Z"
    css = (f".{name}{{animation:{name} {period}s linear infinite}}"
           f"@keyframes {name}{{from{{transform:translateX(0)}}to{{transform:translateX(-{w}px)}}}}")
    svg = (f'<clipPath id="{name}c"><rect x="{x}" y="{y - 2}" width="{w}" height="{h + 4}"/></clipPath>'
           f'<g clip-path="url(#{name}c)"><g class="{name}"><path class="sa" d="{area}"/>'
           f'<path class="sl" d="{d}"/></g></g>')
    return svg, css


def build(theme):
    c = THEMES[theme]
    a = Anim()
    rng = random.Random(7)
    extra_css = []
    s = []

    # Background
    s.append(f'<rect class="bg" x="0.5" y="0.5" width="{W - 1}" height="{H - 1}" rx="22"/>')

    # Title and status line
    s.append('<text class="h1" x="32" y="42">How deep DeeCure looks</text>')
    s.append(f'<text class="sub" x="32" y="64">8 layers, {TOTAL} checks, 55 warnings: '
             'from the apps you see to the chips inside</text>')
    s.append(f'<text class="st {a.show(0, START[0], fade=0.15)}" x="1168" y="40" text-anchor="end" '
             'opacity="0">Starting the scan…</text>')
    for i, (name, *_rest) in enumerate(LAYERS):
        off = START[i + 1] if i + 1 < len(LAYERS) else FINAL
        s.append(f'<text class="st {a.show(START[i], off, fade=0.1)}" x="1168" y="40" text-anchor="end" '
                 f'opacity="0">Layer {i + 1} of {len(LAYERS)}: {esc(name)}</text>')
    s.append(f'<text class="st {a.show(FINAL)}" x="1168" y="40" text-anchor="end">'
             'All 8 layers checked: 2 things to look at</text>')
    s.append('<rect class="track" x="868" y="54" width="300" height="4" rx="2"/>')
    cum = [0]
    for _, _, n, _ in LAYERS:
        cum.append(cum[-1] + n)
    bar = a.steps(DONE, [k / TOTAL for k in cum], "transform", lambda v: f"scaleX({v:.3f})")
    s.append(f'<rect class="bar {bar}" x="868" y="54" width="300" height="4" rx="2"/>')

    # The nested layers
    boxes, rows = [], []
    for i, (name, sub, n, warn) in enumerate(LAYERS):
        x, y = 32 + i * 16, 84 + i * 58
        w, h = 668 - i * 32, (692 - i * 6) - (84 + i * 58)
        fill = mix(c["panel"], c["accent"], i * c["depth"])
        stroke = mix(c["line2"], c["accent"], 0.12 + i * 0.075)
        boxes.append(f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="{16 - i}" fill="{fill}" stroke="{stroke}" stroke-width="1.3"/>')
        bx, by = x + 28, y + 29
        r = []
        r.append(f'<rect class="hl {a.show(START[i], DONE[i] + 0.1, fade=0.15)}" x="{x + 4}" y="{y + 4}" '
                 f'width="{w - 8}" height="50" rx="10" opacity="0"/>')
        r.append(f'<g class="{a.show(START[i], DONE[i] + 0.1, fade=0.1)}" opacity="0">'
                 f'<circle class="pulse" cx="{bx}" cy="{by}" r="14"/></g>')
        r.append(f'<circle class="bdg0" cx="{bx}" cy="{by}" r="14"/>'
                 f'<text class="bdt0" x="{bx}" y="{by + 4}" text-anchor="middle">L{i + 1}</text>')
        r.append(f'<g class="{a.show(START[i], fade=0.15)}"><circle class="bdg" cx="{bx}" cy="{by}" r="14"/>'
                 f'<text class="bdt" x="{bx}" y="{by + 4}" text-anchor="middle">L{i + 1}</text></g>')
        r.append(f'<text class="ln" x="{x + 52}" y="{y + 25}">{esc(name)}</text>')
        r.append(f'<text class="ls" x="{x + 52}" y="{y + 44}">{esc(sub)}</text>')
        px = x + w - 72
        r.append(f'<text class="lc" x="{px - 10}" y="{y + 34}" text-anchor="end">{n} checks</text>')
        kind, word = ("w", "Look") if warn else ("o", "OK")
        r.append(f'<g class="{a.show(DONE[i], fade=0.2)}"><rect class="p{kind}" x="{px}" y="{y + 17}" width="58" '
                 f'height="24" rx="12"/><text class="pt{kind}" x="{px + 29}" y="{y + 33}" '
                 f'text-anchor="middle">{word}</text></g>')
        rows.append("".join(r))
    s += boxes + rows
    s.append('<text class="edge" x="44" y="708">Surface</text>')
    s.append('<text class="edge" x="588" y="708" text-anchor="end">Silicon</text>')
    s.append('<path class="edgeline" d="M98 704 H532 M524 700 532 704 524 708"/>')

    # Inside the hardware layer: the parts it reads, some of them live
    tiles = [("CPU", ["52 °C", "54 °C", "51 °C", "53 °C"]), ("Fan", ["2380 rpm", "2440 rpm", "2410 rpm", "2460 rpm"]),
             ("SSD life left", ["92%"]), ("Battery health", ["84%"]), ("Memory errors", ["0"]),
             ("PCIe link", ["x4 Gen 4"])]
    for j, (label, vals) in enumerate(tiles):
        tx, ty = 158 + (j % 3) * 142, 554 + (j // 3) * 44
        s.append(f'<rect class="tile" x="{tx}" y="{ty}" width="132" height="38" rx="8"/>')
        s.append(f'<text class="tl" x="{tx + 10}" y="{ty + 14}">{label}</text>')
        if len(vals) > 1:
            svg, css = cycle_texts(tx + 10, ty + 31, vals, "tv", name=f"t{j}", period=4.0 + j)
            s.append(svg)
            extra_css.append(css)
            s.append(f'<circle class="live" cx="{tx + 120}" cy="{ty + 12}" r="3"/>')
        else:
            s.append(f'<text class="tv" x="{tx + 10}" y="{ty + 31}">{vals[0]}</text>')

    # Right: the score
    s.append('<rect class="card" x="740" y="84" width="428" height="120" rx="14"/>')
    s.append('<circle class="rtrack" cx="800" cy="144" r="38"/>')
    ring = a.steps(DONE, [100 - SCORE * k / len(LAYERS) for k in range(len(LAYERS) + 1)],
                   "stroke-dashoffset", lambda v: f"{v:.1f}")
    s.append(f'<circle class="ring {ring}" cx="800" cy="144" r="38" pathLength="100" '
             f'transform="rotate(-90 800 144)" stroke-dashoffset="{100 - SCORE}"/>')
    scores = [round(SCORE * k / len(LAYERS)) for k in range(len(LAYERS) + 1)]
    for k, v in enumerate(scores):
        on = 0 if k == 0 else DONE[k - 1]
        last = k == len(scores) - 1
        off = OUT_AT if last else DONE[k]
        op = "1" if last else "0"
        s.append(f'<text class="num {a.show(on, off, fade=0.05)}" x="800" y="151" text-anchor="middle" '
                 f'opacity="{op}">{v}</text>')
        s.append(f'<text class="cnt {a.show(on, min(off, FINAL), fade=0.05)}" x="862" y="162" opacity="0">'
                 f'{cum[k]} of {TOTAL} checks done</text>')
    s.append('<text class="of" x="800" y="167" text-anchor="middle">of 100</text>')
    s.append(f'<text class="big {a.show(0, FINAL, fade=0.15)}" x="862" y="138" opacity="0">Checking…</text>')
    s.append(f'<text class="big {a.show(FINAL)}" x="862" y="138">2 things to look at</text>')
    s.append(f'<text class="cnt {a.show(FINAL)}" x="862" y="162">Everything else is healthy</text>')
    s.append('<circle class="live blink" cx="1094" cy="104" r="4"/><text class="lc" x="1104" y="108">Live</text>')

    # Right: live readings
    s.append('<rect class="card" x="740" y="220" width="428" height="180" rx="14"/>')
    s.append('<text class="cap" x="756" y="246">LIVE READINGS</text>')
    live = [("CPU", ["7%", "12%", "9%", "15%"]), ("Memory", ["41%", "42%", "41%", "43%"]),
            ("CPU temperature", ["52 °C", "54 °C", "51 °C", "53 °C"]),
            ("Network", ["1.2 MB/s", "3.4 MB/s", "0.8 MB/s", "2.1 MB/s"])]
    for j, (label, vals) in enumerate(live):
        y = 276 + j * 32
        s.append(f'<text class="ml" x="756" y="{y}">{label}</text>')
        svg, css = cycle_texts(872, y, vals, "mv", name=f"m{j}", period=4.0 + j * 0.7)
        s.append(svg)
        extra_css.append(css)
        svg, css = spark(rng, 956, y - 16, 196, 20, f"s{j}", 6 + j)
        s.append(svg)
        extra_css.append(css)

    # Right: what you see, each with its path down the layers
    s.append('<text class="cap" x="740" y="430">WHAT YOU SEE</text>')
    for j, (li, warn, title, path, fix) in enumerate(FINDS):
        y = 444 + j * 60
        cy = y + 28
        on = DONE[li] + (0.3 if j == 3 else 0)
        lx, ly = 700 - li * 16 - 4, 84 + li * 58 + 29
        cls = a.show(on, fade=0.35, slide=8)
        s.append(f'<g class="{cls}">')
        s.append(f'<rect class="fr" x="740" y="{y}" width="428" height="56" rx="10"/>')
        s.append(f'<rect class="{"fw" if warn else "fo"}" x="740" y="{y}" width="4" height="56" rx="2"/>')
        s.append(f'<circle class="bdg" cx="770" cy="{cy}" r="13"/>'
                 f'<text class="bdt" x="770" y="{cy + 4}" text-anchor="middle">L{li + 1}</text>')
        s.append(f'<text class="ft" x="792" y="{cy - 4}">{esc(title)}</text>')
        s.append(f'<text class="fp" x="792" y="{cy + 15}">{esc(path)}</text>')
        if fix:
            s.append(f'<rect class="fix" x="1108" y="{cy - 12}" width="46" height="24" rx="7"/>'
                     f'<text class="fixt" x="1131" y="{cy + 4}" text-anchor="middle">Fix</text>')
        s.append("</g>")
        draw = a.keys([(0, "stroke-dashoffset:100;opacity:0"), (on, "stroke-dashoffset:100;opacity:1"),
                       (on + 0.5, "stroke-dashoffset:0;opacity:1"), (OUT_AT, "stroke-dashoffset:0;opacity:1"),
                       (OUT_AT + 0.2, "stroke-dashoffset:0;opacity:0"), (T, "stroke-dashoffset:100;opacity:0")])
        s.append(f'<path class="wire {draw}" pathLength="100" d="M{lx} {ly} C{lx + 50} {ly} {690} {cy} 738 {cy}"/>')
        s.append(f'<circle class="dot {a.show(on)}" cx="{lx}" cy="{ly}" r="3.5"/>')

    css = f"""
text{{font-family:{SANS}}}
.bg{{fill:{c['bg']};stroke:{c['line']}}}
.h1{{font-size:22px;font-weight:700;fill:{c['ink']}}}
.sub{{font-size:13px;fill:{c['muted']}}}
.st{{font-size:13.5px;font-weight:700;fill:{c['ink']}}}
.track{{fill:{c['line']}}}
.bar{{fill:{c['accent']};transform-box:fill-box;transform-origin:left}}
.hl{{fill:{mix(c['panel'], c['accent'], 0.12)};stroke:{c['accent']};stroke-width:1.5}}
.pulse{{fill:none;stroke:{c['accent']};stroke-width:2;transform-box:fill-box;transform-origin:center;animation:pulse 0.9s ease-out infinite}}
@keyframes pulse{{from{{transform:scale(1);opacity:0.9}}to{{transform:scale(1.9);opacity:0}}}}
.bdg0{{fill:{c['panel']};stroke:{c['accent']};stroke-width:1.5}}
.bdt0{{font-size:11px;font-weight:700;fill:{c['accent']}}}
.bdg{{fill:{c['accent']}}}
.bdt{{font-size:11px;font-weight:700;fill:{c['accent_ink']}}}
.ln{{font-size:15.5px;font-weight:700;fill:{c['ink']}}}
.ls{{font-size:12px;fill:{c['muted']}}}
.lc{{font-size:12px;fill:{c['muted']}}}
.po{{fill:{c['good_soft']}}}.pto{{font-size:12px;font-weight:700;fill:{c['good']}}}
.pw{{fill:{c['warn_soft']}}}.ptw{{font-size:12px;font-weight:700;fill:{c['warn']}}}
.edge{{font-size:11px;font-weight:700;letter-spacing:0.08em;text-transform:uppercase;fill:{c['muted']}}}
.edgeline{{fill:none;stroke:{c['line2']};stroke-width:1.5;stroke-linecap:round;stroke-linejoin:round}}
.tile{{fill:{c['panel']};stroke:{mix(c['line2'], c['accent'], 0.5)}}}
.tl{{font-size:10.5px;fill:{c['muted']}}}
.tv{{font-size:14px;font-weight:700;fill:{c['ink']};font-variant-numeric:tabular-nums}}
.live{{fill:{c['good']}}}
.blink{{animation:blink 1.6s ease-in-out infinite}}
@keyframes blink{{50%{{opacity:0.3}}}}
.card{{fill:{c['panel']};stroke:{c['line']}}}
.rtrack{{fill:none;stroke:{c['line']};stroke-width:8}}
.ring{{fill:none;stroke:{c['good']};stroke-width:8;stroke-linecap:round;stroke-dasharray:100}}
.num{{font-size:25px;font-weight:700;fill:{c['ink']}}}
.of{{font-size:10px;fill:{c['muted']}}}
.big{{font-size:16px;font-weight:700;fill:{c['ink']}}}
.cnt{{font-size:13px;fill:{c['muted']}}}
.cap{{font-size:11.5px;font-weight:700;letter-spacing:0.08em;fill:{c['accent']}}}
.ml{{font-size:13px;fill:{c['ink2']}}}
.mv{{font-size:13px;font-weight:700;fill:{c['ink']};font-variant-numeric:tabular-nums}}
.sl{{fill:none;stroke:{c['accent']};stroke-width:1.8;stroke-linejoin:round}}
.sa{{fill:{c['accent']};opacity:0.12}}
.fr{{fill:{c['panel']};stroke:{c['line']}}}
.fw{{fill:{c['warn']}}}.fo{{fill:{c['good']}}}
.ft{{font-size:14px;font-weight:700;fill:{c['ink']}}}
.fp{{font-family:{MONO};font-size:11px;fill:{c['muted']}}}
.fix{{fill:{c['accent']}}}.fixt{{font-size:12.5px;font-weight:700;fill:{c['accent_ink']}}}
.wire{{fill:none;stroke:{c['accent']};stroke-width:1.5;stroke-dasharray:100;opacity:0.75}}
.dot{{fill:{c['accent']}}}
{''.join(a.css)}
{''.join(extra_css)}
@media (prefers-reduced-motion:reduce){{*{{animation:none!important}}}}
"""
    desc = ("DeeCure looks eight layers deep, each one nested inside the one above: apps and processes, "
            "services and jobs, packages and updates, network, security and backups, storage, kernel and "
            "firmware, and the hardware itself. A scan walks down the layers, 117 checks in all. On the right, "
            "live readings move, the health score fills to 92, and what the scan found appears as plain "
            "sentences, each with its path down the layers: the firewall is off, the main disk will be full "
            "in about 12 days, the SSD has 92% of its life left, and battery health is 84%.")
    return (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}" width="{W}" height="{H}" '
            f'role="img" aria-labelledby="title desc">\n'
            f'<title id="title">How deep DeeCure looks</title>\n<desc id="desc">{desc}</desc>\n'
            f'<style>{css}</style>\n' + "\n".join(s) + "\n</svg>\n")


def main():
    check = "--check" in sys.argv[1:]
    stale = []
    for theme in THEMES:
        path = OUT / f"deecure-deep-look-{theme}.svg"
        text = build(theme)
        if check:
            if not path.exists() or path.read_text() != text:
                stale.append(path)
        else:
            path.write_text(text)
            print(f"wrote {path.relative_to(OUT.parent.parent.parent)} ({len(text) // 1024} KB)")
    if stale:
        sys.exit("out of date, run scripts/gen-deep-look.py: " + ", ".join(p.name for p in stale))


if __name__ == "__main__":
    main()
