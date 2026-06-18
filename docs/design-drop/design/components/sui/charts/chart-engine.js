// chart-engine.js — faithful JS port of the real sui-charts engine
// (madvib/Ship@job/sui-framework: engine/normalize.ts + geometry-cartesian.ts +
// the createChartData 0..1 domain scaling). So "existing" charts render against
// the TRUE contract, not a toy. ResizeObserver viewport is omitted for the
// design harness (fixed box). Loaded via the "@charts/engine" import-map entry.

const toVal = (v) => { if (v === null || v === undefined) return null; const n = typeof v === 'number' ? v : Number(v); return Number.isFinite(n) ? n : null; };
const yOf = (p) => (p && typeof p === 'object' && 'y' in p) ? p.y : p;
const xOf = (p, i) => (p && typeof p === 'object' && 'x' in p) ? p.x : i;

function fromArray(input) {
  if (!input.length) return [];
  const isSeries = (v) => v && typeof v === 'object' && typeof v.name === 'string' && Array.isArray(v.data);
  const isSlice = (v) => v && typeof v === 'object' && typeof v.label === 'string' && (typeof v.value === 'number' || v.value === null);
  if (input.every(isSeries)) return input.map((s) => ({ name: s.name, tone: s.tone, data: s.data.map((pv) => (pv && typeof pv === 'object' && 'x' in pv && 'y' in pv) ? { x: pv.x, y: toVal(pv.y) } : toVal(pv)) }));
  if (input.every(isSlice)) { const tone = (input.find((s) => s.tone) || {}).tone; return [{ name: 'series', tone, data: input.map((s) => ({ x: s.label, y: toVal(s.value) })) }]; }
  if (input.every((r) => Array.isArray(r))) return input.map((row, i) => ({ name: 'row-' + i, data: row.map(toVal) }));
  return [{ name: 'series', data: input.map(toVal) }];
}

export function normalizeSeries(input) {
  if (input == null) return [];
  if (typeof input === 'string') {
    const t = input.trim(); if (!t) return [];
    if (t[0] === '[' || t[0] === '{') { let p; try { p = JSON.parse(t); } catch { return []; } return Array.isArray(p) ? fromArray(p) : []; }
    const nums = t.split(',').map((s) => { const x = s.trim(); if (x === '') return null; const n = parseFloat(x); return Number.isFinite(n) ? n : null; });
    return nums.some((n) => n != null) ? [{ name: 'series', data: nums }] : [];
  }
  if (Array.isArray(input)) return fromArray(input);
  return [];
}

// createChartData.normalized() — scale y to 0..1 over the shared domain.
export function normalized(series, { yMin = null, yMax = null } = {}) {
  let lo = Infinity, hi = -Infinity;
  for (const s of series) for (const p of s.data) { const y = yOf(p); if (y == null) continue; if (y < lo) lo = y; if (y > hi) hi = y; }
  if (!isFinite(lo)) { lo = 0; hi = 1; }
  const dlo = yMin == null ? Math.min(0, lo) : yMin;
  const dhi = yMax == null ? hi : yMax;
  const rng = (dhi - dlo) || 1;
  return series.map((s) => ({ name: s.name, tone: s.tone, data: s.data.map((p, i) => { const y = yOf(p); return { x: xOf(p, i), raw: y, y: y == null ? null : (y - dlo) / rng }; }) }));
}

export function toneColor(tone) {
  const m = { primary: '--primary', accent: '--accent', info: '--info', success: '--success', warning: '--warning', destructive: '--destructive', muted: '--muted-foreground' };
  return 'var(' + (m[tone] || '--primary') + ')';
}

// extent normalization (min..max) for trend charts (sparkline/line).
export function normalizedExtent(series) {
  let lo = Infinity, hi = -Infinity;
  for (const s of series) for (const p of s.data) { const y = (p && typeof p === 'object' && 'y' in p) ? p.y : p; if (y == null) continue; if (y < lo) lo = y; if (y > hi) hi = y; }
  if (!isFinite(lo)) { lo = 0; hi = 1; }
  return normalized(series, { yMin: lo, yMax: hi });
}

const clamp01 = (x) => x < 0 ? 0 : x > 1 ? 1 : x;

// geometry-cartesian.lineGeometry / areaGeometry — faithful ports (null-gap split).
export function lineGeometry(series, box, pad = 2) {
  const out = [];
  for (const s of series) {
    const color = toneColor(s.tone); const len = s.data.length;
    const runs = []; let cur = [];
    s.data.forEach((p, i) => { const y = (p && typeof p === 'object' && 'y' in p) ? p.y : p; if (y == null) { if (cur.length) runs.push(cur); cur = []; } else cur.push(i); });
    if (cur.length) runs.push(cur);
    let d = ''; const pts = [];
    for (const run of runs) {
      const segs = run.map((idx) => {
        const cx = len <= 1 ? box.w / 2 : (idx / (len - 1)) * box.w;
        const innerH = box.h - pad * 2;
        const y = (s.data[idx] && typeof s.data[idx] === 'object') ? s.data[idx].y : s.data[idx];
        const cy = pad + innerH - clamp01(y) * innerH;
        return { idx, cx, cy };
      });
      d += segs.map((p, k) => (k === 0 ? 'M' : 'L') + p.cx.toFixed(2) + ',' + p.cy.toFixed(2)).join(' ') + ' ';
      for (const seg of segs) pts.push({ id: s.name + '-' + seg.idx, cx: +seg.cx.toFixed(2), cy: +seg.cy.toFixed(2) });
    }
    if (!d.trim()) continue;
    out.push({ name: s.name, strokeD: d.trim(), color, pts });
  }
  return out;
}
export function areaGeometry(series, box, pad = 2) {
  return lineGeometry(series, box, pad).map((lp) => {
    const baseline = box.h - pad; let fillD = '';
    if (lp.pts.length) { const f = lp.pts[0], l = lp.pts[lp.pts.length - 1]; fillD = lp.strokeD + ' L' + l.cx.toFixed(2) + ',' + baseline.toFixed(2) + ' L' + f.cx.toFixed(2) + ',' + baseline.toFixed(2) + ' Z'; }
    return Object.assign({}, lp, { fillD });
  });
}

// geometry-cartesian.barGeometry — faithful port.
export function barGeometry(series, box, opts = {}) {
  const first = series[0]; if (!first) return [];
  const { horizontal = false, gap = 4, radius = 3 } = opts;
  const color = toneColor(opts.tone); const labels = opts.labels || [];
  const n = first.data.length; if (!n) return [];
  const clamp = (x) => x < 0 ? 0 : x > 1 ? 1 : x;
  const bars = [];
  for (let i = 0; i < n; i++) {
    const pt = first.data[i]; if (pt.y == null) continue;
    const ratio = clamp(pt.y);
    let x, y, bw, bh;
    if (horizontal) { bh = (box.h - gap * (n - 1)) / n; y = i * (bh + gap); bw = ratio * box.w; x = 0; }
    else { bw = (box.w - gap * (n - 1)) / n; x = i * (bw + gap); bh = ratio * box.h; y = box.h - bh; }
    const raw = pt.raw == null ? '' : String(pt.raw);
    bars.push({ id: 'b' + i, x: +x.toFixed(2), y: +y.toFixed(2), w: +Math.max(0, bw).toFixed(2), h: +Math.max(0, bh).toFixed(2), rx: radius, ry: radius, color, raw, title: (labels[i] != null ? labels[i] : raw) + ': ' + raw });
  }
  return bars;
}
