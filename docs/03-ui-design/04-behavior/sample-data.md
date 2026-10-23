# Sample data generator (exact)

[Docs](../../README.md) › [UI design](../README.md) › [Behavior](README.md)

All live charts and sparklines use this deterministic generator, so the clone matches the prototype point for point.

```js
const NOW = new Date(2026, 9, 21, 21, 34);
const RANGES = { '1h':{n:60, step:1/60}, '24h':{n:96, step:0.25}, '7d':{n:168, step:1}, '30d':{n:120, step:6} }; // step in hours
function rng(seed){ let s = (seed * 9301 + 49297) % 2147483647; if (s <= 0) s += 2147483646;
  return () => (s = s * 16807 % 2147483647) / 2147483647; }
const tAt = (i, n, step) => new Date(NOW - (n - 1 - i) * step * 3600e3);
function gen([seed, base, amp, noise, lo = 0, hi = 100, end]){
  const R = RANGES[S.range], r = rng(seed * 31 + R.n), out = []; let w = 0;
  for (let i = 0; i < R.n; i++){
    const t = tAt(i, R.n, R.step), hr = t.getHours() + t.getMinutes() / 60;
    w = w * 0.72 + (r() - 0.5) * noise;
    out.push(Math.max(lo, Math.min(hi, base + amp * Math.sin((hr - 9) / 24 * 2 * Math.PI) + w)));
  }
  if (end != null){ out[R.n - 1] = end; out[R.n - 2] = (out[R.n - 2] + end) / 2; }
  return out;
}
function tLab(i, n, tooltip){ // axis label (tooltip=false) or tooltip header (true)
  const d = tAt(i, n, RANGES[S.range].step), hh = pad2(d.getHours()) + ':' + pad2(d.getMinutes());
  const wd = d.toLocaleDateString('en-GB', {weekday:'short'}), dm = d.toLocaleDateString('en-GB', {day:'numeric', month:'short'});
  if (S.range === '1h') return hh;
  if (S.range === '24h') return tooltip ? `${wd} ${hh}` : hh;
  if (S.range === '7d') return tooltip ? `${wd} ${hh}` : wd;
  return tooltip ? `${dm} ${hh}` : dm;
}
// Heatmaps: heatData(seed, rows, cols, base, spikes): cell = max(0, round((rng()-0.55)*base)), then spikes [row, col, value] override.
function heatData(seed, rows, cols, base, spikes = []){ const r = rng(seed);
  return Array.from({length:rows}, (_, ri) => Array.from({length:cols}, (_, ci) => {
    let v = Math.max(0, Math.round((r() - 0.55) * base)); spikes.forEach(s => { if (s[0] === ri && s[1] === ci) v = s[2]; }); return v; })); }
// 30-day disk history: r = rng(11); for i in 0..29: 74 + i*0.69 + (r()-0.5)*1.1; last value = 224.2/238*100.
// 30-day labels: dates 22 Sep … 21 Oct formatted 'D Mon' (en-GB).
```

The heatmap matrices in the page map are the computed outputs, so you can paste them directly.
