// sparkline.engine.controller.js — ported sparkline geometry off the shared
// engine (normalizeSeries → normalizedExtent → line/area). Same engine as
// bar-chart; existing + new share it. Loaded via "@charts/engine".
import { controller, derived } from '@ship/sui-runtime';
import { normalizeSeries, normalizedExtent, lineGeometry, areaGeometry } from '@charts/engine';

const BOX = { w: 240, h: 80 };

export default controller({
  scope: ({ data, tone }) => {
    const norm = derived(() => normalizeSeries(data()).map((s) => ({ name: s.name, tone: s.tone || tone(), data: s.data })));
    const line = derived(() => lineGeometry(normalizedExtent(norm()), BOX)[0] || { strokeD: '', color: 'var(--primary)', pts: [] });
    const area = derived(() => areaGeometry(normalizedExtent(norm()), BOX)[0] || { fillD: '' });
    const last = derived(() => { const p = line().pts; return p[p.length - 1] || { cx: 0, cy: 0 }; });
    return { line, area, last };
  },
});
