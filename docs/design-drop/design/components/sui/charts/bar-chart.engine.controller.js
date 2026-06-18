// bar-chart.engine.controller.js — ported from the real
// sui-bar-chart.controller.ts. Geometry is a PURE derived off the engine
// (normalizeSeries → normalized(yMin:0) → barGeometry). Shared by BOTH the
// existing and elevated .sui — the only difference between them is the template
// + styling, so it's a true grounded before/after on identical geometry.
import { controller, derived } from '@ship/sui-runtime';
import { normalizeSeries, normalized, barGeometry } from '@charts/engine';

export default controller({
  scope: ({ data, tone, labels }) => {
    const bars = derived(() => barGeometry(
      normalized(normalizeSeries(data()), { yMin: 0 }),
      { w: 240, h: 80 },
      { tone: tone(), labels: String(labels() || '').split(',').map((s) => s.trim()).filter(Boolean) },
    ));
    return { bars };
  },
});
