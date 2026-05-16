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
  // Aggregate CEM lands next to the bundles in `outdir` so the crate that
  // bakes it in (`crates/sui/ship-sui-server`, via `build.rs` +
  // `src/assets.rs`) reads a fresh table. A post-fold typo pointed this at
  // the untracked `crates/ship-sui/assets/`, which nothing consumes — the
  // server CEM then went stale (missing every primitive added since).
  aggregateCemTo: '../../crates/sui/ship-sui-server/assets/sui-custom-elements.json',
  aggregateCemSources: ['src'],
}

export default config
