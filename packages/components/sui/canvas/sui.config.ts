import type { SuiBuildConfig } from '@ship/sui-build'

// `@ship/sui-canvas` — the drawing-surface pack. One umbrella entry
// (`sui-canvas`) side-effect-imports the compiled tag; a single
// `import '@ship/sui-canvas'` registers `<sui-canvas>` globally. Per-pack
// CEM aggregation lands in dist/.
//
// Bundle note: this pack pulls in @excalidraw/excalidraw + a React 19
// tree (~1 MB minified). Consumers only pay that cost when they import
// the pack — it tree-shakes per-tag from '@ship/sui-canvas/sui-canvas'.
const config: SuiBuildConfig = {
  entries: {
    'sui-canvas': 'src/index.ts',
  },
  outdir: 'dist',
  aggregateCemTo: 'dist/custom-elements.json',
  aggregateCemSources: ['src'],
  // Excalidraw does NOT auto-inject its stylesheet in v0.18. Load the
  // `.css` import as raw text so the bridge can adopt it into the
  // component's shadow root (document-head CSS can't cross the boundary).
  loaders: { '.css': 'text' },
}

export default config
