import type { SuiBuildConfig } from '@ship/sui-build'

const config: SuiBuildConfig = {
  entries: {
    'ship-elements': 'src/web-components/index.ts',
  },
  lazyBundles: {
    'ship-code-editor':     'src/web-components/islands/ship-code-editor.ts',
    'ship-markdown-editor': 'src/web-components/islands/ship-markdown-editor.ts',
    'ship-terminal':        'src/web-components/islands/ship-terminal.ts',
    'ship-git':             'src/packs/git/index.ts',
  },
  outdir: '../../apps/shipd/assets',
  loaders: {
    '.svg': 'text',
  },
}

export default config
