import type { SuiBuildConfig } from '@ship/sui-build'

// `@ship/ui-components` — Ship domain `<sui-*>` components.
// Chrome was relocated to `apps/shell/src/components/` (built by the
// shell's own sui-build invocation); this package keeps only the
// feature/domain components (git-diff, pipeline-*, log-stream,
// harness-blob, auth-dialog, graph-canvas).
//
// The walker in `@ship/sui-catalog` reads the per-component CEMs
// emitted under `.ship/sui-out/` and (because this package declares
// `customElements` would mean aggregated only) here we don't declare
// it — the umbrella bundle is consumed by the daemon's
// `/_sui/components.js` route and component-level catalog data flows
// through `.ship/sui-out/` directly.

const config: SuiBuildConfig = {
  entries: {
    'sui-components': 'src/index.ts',
  },
  outdir: '../../crates/sui/ship-sui-server/assets',
}

export default config
