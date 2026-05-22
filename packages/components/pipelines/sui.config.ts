import type { SuiBuildConfig } from '@ship/sui-build'

// `@ship/pipelines` — first-party SUI pack for pipeline primitives.
// One umbrella entry (`sui-pipelines`) side-effect-imports every
// `.compiled.js`; a single `import '@ship/pipelines'` registers all
// eight tags globally. Per-pack CEM aggregation lands in dist/.
const config: SuiBuildConfig = {
  entries: {
    'sui-pipelines': 'src/index.ts',
  },
  outdir: 'dist',
  aggregateCemTo: 'dist/custom-elements.json',
  aggregateCemSources: ['src'],
}

export default config
