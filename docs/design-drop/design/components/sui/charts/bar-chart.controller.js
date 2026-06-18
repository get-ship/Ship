// bar-chart.controller.js — geometry for sui-bar-chart. Computes bar rects in a
// 240×80 viewBox from `data` (JSON number[]). Pure; no daemon.
import { controller, derived } from '@ship/sui-runtime';

const parse = (raw) => {
  try { const a = JSON.parse(raw); return Array.isArray(a) ? a.map(Number).filter((n) => !Number.isNaN(n)) : []; }
  catch { return []; }
};

export default controller({
  scope: ({ data }) => {
    const bars = derived(() => {
      const arr = parse(data());
      const n = arr.length;
      if (!n) return [];
      const W = 240, H = 80, gap = 4;
      const max = Math.max(...arr, 1);
      const bw = (W - gap * (n - 1)) / n;
      return arr.map((v, i) => {
        const h = Math.max(1, (v / max) * (H - 2));
        return { i, x: i * (bw + gap), w: bw, h, y: H - h, value: v };
      });
    });
    return { bars };
  },
});
