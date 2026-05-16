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
    'sui-rail':            'src/islands/sui-rail/index.ts',
    'sui-devtools':        'src/islands/sui-devtools/index.ts',
    'sui-component-picker': 'src/islands/sui-component-picker/index.ts',
    'sui-view-authoring':  'src/islands/sui-view-authoring/index.ts',
  },
  outdir: '../../crates/sui/ship-sui-server/assets',
  // Islands are code-split bundles loaded on demand by sui-runtime; keep
  // them under `sui-islands/` so the always-loaded framework bundles
  // (sui-components.js etc.) at the top of `assets/` stay distinguishable.
  lazyOutSubdir: 'sui-islands',
  // Aggregate CEM lands next to the bundles in `outdir` so the crate that
  // bakes it in (`crates/sui/ship-sui-server`, via `build.rs` +
  // `src/assets.rs`) reads a fresh table. A post-fold typo pointed this at
  // the untracked `crates/ship-sui/assets/`, which nothing consumes — the
  // server CEM then went stale (missing every primitive added since).
  aggregateCemTo: '../../crates/sui/ship-sui-server/assets/sui-custom-elements.json',
  aggregateCemSources: ['src'],
}

export default config
