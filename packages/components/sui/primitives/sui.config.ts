import type { SuiBuildConfig } from '@ship/sui-build'

// `@ship/sui-primitives` — first-party SUI components pack for the generic
// UI primitives (button, input, dialog, popover, tooltip, tabs, etc.).
//
// One umbrella entry (`sui-primitives`) side-effect-imports every
// primitive `.compiled.js`; consumers do a single
// `import '@ship/sui-primitives'` (or `import '@ship/sui-primitives/bundle'`)
// and every primitive tag registers globally before any view template
// instantiates them.
//
// CEM aggregation lands inside this pack's own `dist/` — per-pack
// aggregation is the convention; daemon's pack-bundle handler exposes
// `/_sui/packs/primitives/custom-elements.json`.

const config: SuiBuildConfig = {
  entries: {
    'sui-primitives': 'src/index.ts',
  },
  outdir: 'dist',
  aggregateCemTo: 'dist/custom-elements.json',
  aggregateCemSources: ['src'],
}

export default config
