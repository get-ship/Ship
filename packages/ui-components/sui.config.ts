import type { SuiBuildConfig } from '@ship/sui-build'

const config: SuiBuildConfig = {
  entries: {
    'sui-components': 'src/index.ts',
  },
  lazyBundles: {
    'sui-terminal':        'src/sui-terminal/index.ts',
    'sui-code-editor':     'src/sui-code-editor/index.ts',
    'sui-markdown-editor': 'src/sui-markdown-editor/index.ts',
    'sui-command-bar':     'src/sui-command-bar/index.ts',
    'sui-launcher':        'src/sui-launcher/index.ts',
    'sui-nav-top':         'src/sui-nav-top/index.ts',
    'sui-panel-dock':      'src/sui-panel-dock/index.ts',
    'sui-rail':            'src/sui-rail/index.ts',
    'sui-devtools':        'src/sui-devtools/index.ts',
    'sui-component-picker': 'src/sui-component-picker/index.ts',
    'sui-view-authoring':  'src/sui-view-authoring/index.ts',
  },
  outdir: '../../crates/sui/ship-sui-server/assets',
  // Separate-bundle components (opted in via the `<bundle separate />`
  // directive in their `.sui` source) are code-split and loaded on demand by
  // sui-runtime; keep them under `sui-components/` so the always-loaded
  // framework aggregate (`sui-components.js`) at the top of `assets/` stays
  // distinguishable.
  lazyOutSubdir: 'sui-components',
  // No `aggregateCemTo` here — the workspace-wide catalog is built by
  // `@ship/sui-catalog` (CLI: `bun packages/sui-catalog/src/cli.ts`),
  // which walks every workspace package and emits
  // `.ship/sui-out/catalogs/{workspace,published}.json`. The per-component
  // `*.custom-elements.json` files the compiler emits under `.ship/sui-out/`
  // are the input. Per-pack aggregation (e.g. `@ship/charts`) still uses
  // sui-build's `aggregateCemTo` because the daemon's pack-bundle handler
  // exposes `/_sui/packs/<pack-id>/custom-elements.json`.
}

export default config
