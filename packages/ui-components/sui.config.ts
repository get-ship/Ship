import type { SuiBuildConfig } from '@ship/sui-build'

const config: SuiBuildConfig = {
  entries: {
    'sui-components': 'src/index.ts',
  },
  lazyBundles: {
    'sui-terminal':        'src/islands/sui-terminal/index.ts',
    'sui-code-editor':     'src/islands/sui-code-editor/index.ts',
    'sui-markdown-editor': 'src/islands/sui-markdown-editor/index.ts',
    'sui-command-bar':     'src/islands/sui-command-bar/index.ts',
    'sui-launcher':        'src/islands/sui-launcher/index.ts',
    'sui-nav-top':         'src/islands/sui-nav-top/index.ts',
    'sui-panel-dock':      'src/islands/sui-panel-dock/index.ts',
    'sui-devtools':        'src/islands/sui-devtools/index.ts',
  },
  outdir: '../../crates/ship-sui-server/assets',
  aggregateCemTo: '../../crates/ship-sui/assets/sui-custom-elements.json',
  aggregateCemSources: ['src'],
}

export default config
