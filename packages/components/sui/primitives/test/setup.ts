// Vitest setup for `@ship/sui-primitives` — JSDOM polyfills + per-icon cache
// priming. Shared helpers live in `@ship/test-setup`; only the package-local
// glue stays here (the path to the committed lucide JSON, the registry's
// `primeIconCache` to push into).

import { resolve } from 'node:path'
import { initJsdomPolyfills, primeLucideIcons } from '@ship/test-setup'
import { primeIconCache } from '../src/sui-icon/registry'

primeLucideIcons(
  resolve(process.cwd(), '../../../../crates/deck/deck-core/assets/lucide-icons.json'),
  primeIconCache,
)
initJsdomPolyfills()
