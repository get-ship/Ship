// Vitest setup for the hub (apps/shell) — JSDOM polyfills + per-icon cache
// priming. Shared helpers live in `@ship/test-setup`. The shell flips on
// `suppressRafErrors` because xterm.js's WebGL addon throws inside rAF
// callbacks under jsdom; the other consumers don't bundle xterm.

import { resolve } from 'node:path'
import { initJsdomPolyfills, primeLucideIcons } from '@ship/test-setup'
import { primeIconCache } from '../../../packages/components/primitives/src/sui-icon/registry'

primeLucideIcons(
  resolve(process.cwd(), '../../crates/deck/deck-core/assets/lucide-icons.json'),
  primeIconCache,
)
initJsdomPolyfills({ suppressRafErrors: true })
