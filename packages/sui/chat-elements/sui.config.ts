import type { SuiBuildConfig } from '@ship/sui-build'

// `@sui/chat-elements` — first pack in the new `@sui/*` namespace.
//
// SUI framework lives in `@sui/*`; Ship product in `@ship/*`. These are
// pure chat-shape primitives (`sui-*` tags): no `ship` Proxy, no
// `setShipClient`, no SDK reads. Consumers (`@ship/agents` wrappers,
// edit-mode side chats, the activity stream) bind them to their own
// data — the components stay framework-tier and host-agnostic.
const config: SuiBuildConfig = {
  entries: {
    'sui-chat-elements': 'src/index.ts',
  },
  outdir: 'dist',
  aggregateCemTo: 'dist/custom-elements.json',
  aggregateCemSources: ['src'],
}

export default config
