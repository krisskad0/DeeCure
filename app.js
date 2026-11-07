// DeeCure landing page: copy buttons, and download links for the newest release.
// The page works without this script; it only adds comfort and the exact file links.
(() => {
  "use strict";

  // Copy buttons: copy the command next to them.
  for (const button of document.querySelectorAll("[data-copy]")) {
    button.addEventListener("click", async () => {
      const text = document.getElementById(button.dataset.copy)?.textContent ?? "";
      try {
        await navigator.clipboard.writeText(text);
        button.textContent = "Copied";
        button.classList.add("done");
      } catch {
        // No clipboard access: select the text so Ctrl+C works.
        const range = document.createRange();
        range.selectNodeContents(document.getElementById(button.dataset.copy));
        const selection = window.getSelection();
        selection.removeAllRanges();
        selection.addRange(range);
        button.textContent = "Press Ctrl+C";
      }
      setTimeout(() => {
        button.textContent = "Copy";
        button.classList.remove("done");
      }, 2000);
    });
  }

  // The newest release: the highest version, not the first in GitHub's list (it once listed
  // 0.1.0-alpha.9 before 0.1.0-alpha.10). Same order as install.sh and the in-app updater.
  const version = (tag) => {
    const [core, ...rest] = String(tag).replace(/^v/, "").split("-");
    const numbers = core.split(".").map(Number);
    if (numbers.some(Number.isNaN)) return null;
    const pre = rest.join("-");
    return { numbers, pre: pre ? pre.split(".") : [] };
  };
  const compare = (a, b) => {
    for (let i = 0; i < Math.max(a.numbers.length, b.numbers.length); i++) {
      const d = (a.numbers[i] ?? 0) - (b.numbers[i] ?? 0);
      if (d) return d;
    }
    // A stable release beats any pre-release of the same version.
    if (!a.pre.length || !b.pre.length) return b.pre.length - a.pre.length;
    for (let i = 0; i < Math.max(a.pre.length, b.pre.length); i++) {
      const x = a.pre[i], y = b.pre[i];
      if (x === undefined) return -1;
      if (y === undefined) return 1;
      const nx = /^\d+$/.test(x), ny = /^\d+$/.test(y);
      if (nx && ny) { if (+x !== +y) return +x - +y; }
      else if (nx !== ny) return nx ? -1 : 1;
      else if (x !== y) return x < y ? -1 : 1;
    }
    return 0;
  };
  const size = (bytes) => `${(bytes / 1048576).toFixed(1)} MB`;

  fetch("https://api.github.com/repos/krisskad0/DeeCure/releases?per_page=30", { headers: { Accept: "application/vnd.github+json" } })
    .then((r) => (r.ok ? r.json() : Promise.reject(r.status)))
    .then((list) => {
      const releases = list.filter((r) => !r.draft && version(r.tag_name));
      const stable = releases.filter((r) => !r.prerelease);
      const pool = stable.length ? stable : releases;
      if (!pool.length) return;
      const newest = pool.reduce((best, r) => (compare(version(r.tag_name), version(best.tag_name)) > 0 ? r : best));
      const name = newest.tag_name.replace(/^v/, "");
      const kind = newest.prerelease ? "pre-release" : "release";

      for (const el of document.querySelectorAll("[data-release]")) {
        el.innerHTML = "";
        const a = document.createElement("a");
        a.href = newest.html_url;
        a.textContent = `Version ${name} (${kind})`;
        el.append(a);
      }
      for (const el of document.querySelectorAll("[data-release-long]")) {
        const date = newest.published_at ? new Date(newest.published_at).toLocaleDateString(undefined, { day: "numeric", month: "long", year: "numeric" }) : "";
        el.textContent = `Newest: ${name} (${kind}${date ? `, ${date}` : ""}).`;
      }

      const wanted = [
        ["deecure-desktop_", "Desktop app (.deb)"],
        ["deecure_", "Agent and command (.deb)"],
        ["SHA256SUMS", "Checksums (SHA256SUMS)"],
      ];
      const files = document.querySelector("[data-files]");
      const items = wanted
        .map(([prefix, label]) => [newest.assets.find((a) => a.name.startsWith(prefix)), label])
        .filter(([asset]) => asset)
        .map(([asset, label]) => {
          const li = document.createElement("li");
          const a = document.createElement("a");
          a.className = "btn ghost small";
          a.href = asset.browser_download_url;
          a.textContent = label;
          const small = document.createElement("small");
          small.textContent = asset.name === "SHA256SUMS" ? "text" : size(asset.size);
          a.append(small);
          li.append(a);
          return li;
        });
      if (items.length) files.prepend(...items);
    })
    .catch(() => {
      // Offline or rate-limited: the "All releases on GitHub" link stays.
    });
})();

// The animated tour in "What it does": plays the five steps while it is on screen, and the
// step list below it doubles as its chapters. CSS draws and animates; this only picks the step.
(() => {
  "use strict";
  const demo = document.querySelector(".demo");
  const flow = document.querySelector("#features .flow");
  if (!demo || !flow) return;
  const items = [...flow.children];
  const num = demo.querySelector("[data-score]");
  const pause = demo.querySelector(".demo-pause");
  const still = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
  // Per step: how long it shows, and the health score [from, to, when it moves].
  const steps = [
    { ms: 4200, score: [0, 86, 300] },
    { ms: 4400, score: [86, 86, 0] },
    { ms: 4600, score: [86, 86, 0] },
    { ms: 5400, score: [86, 94, 3100] },
    { ms: 4600, score: [94, 94, 0] },
  ];
  let current = 3, timer = 0, frame = 0, playing = !still, seen = false, started = false;

  const setScore = (v) => {
    num.textContent = Math.round(v);
    demo.style.setProperty("--s", v.toFixed(1));
  };
  const moveScore = ([from, to, delay]) => {
    cancelAnimationFrame(frame);
    setScore(from);
    if (from === to || still) return setScore(to);
    const start = performance.now() + delay;
    const tick = (now) => {
      const t = Math.min(1, Math.max(0, (now - start) / 1100));
      setScore(from + (to - from) * (1 - (1 - t) ** 3));
      if (t < 1) frame = requestAnimationFrame(tick);
    };
    frame = requestAnimationFrame(tick);
  };
  const run = () => playing && seen && !document.hidden;
  const show = (i) => {
    current = i;
    clearTimeout(timer);
    // Restart the CSS animations, even when the same step is picked again.
    demo.removeAttribute("data-step");
    items.forEach((li) => li.classList.remove("on"));
    void demo.offsetWidth;
    demo.dataset.step = String(i + 1);
    demo.style.setProperty("--dur", `${steps[i].ms}ms`);
    demo.classList.toggle("paused", !run());
    flow.style.setProperty("--dur", `${steps[i].ms}ms`);
    items.forEach((li, k) => {
      li.classList.toggle("on", k === i);
      li.setAttribute("aria-current", k === i ? "step" : "false");
    });
    moveScore(steps[i].score);
    if (run()) timer = setTimeout(() => show((i + 1) % steps.length), steps[i].ms);
  };

  flow.classList.add("tour");
  items.forEach((li, k) => {
    li.tabIndex = 0;
    li.setAttribute("role", "button");
    li.addEventListener("click", () => show(k));
    li.addEventListener("keydown", (e) => {
      if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        show(k);
      }
    });
  });
  pause.hidden = still;
  pause.addEventListener("click", () => {
    playing = !playing;
    pause.setAttribute("aria-pressed", String(!playing));
    pause.setAttribute("aria-label", playing ? "Pause the animation" : "Play the animation");
    show(current);
  });
  // Only play while the tour is on screen and the tab is open.
  new IntersectionObserver(([entry]) => {
    const was = seen;
    seen = entry.isIntersecting;
    if (seen && !was) {
      // The first time from step 1; after scrolling away and back, where it was.
      show(started || still ? current : 0);
      started = true;
    } else if (!seen) {
      clearTimeout(timer);
      demo.classList.add("paused");
    }
  }, { threshold: 0.35 }).observe(demo);
  document.addEventListener("visibilitychange", () => (document.hidden ? (clearTimeout(timer), demo.classList.add("paused")) : seen && show(current)));
  show(current);
})();

// "How deep it looks": the scan walks down the eight layers. In each layer it lights the checks one
// by one; a check that leads to a sentence on the card shows that sentence. It plays only while the
// section is on screen. Without the script, or with reduced motion, the finished scan stays.
(() => {
  "use strict";
  const root = document.querySelector(".dc");
  if (!root) return;
  const layers = [...root.querySelectorAll(".dc-layer")];
  const chips = layers.map((l) => [...l.querySelectorAll(".dc-chips li")]);
  const finds = [...root.querySelectorAll(".dc-finds li")];
  const total = chips.flat().length;
  const count = root.querySelector(".dc-count");
  const status = root.querySelector(".dc-status-t");
  const num = root.querySelector(".dc-num");
  const pause = root.querySelector(".dc-pause");
  const DONE = status.textContent;
  // The totals come from the page itself, so they stay right when a check is added.
  root.querySelectorAll(".dc-total").forEach((el) => (el.textContent = total));
  count.textContent = total;

  if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
    pause.hidden = true;
    return;
  }
  root.classList.add("anim");

  const SCORE = 92, START = 300, CHIP = 38, LAYER = 420, HOLD = 5000;
  let timers = [], frame = 0, score = 0, done = 0, playing = true, seen = false;
  const at = (ms, fn) => timers.push(setTimeout(fn, ms));
  const setDone = (v) => {
    done = v;
    count.textContent = v;
    root.style.setProperty("--done", (v / total).toFixed(3));
  };
  const setScore = (v) => {
    score = v;
    num.textContent = Math.round(v);
    root.style.setProperty("--score", v.toFixed(1));
  };
  const tween = (to, ms) => {
    cancelAnimationFrame(frame);
    const from = score, t0 = performance.now();
    const tick = (now) => {
      // A frame's time can be a little before t0, so keep t between 0 and 1.
      const t = Math.max(0, Math.min(1, (now - t0) / ms));
      setScore(from + (to - from) * (1 - (1 - t) ** 3));
      if (t < 1) frame = requestAnimationFrame(tick);
    };
    frame = requestAnimationFrame(tick);
  };
  const stop = () => {
    timers.forEach(clearTimeout);
    timers = [];
    cancelAnimationFrame(frame);
  };

  const cycle = () => {
    stop();
    root.classList.remove("ready", "fade");
    layers.forEach((l) => l.classList.remove("scan", "done"));
    chips.flat().forEach((c) => c.classList.remove("on"));
    finds.forEach((f) => f.classList.remove("on"));
    setDone(0);
    setScore(0);
    let t = START;
    layers.forEach((layer, i) => {
      const name = layer.querySelector("h3").textContent;
      at(t, () => {
        layer.classList.add("scan");
        status.textContent = `Layer ${i + 1} of ${layers.length}: ${name}`;
      });
      chips[i].forEach((chip, j) =>
        at(t + 150 + j * CHIP, () => {
          chip.classList.add("on");
          setDone(done + 1);
          if (chip.dataset.find) finds[+chip.dataset.find]?.classList.add("on");
        })
      );
      t += 150 + chips[i].length * CHIP + 150;
      at(t, () => {
        layer.classList.replace("scan", "done");
        tween((SCORE * (i + 1)) / layers.length, 450);
      });
      t += LAYER - 150;
    });
    at(t, () => {
      root.classList.add("ready");
      status.textContent = DONE;
    });
    at(t + HOLD, () => root.classList.add("fade"));
    at(t + HOLD + 700, cycle);
  };
  const update = () => {
    const run = playing && seen && !document.hidden;
    root.classList.toggle("paused", !run);
    if (run && !timers.length) cycle();
    if (!run) stop();
  };

  pause.addEventListener("click", () => {
    playing = !playing;
    pause.setAttribute("aria-pressed", String(!playing));
    pause.setAttribute("aria-label", playing ? "Pause the animation" : "Play the animation");
    update();
  });
  // The section is taller than most screens, so "on screen" means any part of it is in the middle.
  new IntersectionObserver(([entry]) => {
    seen = entry.isIntersecting;
    update();
  }, { rootMargin: "-20% 0px -20% 0px" }).observe(root);
  document.addEventListener("visibilitychange", update);
})();
