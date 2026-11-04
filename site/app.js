// DeeCure landing page: copy buttons, screenshot tabs, and download links for the newest release.
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

  // Screenshot tabs, with arrow keys between them.
  const tabs = [...document.querySelectorAll(".tabs [role=tab]")];
  const img = document.getElementById("shot-img");
  const cap = document.getElementById("shot-cap");
  const show = (tab) => {
    for (const t of tabs) {
      t.setAttribute("aria-selected", String(t === tab));
      t.tabIndex = t === tab ? 0 : -1;
    }
    img.style.opacity = "0.3";
    const next = new Image();
    next.onload = next.onerror = () => {
      img.src = next.src;
      img.width = next.naturalWidth || img.width;
      img.height = next.naturalHeight || img.height;
      img.alt = tab.dataset.alt;
      img.style.opacity = "1";
    };
    next.src = `assets/screens/${tab.dataset.shot}.png`;
    cap.textContent = tab.dataset.cap;
  };
  tabs.forEach((tab, i) => {
    tab.tabIndex = i === 0 ? 0 : -1;
    tab.addEventListener("click", () => show(tab));
    tab.addEventListener("keydown", (e) => {
      const step = e.key === "ArrowRight" ? 1 : e.key === "ArrowLeft" ? -1 : 0;
      if (!step) return;
      e.preventDefault();
      const next = tabs[(i + step + tabs.length) % tabs.length];
      next.focus();
      show(next);
    });
  });

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
