// sparkline.controller.js — geometry for sui-sparkline. Builds line/area paths
// in a 240×80 viewBox from `data` (JSON number[]). Pure; no daemon.
import { controller, derived } from '@ship/sui-runtime';

const parse = (raw) => {
  try { const a = JSON.parse(raw); return Array.isArray(a) ? a.map(Number).filter((n) => !Number.isNaN(n)) : []; }
  catch { return []; }
};

export default controller({
  scope: ({ data }) => {
    const pts = derived(() => {
      const arr = parse(data());
      const n = arr.length;
      if (n < 2) return [];
      const W = 240, H = 80, pad = 5;
      const min = Math.min(...arr), max = Math.max(...arr), rng = (max - min) || 1;
      return arr.map((v, i) => ({
        x: (i / (n - 1)) * W,
        y: pad + (1 - (v - min) / rng) * (H - 2 * pad),
      }));
    });
    const path = derived(() => pts().map((p, i) => (i ? 'L' : 'M') + p.x.toFixed(1) + ' ' + p.y.toFixed(1)).join(' '));
    const area = derived(() => {
      const p = pts();
      if (!p.length) return '';
      return 'M' + p[0].x.toFixed(1) + ' 80 L' + p.map((q) => q.x.toFixed(1) + ' ' + q.y.toFixed(1)).join(' L') + ' L' + p[p.length - 1].x.toFixed(1) + ' 80 Z';
    });
    const last = derived(() => pts()[pts().length - 1] || { x: 0, y: 0 });
    return { pts, path, area, last };
  },
});
