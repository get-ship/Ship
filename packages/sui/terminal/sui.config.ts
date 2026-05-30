import type { SuiBuildConfig } from '@ship/sui-build'

// `@sui/terminal` — PTY rendering primitive.
//
// Wraps xterm.js in a SUI custom element. Generic terminal surface:
// code editors, hosted shells, devops UIs. No `ship` Proxy, no SDK
// reads. Consumers bind their own PTY transport.
const config: SuiBuildConfig = {
  entries: {
    'sui-terminal': 'src/index.ts',
  },
  outdir: 'dist',
  aggregateCemTo: 'dist/custom-elements.json',
  aggregateCemSources: ['src'],
}

export default config
