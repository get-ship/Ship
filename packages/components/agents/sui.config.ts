import type { SuiBuildConfig } from '@ship/sui-build'

// `@ship/agents` — first pre-bound (`ship-*`) pack. Components reach
// for `ship.agents.*` via the lazy `ship` Proxy from `@ship/sdk-client`,
// so the host must register a client (`setShipClient`) before any of
// these tags mount.
//
// Pure `sui-*` packs (primitives, charts, pipelines) stay above this
// layer — they never see `ship`. This pack establishes the `ship-*`
// naming convention: tag prefix `ship-*` ↔ reaches into Ship runtime.
const config: SuiBuildConfig = {
  entries: {
    'ship-agents': 'src/index.ts',
  },
  outdir: 'dist',
  aggregateCemTo: 'dist/custom-elements.json',
  aggregateCemSources: ['src'],
}

export default config
