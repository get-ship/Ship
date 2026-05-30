import type { SuiBuildConfig } from '@ship/sui-build'

// `@ship/sui-chat-elements` — portable chat-presentation pack.
//
// Naming rule: `@ship/sui-*` (tags `sui-*`) = portable, runs anywhere — no
// `ship` Proxy, no `setShipClient`, no SDK reads. `@ship/*` (tags `ship-*`) =
// Ship-coupled product packs. These are pure chat-shape primitives; consumers
// (`@ship/agents` wrappers, edit-mode side chats, the activity stream) bind
// them to their own data, so the components stay portable and host-agnostic.
const config: SuiBuildConfig = {
  entries: {
    'sui-chat-elements': 'src/index.ts',
  },
  outdir: 'dist',
  aggregateCemTo: 'dist/custom-elements.json',
  aggregateCemSources: ['src'],
}

export default config
