import type { SuiBuildConfig } from '@ship/sui-build'

// `@ship/ui-components` — Ship domain `<sui-*>` components.
// Chrome was relocated to `apps/shell/src/components/` (built by the
// shell's own sui-build invocation); this package keeps only the
// feature/domain components (git-diff, log-stream, harness-blob,
// auth-dialog, graph-canvas). sui-pipeline-* moved to the dedicated
// @ship/pipelines pack (#151) — don't re-add them here.
//
// The entry drives per-component .sui compilation; CEMs land under
// `.ship/sui-out/` (read by `@ship/sui-catalog`). The SHARED served
// umbrella at `crates/deck/deck-core/assets/sui-components.js` (the
// daemon's `/_sui/components.js`) is owned by the DECK BUNDLER, which
// builds it from this same entry. This package writes its umbrella only
// to its own `dist/` — both wrote the shared path before, a
// two-producer collision (#160); the deck bundler is canonical.

const config: SuiBuildConfig = {
  entries: {
    'sui-components': 'src/index.ts',
  },
  outdir: 'dist',
}

export default config
