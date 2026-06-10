// Vitest setup for the hub (apps/shell) — JSDOM polyfills + per-icon cache
// priming. Shared helpers live in `@ship/test-setup`. The shell flips on
// `suppressRafErrors` because xterm.js's WebGL addon throws inside rAF
// callbacks under jsdom; the other consumers don't bundle xterm.

import { initJsdomPolyfills, primeLucideIcons } from '@ship/test-setup'
import { primeIconCache } from '../../../packages/components/sui/primitives/src/sui-icon/registry'

primeLucideIcons(primeIconCache)
initJsdomPolyfills({ suppressRafErrors: true })
