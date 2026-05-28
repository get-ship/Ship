// Vitest setup for `@ship/ui-components` — JSDOM polyfills + per-icon cache
// priming. Shared helpers live in `@ship/test-setup`; the registry primer
// comes from `@ship/primitives` (where `<sui-icon>` actually lives).

import { resolve } from 'node:path'
import { initJsdomPolyfills, primeLucideIcons } from '@ship/test-setup'
import { primeIconCache } from '../../primitives/src/sui-icon/registry'

primeLucideIcons(
  resolve(process.cwd(), '../../../crates/deck/deck-core/assets/lucide-icons.json'),
  primeIconCache,
)
initJsdomPolyfills()
