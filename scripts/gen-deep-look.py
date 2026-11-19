#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Draw the README's top animation: what DeeCure does, as a portrait (4:5) animated SVG.

It tells one short story in the app's own look (colours from app/src/tokens.css):
  1. The question: is your Linux computer really healthy?
  2. Look deep: a scan beam walks down 10 layers, from the apps to the hardware.
  3. See it: one health score, live readings, and only what needs you.
  4. Fix it: explain, show the exact command, apply and check again, undo.
  5. Optimize: only the improvements this computer measurably needs.
  6. The end card: name, promise, and where to get it.

The numbers are real counts: 10 layers, more than 150 readings (the 153 Expert readings in
docs/01-product/expert-coverage.md, plus the basic ones), 56 warnings (one per file in rules/),
and 49 fixes. The layer list matches the site's
"How deep it looks" section. CSS animation only (no script), so GitHub plays it. With
"reduce motion" it shows the "See it" step without moving. The size is 1080 x 1350, so the
same drawing can be recorded as a portrait video for social posts.

  docs/07-user-guide/images/deecure-deep-look-light.svg
  docs/07-user-guide/images/deecure-deep-look-dark.svg

Usage: scripts/gen-deep-look.py           write both files
       scripts/gen-deep-look.py --check   fail if the files are out of date
"""

import math
import random
import sys
from pathlib import Path

OUT = Path(__file__).resolve().parent.parent / "docs/07-user-guide/images"

# The app's own colours (app/src/tokens.css).
THEMES = {
    "light": dict(bg="#f3f4f6", panel="#ffffff", panel2="#f7f8fa", track="#e8ebf0", line="#e2e5ea",
                  line2="#cfd4dc", ink="#111620", ink2="#465061", muted="#768092", accent="#2563c9",
                  accent_ink="#ffffff", accent_soft="#e8f0fc", good="#0ca30c", good_ink="#14742b",
                  good_soft="#e4f4e6", warn="#fab219", warn_ink="#8a5900", warn_soft="#fdf2da",
                  s1="#2a78d6", s2="#eb6834", s3="#1baf7a", s4="#eda100", orb=0.16),
    "dark": dict(bg="#0b0d11", panel="#14171c", panel2="#191d23", track="#262c35", line="#252a32",
                 line2="#343b46", ink="#e8ebef", ink2="#aab2bf", muted="#7d8695", accent="#5f9df0",
                 accent_ink="#0b0d11", accent_soft="#16243a", good="#0ca30c", good_ink="#5fcf6a",
                 good_soft="#12261a", warn="#fab219", warn_ink="#f6c457", warn_soft="#2c2411",
                 s1="#3987e5", s2="#d95926", s3="#199e70", s4="#c98500", orb=0.22),
}

W, H = 1080, 1350
X0, X1 = 64, 1016  # the content column
CW = X1 - X0
SANS = '"Ubuntu Sans", Ubuntu, "Segoe UI", system-ui, -apple-system, "Helvetica Neue", Arial, sans-serif'
MONO = '"Ubuntu Sans Mono", "Ubuntu Mono", ui-monospace, "DejaVu Sans Mono", Menlo, Consolas, monospace'
EASE = "cubic-bezier(.22,1,.36,1)"     # fast in, soft landing
SPRING = "cubic-bezier(.34,1.56,.64,1)"  # a small overshoot

# The scenes: (start, end) in seconds. One loop is T long.
HOOK = (0.0, 5.4)
DEEP = (5.4, 13.2)
VIEW = (13.2, 18.2)
FIX = (18.2, 24.6)
OPT = (24.6, 28.8)
END = (28.8, 32.2)
T = END[1]
STATIC = "view"  # the scene shown with "reduce motion"

# The layers, from the surface down: (name, what it looks at, warnings, a finding here?)
# The warnings add up to the 56 rule files in rules/.
LAYERS = [
    ("Apps and processes", "CPU, memory, power per app", 3, False),
    ("Services, start-up, and jobs", "units, boot, timers, cron", 3, False),
    ("Containers", "Docker, virtual machines, ports", 1, False),
    ("Packages and updates", "apt, snap, Flatpak, checksums", 5, False),
    ("Network", "Wi-Fi, DNS, routes, open ports", 4, False),
    ("Security, accounts, and privacy", "firewall, AppArmor, SSH", 5, True),
    ("Storage and backups", "space, days until full, ext4", 10, True),
    ("Kernel, drivers, and firmware", "crashes, Secure Boot, TPM", 8, False),
    ("Power, battery, and sleep", "battery health, charger, suspend", 6, False),
    ("Hardware and devices", "sensors, SSD, PCIe, USB-C", 11, False),
]
WARNINGS = sum(n for _, _, n, _ in LAYERS)
READINGS, FIXES = 150, 49  # 150+: the Expert readings alone are 153
assert WARNINGS == 56
SCAN_START, SCAN_STEP, SCAN_LEN = DEEP[0] + 1.0, 0.5, 0.42
LSTART = [SCAN_START + i * SCAN_STEP for i in range(len(LAYERS))]
LDONE = [t + SCAN_LEN for t in LSTART]
ROW0, ROWH, ROWG = 392, 72, 8


def esc(s):
    return s.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")


class Anim:
    """Collects one @keyframes per animated element, on the shared loop of T seconds."""

    def __init__(self):
        self.css = []
        self.n = 0

    def keys(self, points, period=T, timing="linear"):
        """points: [(seconds, "prop: value; ...")]. Returns the class to put on the element."""
        self.n += 1
        name = f"k{self.n}"
        last = -1.0
        frames = []
        for t, decl in sorted(points, key=lambda p: p[0]):
            t = max(t, last + 0.001)  # keyframes need rising times
            if t > period + 1e-9:  # past the end of the loop: the scene is already gone
                continue
            last = t
            frames.append(f"{t / period * 100:.3f}%{{{decl}}}")
        self.css.append(f".{name}{{animation:{name} {period}s {timing} infinite}}"
                        f"@keyframes {name}{{{''.join(frames)}}}")
        return name

    def show(self, on, off, fade=0.45, dy=0, dx=0, timing=EASE):
        """Visible from `on` to `off`; moves in by (dx, dy) px while it fades in."""
        mv = f";transform:translate({dx}px,{dy}px)" if dx or dy else ""
        st = ";transform:translate(0,0)" if dx or dy else ""
        hid, vis = f"opacity:0{mv}", f"opacity:1{st}"
        return self.keys([(0, hid), (on, hid), (on + fade, vis), (off, vis), (off + 0.01, hid), (T, hid)], timing=timing)

    def pop(self, on, off, start=0.5, dur=0.5):
        """Grows from `start` to full size with a small overshoot (needs the "pp" class)."""
        hid, vis = f"opacity:0;transform:scale({start})", "opacity:1;transform:scale(1)"
        return "pp " + self.keys([(0, hid), (on, hid), (on + dur, vis), (off, vis), (off + 0.01, hid), (T, hid)],
                                 timing=SPRING)

    def scene(self, span):
        """A whole scene: there from `on`, then lifts away and fades at `off`."""
        on, off = span
        vis, gone = "opacity:1;transform:translateY(0)", "opacity:0;transform:translateY(-28px)"
        if on == 0:
            return self.keys([(0, vis), (off - 0.4, vis), (off, gone), (T, gone)], timing="ease-in")
        return self.keys([(0, "opacity:0"), (on, "opacity:0"), (on + 0.15, vis), (off - 0.4, vis), (off, gone),
                          (T, gone)], timing="ease-in")

    def draw(self, on, dur, off, timing=EASE):
        """A line that draws itself (needs pathLength="1" and the "dr" class)."""
        return "dr " + self.keys([(0, "stroke-dashoffset:1"), (on, "stroke-dashoffset:1"), (on + dur, "stroke-dashoffset:0"),
                                  (off, "stroke-dashoffset:0"), (off + 0.01, "stroke-dashoffset:1"),
                                  (T, "stroke-dashoffset:1")], timing=timing)

    def grow(self, on, dur, off, axis="X", timing=EASE):
        """Scales from 0 to 1 between on and on + dur, held until off (needs the "gx"/"gy" class)."""
        z, o = f"transform:scale{axis}(0)", f"transform:scale{axis}(1)"
        return self.keys([(0, z), (on, z), (on + dur, o), (off, o), (off + 0.01, z), (T, z)], timing=timing)


def text(x, y, s, cls, anchor="start", extra=""):
    a = f' text-anchor="{anchor}"' if anchor != "start" else ""
    return f'<text class="{cls}" x="{x}" y="{y}"{a}{extra}>{esc(s)}</text>'


def card(x, y, w, h, cls="card", rx=16, extra=""):
    return f'<rect class="{cls}" x="{x}" y="{y}" width="{w}" height="{h}" rx="{rx}"{extra}/>'


def pill(x, y, label, cls_box, cls_text, size=22, pad=18, anchor="start"):
    """A rounded label. x is the left edge, or the right edge with anchor="end"."""
    w = round(len(label) * size * 0.56 + 2 * pad)
    h = size + 20
    if anchor == "end":
        x -= w
    return (card(x, y, w, h, cls_box, rx=h // 2) +
            text(x + w / 2, y + h / 2 + size * 0.36, label, cls_text, "middle")), w


def check_path(cx, cy, s):
    return (f"M{cx - s * 0.45:.1f} {cy + s * 0.02:.1f} L{cx - s * 0.12:.1f} {cy + s * 0.34:.1f} "
            f"L{cx + s * 0.48:.1f} {cy - s * 0.32:.1f}")


def pulse_path(x, y, size):
    """The logo's heartbeat line, in a size x size box at (x, y)."""
    k = size / 192
    pts = [(40, 100), (68, 100), (82, 70), (102, 132), (118, 90), (126, 100), (152, 100)]
    return "M" + " L".join(f"{x + px * k:.1f} {y + py * k:.1f}" for px, py in pts)


def logo(x, y, size, line_cls="logol", extra=""):
    return (f'<rect class="logo" x="{x}" y="{y}" width="{size}" height="{size}" rx="{size * 0.22:.1f}"/>'
            f'<path class="{line_cls}" stroke-width="{13 * size / 192:.1f}" d="{pulse_path(x, y, size)}"{extra}/>')


class Clips:
    """Text that slides up from behind a mask: the classic motion-graphics title reveal."""

    def __init__(self):
        self.n = 0

    def reveal(self, a, x, y, s, cls, size, on, off, anchor="start", width=CW + 40):
        self.n += 1
        cid = f"rv{self.n}"
        left = x - 6 if anchor == "start" else x - width / 2
        k = a.keys([(0, f"transform:translateY({size * 1.25:.0f}px)"), (on, f"transform:translateY({size * 1.25:.0f}px)"),
                    (on + 0.7, "transform:translateY(0)"), (off, "transform:translateY(0)"),
                    (off + 0.01, f"transform:translateY({size * 1.25:.0f}px)"), (T, f"transform:translateY({size * 1.25:.0f}px)")],
                   timing=EASE)
        return (f'<clipPath id="{cid}"><rect x="{left:.0f}" y="{y - size * 1.05:.0f}" width="{width:.0f}" '
                f'height="{size * 1.4:.0f}"/></clipPath><g clip-path="url(#{cid})"><g class="{k}">'
                + text(x, y, s, cls, anchor) + "</g></g>")


def swaps(a, x, y, values, cls, anchor="start", off=T):
    """Text that changes at set times: values = [(time, text)], each shown until the next one.
    The last one is the one shown without motion."""
    out = []
    for i, (t, v) in enumerate(values):
        end = values[i + 1][0] if i + 1 < len(values) else off
        k = a.keys([(0, "opacity:0"), (t, "opacity:0"), (t + 0.001, "opacity:1"), (end, "opacity:1"),
                    (end + 0.001, "opacity:0"), (T, "opacity:0")])
        op = "" if i == len(values) - 1 else ' opacity="0"'
        out.append(text(x, y, v, f"{cls} {k}", anchor, op))
    return "".join(out)


def count_up(a, x, y, to, on, dur, cls, anchor="start", off=T, start=0, fmt=str, steps=14):
    """A number that counts from `start` to `to`, fast first and slow at the end."""
    vals, last = [], None
    for i in range(steps + 1):
        f = 1 - (1 - i / steps) ** 3
        v = round(start + (to - start) * f)
        if v != last:
            vals.append((on + dur * i / steps, fmt(v)))
            last = v
    return swaps(a, x, y, vals, cls, anchor, off)


def cycle(x, y, values, cls, name, period=3.2):
    """Live values: each shows for an equal share of `period`, forever (its own short loop)."""
    out, css = [], []
    n = len(values)
    for i, v in enumerate(values):
        lo, hi = i / n * 100, (i + 1) / n * 100
        k = f"{name}{i}"
        css.append(f".{k}{{animation:{k} {period}s steps(1,end) infinite}}"
                   f"@keyframes {k}{{0%{{opacity:{1 if i == 0 else 0}}}{lo:.2f}%{{opacity:1}}"
                   f"{hi:.2f}%{{opacity:0}}100%{{opacity:{1 if i == 0 else 0}}}}}")
        op = "" if i == 0 else ' opacity="0"'
        out.append(text(x, y, v, f"{cls} {k}", extra=op))
    return "".join(out), "".join(css)


def spark(rng, x, y, w, h, color, name, period):
    """A live line that scrolls left forever: one random wave drawn twice, moved by one width."""
    n = 9
    ys = [y + rng.uniform(0.15, 0.9) * h for _ in range(n)]
    ys.append(ys[0])
    pts = [(x + i * w / n, ys[i]) for i in range(n + 1)]
    pts += [(px + w, py) for px, py in pts[1:]]
    d = "M" + " L".join(f"{px:.1f} {py:.1f}" for px, py in pts)
    area = d + f" L{x + 2 * w:.1f} {y + h} L{x} {y + h}Z"
    css = (f".{name}{{animation:{name} {period}s linear infinite}}"
           f"@keyframes {name}{{from{{transform:translateX(0)}}to{{transform:translateX(-{w}px)}}}}")
    svg = (f'<clipPath id="{name}c"><rect x="{x}" y="{y - 3}" width="{w}" height="{h + 6}"/></clipPath>'
           f'<g clip-path="url(#{name}c)"><g class="{name}"><path fill="{color}" opacity="0.13" d="{area}"/>'
           f'<path fill="none" stroke="{color}" stroke-width="3" stroke-linejoin="round" d="{d}"/></g></g>')
    return svg, css


def burst(a, cx, cy, on, off, n=10, dist=46):
    """Small dots that fly out of a point and fade: a quick celebration."""
    out = []
    for i in range(n):
        ang = 2 * math.pi * i / n
        dx, dy = math.cos(ang) * dist, math.sin(ang) * dist
        k = a.keys([(0, "opacity:0;transform:translate(0,0)"), (on, "opacity:1;transform:translate(0,0)"),
                    (on + 0.6, f"opacity:0;transform:translate({dx:.0f}px,{dy:.0f}px)"), (T, "opacity:0")], timing=EASE)
        cls = "bdot" if i % 2 else "bdot2"
        out.append(f'<circle class="{cls} {k}" cx="{cx}" cy="{cy}" r="{5 if i % 2 else 4}" opacity="0"/>')
    return "".join(out)


def heading(a, rv, span, kicker, title, sub):
    on, off = span
    return (f'<g class="{a.show(on + 0.05, off, dx=-24)}">' + text(X0, 196, kicker, "kick") + "</g>"
            + rv.reveal(a, X0, 262, title, "h1", 56, on + 0.12, off)
            + f'<g class="{a.show(on + 0.4, off, dy=10)}">' + text(X0, 312, sub, "sub") + "</g>")


def scene_hook(a, rv):
    on, off = HOOK
    s = []
    # The heartbeat line, drawn across the scene, with a bright pulse running along it.
    y = 370
    d = (f"M{X0} {y} L{X0 + 360} {y} L{X0 + 392} {y - 46} L{X0 + 430} {y + 58} L{X0 + 462} {y - 22} "
         f"L{X0 + 480} {y} L{X1} {y}")
    s.append(f'<path class="ekg {a.draw(0.05, 1.2, off)}" pathLength="1" d="{d}"/>')
    s.append(f'<path class="ekgp" pathLength="1" d="{d}"/>')
    s.append(rv.reveal(a, X0, 520, "Is your Linux computer", "hero", 76, 0.1, off))
    s.append(rv.reveal(a, X0, 606, "really healthy?", "hero accent", 76, 0.3, off))
    s.append(f'<path class="uline {a.draw(0.95, 0.5, off)}" pathLength="1" d="M{X0 + 4} 632 Q {X0 + 260} 646 {X0 + 560} 628"/>')
    s.append(f'<g class="{a.show(1.3, off, dy=14)}">'
             + text(X0, 692, "The answer is spread across many", "lead")
             + text(X0, 740, "layers, tools, and log files.", "lead") + "</g>")
    s.append(f'<g class="{a.show(1.9, off, dx=-30)}">'
             + f'<rect class="accentbar" x="{X0}" y="800" width="8" height="64" rx="4"/>'
             + text(X0 + 32, 846, "DeeCure checks every layer in one go.", "lead2") + "</g>")
    stats = [(len(LAYERS), "layers deep"), (READINGS, "+ readings"), (WARNINGS, "warnings"), (FIXES, "fixes with undo")]
    cw = CW / 4
    row = [f'<g class="{a.show(2.4, off, dy=24)}">' + card(X0, 930, CW, 170)]
    for i in range(1, 4):
        row.append(f'<line class="hr" x1="{X0 + cw * i:.1f}" y1="960" x2="{X0 + cw * i:.1f}" y2="1070"/>')
    row.append("</g>")
    for i, (num, label) in enumerate(stats):
        cx = round(X0 + cw * i + cw / 2)
        plus = label.startswith("+ ")
        fmt = (lambda v: f"{v}+") if plus else str
        row.append(count_up(a, cx, 1020, num, 2.55 + i * 0.12, 1.0, "stat", "middle", off, fmt=fmt))
        row.append(f'<g class="{a.show(2.6 + i * 0.12, off)}">' + text(cx, 1062, label[2:] if plus else label, "small2", "middle")
                   + "</g>")
    s.append("".join(row))
    return "".join(s)


def scene_deep(a, rv):
    on, off = DEEP
    s = [heading(a, rv, DEEP, "STEP 1 · LOOK DEEP", "Checks every layer", "From the apps you see down to the chips inside.")]
    cum, vals = 0, [(on, "Starting the scan…")]
    for i, (_, _, n, _) in enumerate(LAYERS):
        cum += n
        vals.append((LDONE[i], f"Layer {i + 1} of {len(LAYERS)} · {cum} of {WARNINGS} warnings checked"))
    s.append(swaps(a, X0, 364, vals, "count", off=off))
    p, _ = pill(X1, 334, "2 things to look at", "wsoft", "wtext", anchor="end")
    s.append(f'<g class="{a.pop(LDONE[-1] + 0.15, off)}">' + p + "</g>")
    rows = []
    for i, (name, hint, n, look) in enumerate(LAYERS):
        x, w, y = X0 + i * 10, CW - i * 20, ROW0 + i * (ROWH + ROWG)
        enter = on + 0.35 + i * 0.05
        k = a.keys([(0, "opacity:0;transform:translateX(60px)"), (enter, "opacity:0;transform:translateX(60px)"),
                    (enter + 0.5, "opacity:0.42;transform:translateX(0)"), (LSTART[i], "opacity:0.42;transform:translateX(0)"),
                    (LSTART[i] + 0.12, "opacity:1;transform:translateX(0)"), (off, "opacity:1;transform:translateX(0)"),
                    (off + 0.01, "opacity:0"), (T, "opacity:0")], timing=EASE)
        r = [f'<g class="{k}">', card(x, y, w, ROWH, rx=14)]
        r.append(f'<g class="{a.show(LSTART[i], LDONE[i] + 0.25, fade=0.1)}" opacity="0">'
                 + card(x, y, w, ROWH, "scan", rx=14) + "</g>")
        r.append(f'<circle class="badge" cx="{x + 40}" cy="{y + 36}" r="21"/>'
                 + text(x + 40, y + 43, str(i + 1), "badget", "middle"))
        r.append(text(x + 76, y + 33, name, "rname") + text(x + 76, y + 59, hint, "rhint"))
        p, pw = pill(x + w - 20, y + 15, "Look" if look else "OK", "wsoft" if look else "gsoft",
                     "wtext" if look else "gtext", anchor="end")
        r.append(f'<g class="{a.pop(LDONE[i], off, start=0.3, dur=0.45)}">' + p + "</g>")
        r.append(f'<g class="{a.show(LDONE[i], off, fade=0.3)}">'
                 + text(x + w - 20 - pw - 14, y + 44, f"{n} check{'s' if n > 1 else ''}", "rcount", "end") + "</g>")
        r.append("</g>")
        rows.append("".join(r))
    s.append("".join(rows))
    # The scan beam: a glowing line that sweeps down the layers.
    pts = [(0, "opacity:0;transform:translateY(0)"), (LSTART[0] - 0.01, "opacity:0;transform:translateY(0)"),
           (LSTART[0], "opacity:1;transform:translateY(0)")]
    for i in range(1, len(LAYERS)):
        pts.append((LSTART[i], f"opacity:1;transform:translateY({i * (ROWH + ROWG)}px)"))
    end_y = len(LAYERS) * (ROWH + ROWG) - ROWG
    pts += [(LDONE[-1], f"opacity:1;transform:translateY({end_y - ROWH / 2:.0f}px)"),
            (LDONE[-1] + 0.3, f"opacity:0;transform:translateY({end_y - ROWH / 2:.0f}px)"), (T, "opacity:0")]
    beam = a.keys(pts)
    by = ROW0 + ROWH / 2
    s.append(f'<g class="{beam}" opacity="0"><rect x="{X0 - 30}" y="{by - 34}" width="{CW + 60}" height="68" fill="url(#glow)"/>'
             f'<rect x="{X0 - 30}" y="{by - 1.5}" width="{CW + 60}" height="3" fill="url(#beam)"/></g>')
    hist = ROW0 + len(LAYERS) * (ROWH + ROWG) + 26
    s.append(f'<g class="{a.show(LDONE[-1] + 0.3, off, dy=10)}">'
             + text(X0, hist, "+ the history of every layer: logs, crashes, boots, and what changed", "hist") + "</g>")
    return "".join(s)


def scene_view(a, rv, rng, colours, extra_css):
    on, off = VIEW
    s = [heading(a, rv, VIEW, "STEP 2 · SEE IT AT A GLANCE", "One score, live readings",
                 "And only what needs you, in plain English.")]
    # The health score
    s.append(f'<g class="{a.show(on + 0.25, off, dy=30)}">' + card(X0, 372, CW, 236) + "</g>")
    s.append(f'<g class="{a.show(on + 0.3, off)}"><circle class="rtrack" cx="184" cy="490" r="78" stroke-width="18"/></g>')
    s.append(f'<circle class="rgood {a.draw(on + 0.45, 1.3, off)}" pathLength="1" cx="184" cy="490" r="78" stroke-width="18" '
             f'transform="rotate(-90 184 490)" style="stroke-dasharray:0.92 1"/>')
    s.append(count_up(a, 184, 506, 92, on + 0.45, 1.3, "score", "middle", off))
    s.append(f'<g class="{a.show(on + 0.5, off)}">' + text(184, 538, "of 100", "small", "middle") + "</g>")
    s.append(f'<g class="{a.show(on + 1.5, off, dx=-24)}">' + text(316, 472, "Healthy", "good")
             + text(316, 516, "2 things to look at", "big2") + text(316, 556, "Everything else is fine", "small2") + "</g>")
    s.append(f'<g class="{a.pop(on + 0.8, off)}">' + card(X1 - 28 - 104, 396, 104, 42, "gsoft", rx=21)
             + f'<circle class="dot pulse" cx="{X1 - 28 - 80}" cy="417" r="6"/>' + text(X1 - 28 - 62, 425, "Live", "gtext") + "</g>")
    # Live readings
    tiles = [("CPU", ["7 %", "9 %", "6 %", "12 %"], "s1"), ("Memory", ["10.5 GB", "10.6 GB", "10.4 GB", "10.7 GB"], "s2"),
             ("CPU temperature", ["52 °C", "53 °C", "51 °C", "54 °C"], "s4"),
             ("Network", ["1.2 MB/s", "0.8 MB/s", "2.4 MB/s", "1.1 MB/s"], "s3")]
    tw = (CW - 20) // 2
    for i, (label, values, col) in enumerate(tiles):
        x, y = X0 + (i % 2) * (tw + 20), 628 + (i // 2) * 136
        t = [card(x, y, tw, 116), text(x + 24, y + 40, label, "tlabel")]
        vs, css = cycle(x + 24, y + 92, values, "tval", f"v{i}", period=3.2 + i * 0.4)
        extra_css.append(css)
        sp, css = spark(rng, x + 230, y + 30, tw - 254, 62, colours[col], f"sp{i}", 6 + i)
        extra_css.append(css)
        s.append(f'<g class="{a.pop(on + 0.6 + i * 0.1, off, start=0.85)}">' + "".join(t) + vs + sp + "</g>")
    # What needs you
    s.append(f'<g class="{a.show(on + 1.5, off, dx=-24)}">' + text(X0, 924, "WHAT NEEDS YOU", "cap") + "</g>")
    finds = [("The firewall is off", "Security › Firewall › /etc/ufw", "warn", "Fix"),
             ("Main disk full in about 12 days", "Storage › / (ext4) › 14-day trend", "warn", "Open"),
             ("SSD has 92% of its life left", "Hardware › nvme0 › SMART", "good", "Open")]
    for i, (title, path, kind, btn) in enumerate(finds):
        y = 944 + i * 92
        r = [card(X0, y, CW, 80, rx=14), f'<rect class="{kind}bar" x="{X0}" y="{y}" width="7" height="80" rx="3"/>',
             text(X0 + 32, y + 36, title, "ftitle"), text(X0 + 32, y + 64, path, "fpath")]
        bw = 104 if btn == "Fix" else 112
        bx = X1 - 20 - bw
        if btn == "Fix":
            r.append(card(bx, y + 18, bw, 44, "btn", rx=10) + text(bx + bw / 2, y + 48, "Fix", "btnt", "middle"))
            for t0 in (off - 1.3, off - 0.8):
                k = a.keys([(0, "opacity:0;transform:scale(1)"), (t0, "opacity:0.9;transform:scale(1)"),
                            (t0 + 0.6, "opacity:0;transform:scale(1.5)"), (T, "opacity:0;transform:scale(1.5)")], timing=EASE)
                r.append(f'<rect class="press pp {k}" x="{bx}" y="{y + 18}" width="{bw}" height="44" rx="10" opacity="0"/>')
        else:
            r.append(card(bx, y + 18, bw, 44, "btn2", rx=10) + text(bx + bw / 2, y + 48, btn, "btn2t", "middle"))
        s.append(f'<g class="{a.show(on + 1.7 + i * 0.18, off, dx=70)}">' + "".join(r) + "</g>")
    return "".join(s)


def scene_fix(a, rv):
    on, off = FIX
    s = [heading(a, rv, FIX, "STEP 3 · FIX IT SAFELY", "The firewall is off", "Four clear steps, and you can always undo.")]
    act = [on + 0.6, on + 1.9, on + 3.3, on + 4.9]
    done = act[1:] + [on + 5.6]
    tops = [380, 570, 770, 960]
    s.append(f'<line class="rail" x1="100" y1="{tops[0] + 26}" x2="100" y2="{tops[-1] + 26}"/>')
    s.append(f'<line class="railf gy {a.grow(act[0], done[-1] - act[0], off, "Y", "linear")}" x1="100" y1="{tops[0] + 26}" '
             f'x2="100" y2="{tops[-1] + 26}"/>')
    titles = ["Explain it in plain English", "Show the exact command first", "Apply it, then check again", "Undo it any time"]
    for i, (y0, title) in enumerate(zip(tops, titles)):
        cy = y0 + 26
        st = [f'<circle class="stepn" cx="100" cy="{cy}" r="24"/>' + text(100, cy + 8, str(i + 1), "stepnt", "middle")]
        st.append(f'<g class="{a.pop(act[i], done[i], start=0.6, dur=0.35)}" opacity="0">'
                  f'<circle class="stepa" cx="100" cy="{cy}" r="24"/>' + text(100, cy + 8, str(i + 1), "stepat", "middle") + "</g>")
        st.append(f'<g class="{a.pop(done[i], off, start=0.6, dur=0.35)}"><circle class="stepd" cx="100" cy="{cy}" r="24"/></g>')
        st.append(f'<path class="tick {a.draw(done[i] + 0.1, 0.35, off)}" pathLength="1" d="{check_path(100, cy, 22)}"/>')
        dim = a.keys([(0, "opacity:0.35"), (act[i], "opacity:0.35"), (act[i] + 0.2, "opacity:1"), (T, "opacity:1")])
        st.append(f'<g class="{dim}">' + text(148, cy + 10, title, "stitle") + "</g>")
        body = []
        bx, bw = 148, X1 - 148
        if i == 0:
            body.append(card(bx, y0 + 60, bw, 108))
            body.append(text(bx + 28, y0 + 103, "Any program can take connections from the", "body"))
            body.append(text(bx + 28, y0 + 141, "network. A firewall blocks what you did not allow.", "body"))
        elif i == 1:
            body.append(card(bx, y0 + 60, bw, 124, "code"))
            cmd = "$ sudo ufw enable"
            typed = [(act[i] + 0.25 + j * 0.045, cmd[:j]) for j in range(2, len(cmd) + 1)]
            body.append(swaps(a, bx + 28, y0 + 112, typed, "mono", off=off))
            cur = a.keys([(0, "opacity:0"), (typed[-1][0], "opacity:0"), (typed[-1][0] + 0.01, "opacity:1"), (done[i] + 0.6, "opacity:1"),
                          (done[i] + 0.61, "opacity:0"), (T, "opacity:0")])
            body.append(f'<g class="{cur}" opacity="0"><rect class="cursor blink" x="{bx + 28 + len(cmd) * 19.3 + 6:.0f}" '
                        f'y="{y0 + 86}" width="14" height="32" rx="2"/></g>')
            body.append(text(bx + 28, y0 + 156, "Your password is asked once, for this step only.", "small2"))
        elif i == 2:
            body.append(card(bx, y0 + 60, bw, 108))
            body.append(card(bx + 28, y0 + 92, bw - 56, 12, "ptrack", rx=6))
            body.append(f'<rect class="pfill gx {a.grow(act[i] + 0.2, 0.9, off)}" x="{bx + 28}" y="{y0 + 92}" '
                        f'width="{bw - 56}" height="12" rx="6"/>')
            sh = a.keys([(0, "opacity:0;transform:translateX(0)"), (act[i] + 0.2, "opacity:0.9;transform:translateX(0)"),
                         (act[i] + 1.1, f"opacity:0.9;transform:translateX({bw - 140}px)"), (act[i] + 1.2, "opacity:0"),
                         (T, "opacity:0")])
            body.append(f'<rect class="{sh}" x="{bx + 28}" y="{y0 + 92}" width="84" height="12" rx="6" fill="url(#shine)" opacity="0"/>')
            body.append(f'<path class="tickg {a.draw(act[i] + 1.15, 0.3, off)}" pathLength="1" d="{check_path(bx + 42, y0 + 140, 22)}"/>')
            body.append(f'<g class="{a.show(act[i] + 1.2, off, dx=-16)}">'
                        + text(bx + 66, y0 + 148, "The firewall is on. Checked again: fixed.", "goodt") + "</g>")
            body.append(burst(a, bx + 42, y0 + 140, act[i] + 1.15, off))
            p, _ = pill(X1, y0 + 6, "Health 92 → 97", "gsoft", "gtext", anchor="end")
            body.append(f'<g class="{a.pop(act[i] + 1.35, off)}">' + p + "</g>")
        else:
            body.append(card(bx, y0 + 60, bw, 92))
            body.append(text(bx + 28, y0 + 115, "Every change is kept in the fix log.", "body"))
            body.append(card(X1 - 24 - 120, y0 + 84, 120, 44, "btn2", rx=10)
                        + text(X1 - 24 - 60, y0 + 114, "Undo", "btn2t", "middle"))
        st.append(f'<g class="{a.show(act[i], off, dy=22)}">' + "".join(body) + "</g>")
        s.append("".join(st))
    return "".join(s)


def scene_opt(a, rv):
    on, off = OPT
    s = [heading(a, rv, OPT, "STEP 4 · OPTIMIZE", "Improves what it measured", "Only what this computer needs, each with a preview.")]
    s.append(f'<g class="{a.show(on + 0.2, off, dy=30)}">' + card(X0, 372, CW, 228) + "</g>")
    s.append(f'<g class="{a.show(on + 0.25, off)}"><circle class="rtrack" cx="184" cy="486" r="78" stroke-width="18"/></g>')
    g, w_ = 22 / 34, 12 / 34
    s.append(f'<circle class="rgood {a.draw(on + 0.35, 0.8, off)}" pathLength="1" cx="184" cy="486" r="78" stroke-width="18" '
             f'transform="rotate(-90 184 486)" style="stroke-dasharray:{g:.3f} 1"/>')
    s.append(f'<circle class="rwarn {a.draw(on + 0.95, 0.6, off)}" pathLength="1" cx="184" cy="486" r="78" stroke-width="18" '
             f'transform="rotate({-90 + g * 360:.1f} 184 486)" style="stroke-dasharray:{w_:.3f} 1"/>')
    s.append(count_up(a, 184, 500, 12, on + 0.35, 1.0, "score warnt", "middle", off))
    s.append(f'<g class="{a.show(on + 0.4, off)}">' + text(184, 530, "to improve", "small", "middle") + "</g>")
    s.append(rv.reveal(a, 316, 462, "12 improvements", "big", 40, on + 0.4, off, width=640))
    s.append(f'<g class="{a.show(on + 0.6, off, dy=10)}">' + text(316, 502, "22 of 34 checks passed", "small2") + "</g>")
    p1, w1 = pill(316, 528, "11 with a preview", "asoft", "atext")
    p2, _ = pill(316 + w1 + 12, 528, "22 passed", "gsoft", "gtext")
    s.append(f'<g class="{a.pop(on + 0.8, off)}">' + p1 + "</g>" + f'<g class="{a.pop(on + 0.92, off)}">' + p2 + "</g>")
    items = [("SECURITY", "Install security updates", "6 waiting · they fix known holes"),
             ("STORAGE", "Clean APT package cache", "1.2 GB can be freed"),
             ("POWER", "Let the second graphics card sleep", "awake while idle"),
             ("BATTERY", "Limit battery charge to 80%", "keeps the battery healthy longer")]
    for i, (cat, title, note) in enumerate(items):
        y = 624 + i * 136
        r = [card(X0, y, CW, 120, rx=14), text(X0 + 28, y + 38, cat, "cat"), text(X0 + 28, y + 76, title, "ftitle"),
             text(X0 + 28, y + 104, note, "fpath")]
        bx = X1 - 24 - 136
        r.append(card(bx, y + 38, 136, 46, "btn", rx=10) + text(bx + 68, y + 69, "Preview", "btnt", "middle"))
        s.append(f'<g class="{a.show(on + 0.7 + i * 0.22, off, dx=90)}">' + "".join(r) + "</g>")
    return "".join(s)


def scene_end(a, rv):
    on, off = END
    cx, cy = W / 2, 462
    s = []
    for i in range(3):
        s.append(f'<circle class="ripple pp rip{i}" cx="{cx}" cy="{cy}" r="78"/>')
    s.append(f'<g class="{a.pop(on + 0.1, off, start=0.3, dur=0.6)}">'
             + logo(cx - 72, cy - 72, 144, line_cls=f"logol {a.draw(on + 0.45, 0.7, off)}", extra=' pathLength="1"') + "</g>")
    s.append(rv.reveal(a, cx, 640, "DeeCure", "brandbig", 84, on + 0.35, off, anchor="middle", width=700))
    s.append(f'<g class="{a.show(on + 0.6, off, dy=14)}">' + text(cx, 712, "Deep insight into every layer", "lead", "middle")
             + text(cx, 760, "of your Linux computer.", "lead", "middle") + "</g>")
    chips = ["Free and open source", "No account, no cloud", "Undo for every fix"]
    widths = [round(len(c) * 22 * 0.56 + 36) for c in chips]
    x = (W - sum(widths) - 16 * (len(chips) - 1)) / 2
    for i, (c, w) in enumerate(zip(chips, widths)):
        s.append(f'<g class="{a.pop(on + 0.9 + i * 0.12, off)}">' + pill(round(x), 818, c, "chip", "chipt")[0] + "</g>")
        x += w + 16
    s.append(f'<g class="{a.show(on + 1.3, off, dy=16)}">'
             + text(cx, 980, "For Ubuntu 24.04 and 26.04 LTS", "small2", "middle")
             + card(cx - 290, 1008, 580, 72, "code", rx=14)
             + text(cx, 1054, "github.com/krisskad0/DeeCure", "url", "middle") + "</g>")
    return "".join(s)


def build(theme):
    c = THEMES[theme]
    a = Anim()
    rv = Clips()
    rng = random.Random(7)
    extra_css = []
    s = [f'<defs>'
         f'<radialGradient id="orbA"><stop offset="0" stop-color="{c["accent"]}" stop-opacity="{c["orb"]}"/>'
         f'<stop offset="1" stop-color="{c["accent"]}" stop-opacity="0"/></radialGradient>'
         f'<radialGradient id="orbB"><stop offset="0" stop-color="{c["s3"]}" stop-opacity="{c["orb"] * 0.8:.2f}"/>'
         f'<stop offset="1" stop-color="{c["s3"]}" stop-opacity="0"/></radialGradient>'
         f'<linearGradient id="beam"><stop offset="0" stop-color="{c["accent"]}" stop-opacity="0"/>'
         f'<stop offset="0.5" stop-color="{c["accent"]}" stop-opacity="1"/><stop offset="1" stop-color="{c["accent"]}" stop-opacity="0"/></linearGradient>'
         f'<linearGradient id="glow" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="{c["accent"]}" stop-opacity="0"/>'
         f'<stop offset="0.5" stop-color="{c["accent"]}" stop-opacity="0.16"/><stop offset="1" stop-color="{c["accent"]}" stop-opacity="0"/></linearGradient>'
         f'<linearGradient id="shine"><stop offset="0" stop-color="#ffffff" stop-opacity="0"/>'
         f'<stop offset="0.5" stop-color="#ffffff" stop-opacity="0.75"/><stop offset="1" stop-color="#ffffff" stop-opacity="0"/></linearGradient>'
         f'<clipPath id="frame"><rect x="0" y="0" width="{W}" height="{H}" rx="28"/></clipPath>'
         f'</defs>',
         f'<g clip-path="url(#frame)"><rect class="bg" x="0" y="0" width="{W}" height="{H}"/>'
         f'<circle class="orbA" cx="880" cy="260" r="460" fill="url(#orbA)"/>'
         f'<circle class="orbB" cx="160" cy="1120" r="440" fill="url(#orbB)"/></g>']

    # The header: always there.
    s.append(logo(X0, 44, 56) + text(X0 + 76, 84, "DeeCure", "brand"))
    s.append(text(X1, 82, "Linux health, every layer", "tag", "end"))
    s.append(f'<line class="hr" x1="{X0}" y1="132" x2="{X1}" y2="132"/>')

    colours = {k: c[k] for k in ("s1", "s2", "s3", "s4")}
    scenes = [("hook", HOOK, scene_hook(a, rv)), ("deep", DEEP, scene_deep(a, rv)),
              ("view", VIEW, scene_view(a, rv, rng, colours, extra_css)), ("fix", FIX, scene_fix(a, rv)),
              ("opt", OPT, scene_opt(a, rv)), ("end", END, scene_end(a, rv))]
    for name, span, body in scenes:
        op = "" if name == STATIC else ' opacity="0"'
        s.append(f'<g class="{a.scene(span)}"{op}>{body}</g>')

    # The step bar: one part per step, filled while that step plays.
    steps = [("Look deep", DEEP), ("See it", VIEW), ("Fix it", FIX), ("Optimize", OPT)]
    gap = 16
    sw = (CW - gap * 3) / 4
    bar = [f'<g class="{a.show(DEEP[0], OPT[1] - 0.4, fade=0.4, dy=20)}">']
    static_at = [n for n, _, _ in scenes].index(STATIC)
    for i, (label, (on, off)) in enumerate(steps):
        x = X0 + i * (sw + gap)
        full = "" if i + 1 <= static_at else ' transform="scale(0 1)"'
        bar.append(card(x, 1252, sw, 8, "ptrack", rx=4))
        bar.append(f'<rect class="pfill gx {a.grow(on, off - on, OPT[1], timing="linear")}" x="{x:.1f}" y="1252" '
                   f'width="{sw:.1f}" height="8" rx="4"{full}/>')
        lit = a.keys([(0, f"fill:{c['muted']}"), (on, f"fill:{c['muted']}"), (on + 0.2, f"fill:{c['ink']}"),
                      (T, f"fill:{c['ink']}")])
        bar.append(text(round(x), 1296, label, f"slabel {lit}"))
    bar.append("</g>")
    s.append("".join(bar))

    css = f"""
text{{font-family:{SANS};fill:{c['ink']}}}
.bg{{fill:{c['bg']}}}
.orbA{{animation:orbA 13s ease-in-out infinite alternate}}@keyframes orbA{{to{{transform:translate(-260px,180px)}}}}
.orbB{{animation:orbB 17s ease-in-out infinite alternate}}@keyframes orbB{{to{{transform:translate(300px,-240px)}}}}
.pp{{transform-box:fill-box;transform-origin:center}}
.gx{{transform-box:fill-box;transform-origin:left center}}
.gy{{transform-box:fill-box;transform-origin:center top}}
.dr{{stroke-dasharray:1 1}}
.logo{{fill:#2563c9}}
.logol{{fill:none;stroke:#ffffff;stroke-linecap:round;stroke-linejoin:round}}
.brand{{font-size:36px;font-weight:700}}
.brandbig{{font-size:84px;font-weight:700}}
.tag{{font-size:24px;fill:{c['muted']}}}
.hr{{stroke:{c['line']};stroke-width:2}}
.kick{{font-size:24px;font-weight:700;letter-spacing:0.08em;fill:{c['accent']}}}
.h1{{font-size:56px;font-weight:700}}
.sub{{font-size:29px;fill:{c['ink2']}}}
.hero{{font-size:76px;font-weight:700}}
.stat{{font-size:60px;font-weight:700;fill:{c['accent']};font-variant-numeric:tabular-nums}}
.accent{{fill:{c['accent']}}}
.accentbar{{fill:{c['accent']}}}
.ekg{{fill:none;stroke:{c['accent']};stroke-width:4;stroke-linecap:round;stroke-linejoin:round;opacity:0.35}}
.ekgp{{fill:none;stroke:{c['accent']};stroke-width:6;stroke-linecap:round;stroke-linejoin:round;stroke-dasharray:0.07 1;
  animation:ekgp 2.4s linear infinite}}@keyframes ekgp{{0%{{stroke-dashoffset:0.07}}100%{{stroke-dashoffset:-1}}}}
.uline{{fill:none;stroke:{c['warn']};stroke-width:7;stroke-linecap:round}}
.lead{{font-size:38px;fill:{c['ink2']}}}
.lead2{{font-size:38px;font-weight:600}}
.card{{fill:{c['panel']};stroke:{c['line']};stroke-width:2}}
.scan{{fill:{c['accent_soft']};stroke:{c['accent']};stroke-width:2.5}}
.code{{fill:{c['panel2']};stroke:{c['line2']};stroke-width:2}}
.badge{{fill:{c['accent']}}}
.badget{{font-size:20px;font-weight:700;fill:{c['accent_ink']}}}
.rname{{font-size:27px;font-weight:600}}
.rhint{{font-family:{MONO};font-size:19px;fill:{c['muted']}}}
.rcount{{font-size:21px;fill:{c['muted']}}}
.count{{font-size:25px;font-weight:600;fill:{c['ink2']};font-variant-numeric:tabular-nums}}
.hist{{font-size:22px;font-style:italic;fill:{c['muted']}}}
.gsoft{{fill:{c['good_soft']}}}.gtext{{font-size:22px;font-weight:700;fill:{c['good_ink']}}}
.wsoft{{fill:{c['warn_soft']}}}.wtext{{font-size:22px;font-weight:700;fill:{c['warn_ink']}}}
.asoft{{fill:{c['accent_soft']}}}.atext{{font-size:22px;font-weight:700;fill:{c['accent']}}}
.chip{{fill:{c['panel']};stroke:{c['line2']};stroke-width:2}}.chipt{{font-size:22px;font-weight:600;fill:{c['ink2']}}}
.rtrack{{fill:none;stroke:{c['track']}}}
.rgood{{fill:none;stroke:{c['good']}}}
.rwarn{{fill:none;stroke:{c['warn']}}}
.score{{font-size:56px;font-weight:700;font-variant-numeric:tabular-nums}}
.warnt{{fill:{c['warn_ink']}}}
.small{{font-size:20px;fill:{c['muted']}}}
.small2{{font-size:25px;fill:{c['muted']}}}
.good{{font-size:42px;font-weight:700;fill:{c['good_ink']}}}
.big{{font-size:40px;font-weight:700}}
.big2{{font-size:31px;font-weight:600}}
.dot{{fill:{c['good']}}}
.pulse{{animation:pl 1.6s ease-in-out infinite}}@keyframes pl{{0%,100%{{opacity:1}}50%{{opacity:0.25}}}}
.blink{{animation:bl 0.9s steps(1,end) infinite}}@keyframes bl{{0%{{opacity:1}}50%{{opacity:0}}}}
.cursor{{fill:{c['accent']}}}
.tlabel{{font-size:23px;fill:{c['muted']}}}
.tval{{font-size:40px;font-weight:700;font-variant-numeric:tabular-nums}}
.cap{{font-size:22px;font-weight:700;letter-spacing:0.08em;fill:{c['accent']}}}
.cat{{font-size:20px;font-weight:700;letter-spacing:0.08em;fill:{c['muted']}}}
.warnbar{{fill:{c['warn']}}}.goodbar{{fill:{c['good']}}}
.ftitle{{font-size:29px;font-weight:600}}
.fpath{{font-family:{MONO};font-size:21px;fill:{c['muted']}}}
.btn{{fill:{c['accent']}}}.btnt{{font-size:23px;font-weight:700;fill:{c['accent_ink']}}}
.btn2{{fill:{c['panel']};stroke:{c['line2']};stroke-width:2}}.btn2t{{font-size:23px;font-weight:600}}
.press{{fill:none;stroke:{c['accent']};stroke-width:3}}
.rail{{stroke:{c['line2']};stroke-width:3}}
.railf{{stroke:{c['good']};stroke-width:3}}
.stepn{{fill:{c['panel']};stroke:{c['line2']};stroke-width:2.5}}.stepnt{{font-size:23px;font-weight:700;fill:{c['muted']}}}
.stepa{{fill:{c['accent']}}}.stepat{{font-size:23px;font-weight:700;fill:{c['accent_ink']}}}
.stepd{{fill:{c['good']}}}
.tick{{fill:none;stroke:#ffffff;stroke-width:4;stroke-linecap:round;stroke-linejoin:round}}
.tickg{{fill:none;stroke:{c['good']};stroke-width:4;stroke-linecap:round;stroke-linejoin:round}}
.bdot{{fill:{c['good']}}}.bdot2{{fill:{c['warn']}}}
.stitle{{font-size:31px;font-weight:700}}
.body{{font-size:27px;fill:{c['ink2']}}}
.mono{{font-family:{MONO};font-size:32px;font-weight:600}}
.goodt{{font-size:26px;font-weight:600;fill:{c['good_ink']}}}
.ptrack{{fill:{c['track']}}}
.pfill{{fill:{c['accent']}}}
.ripple{{fill:none;stroke:{c['accent']};stroke-width:3;opacity:0}}
.rip0,.rip1,.rip2{{animation:rip 2.4s ease-out infinite}}.rip1{{animation-delay:0.8s}}.rip2{{animation-delay:1.6s}}
@keyframes rip{{0%{{opacity:0.45;transform:scale(1)}}100%{{opacity:0;transform:scale(2.6)}}}}
.url{{font-family:{MONO};font-size:30px;font-weight:600;fill:{c['accent']}}}
.slabel{{font-size:22px;font-weight:600;fill:{c['ink']}}}
{''.join(a.css)}
{''.join(extra_css)}
@media (prefers-reduced-motion:reduce){{*{{animation:none!important}}}}
"""
    desc = ("A short story of what DeeCure does, in the app's own look. Is your Linux computer really healthy? "
            f"DeeCure checks {len(LAYERS)} layers, with more than {READINGS} readings, {WARNINGS} warnings, and {FIXES} fixes "
            "with undo. Step 1, look deep: a scan beam walks down the layers: apps and processes; services, "
            "start-up, and jobs; containers; packages and updates; network; security, accounts, and privacy; "
            "storage and backups; kernel, drivers, and firmware; power, battery, and sleep; and hardware and "
            "devices, and finds 2 things to look at. Step 2, see it at a glance: a health score of 92, live CPU, "
            "memory, temperature, and network readings, and what needs you: the firewall is off, the main disk "
            "is full in about 12 days, and the SSD has 92% of its life left. Step 3, fix it safely: the problem "
            "is explained in plain English, the exact command (sudo ufw enable) is shown first, the fix is "
            "applied and checked again, the score goes from 92 to 97, and it can be undone. Step 4, optimize: "
            "12 improvements this computer needs, each with a preview. DeeCure is free and open source, for "
            "Ubuntu 24.04 and 26.04 LTS.")
    return (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}" width="{W}" height="{H}" '
            f'role="img" aria-labelledby="title desc">\n'
            f'<title id="title">What DeeCure does: look deep, see it, fix it, optimize</title>\n'
            f'<desc id="desc">{desc}</desc>\n'
            f'<style>{css}</style>\n' + "\n".join(s) + "\n</svg>\n")


def main():
    check = "--check" in sys.argv[1:]
    stale = []
    for theme in THEMES:
        path = OUT / f"deecure-deep-look-{theme}.svg"
        text_out = build(theme)
        if check:
            if not path.exists() or path.read_text() != text_out:
                stale.append(path)
        else:
            path.write_text(text_out)
            print(f"wrote {path.relative_to(OUT.parent.parent.parent)} ({len(text_out) // 1024} KB)")
    if stale:
        sys.exit("out of date, run scripts/gen-deep-look.py: " + ", ".join(p.name for p in stale))


if __name__ == "__main__":
    main()
