import type { SuiBuildConfig } from '@ship/sui-build'

// `@ship/arcade` — declarative 2D arcade game primitives.
//
// One umbrella entry (`sui-arcade`) side-effect-imports every tag's
// `.compiled.js`; consumers do a single `import '@ship/arcade'` and all
// eight tags register globally before any view template instantiates them.

const config: SuiBuildConfig = {
  entries: {
    'sui-arcade': 'src/index.ts',
  },
  outdir: 'dist',
  aggregateCemTo: 'dist/custom-elements.json',
  aggregateCemSources: ['src'],
  // The engine vendors Pixi (~800 KB). Externalize it so the pack bundle
  // ships just the component tags (~20 KB); the consumer loads
  // `@ship/arcade-engine` separately (per-page or via importmap).
  external: ['@ship/arcade-engine'],
}

export default config
