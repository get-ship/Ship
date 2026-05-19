import type { SuiBuildConfig } from '@ship/sui-build'

// `@ship/charts` — first-party SUI components pack for chart primitives.
//
// One umbrella entry (`sui-charts`) that side-effect-imports every chart
// `.compiled.js`; consumers do a single `import '@ship/charts'` (or
// `import '@ship/charts/dist/sui-charts.js'`) and all eight tags
// register globally before any view template instantiates them.
//
// CEM aggregation lands inside this pack's own `dist/` —
// per-pack aggregation is the convention; a higher-tier aggregator
// (cross-pack, daemon-served) is a separate concern that arrives once
// more than one pack lands.

const config: SuiBuildConfig = {
  entries: {
    'sui-charts': 'src/index.ts',
  },
  outdir: 'dist',
  aggregateCemTo: 'dist/custom-elements.json',
  aggregateCemSources: ['src'],
}

export default config
