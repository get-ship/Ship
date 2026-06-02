// Vitest setup for `@ship/ui-components` — JSDOM polyfills + per-icon cache
// priming. Shared helpers live in `@ship/test-setup`; the registry primer
// comes from `@ship/sui-primitives` (where `<sui-icon>` actually lives).

import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { initJsdomPolyfills, primeLucideIcons } from '@ship/test-setup'
import { primeIconCache } from '../../sui/primitives/src/sui-icon/registry'

// Resolve the lucide JSON relative to THIS file, not process.cwd(): vitest's
// cwd is the invocation dir (repo root under `--root pkg`), not the package
// dir, so a cwd-relative path walks above the repo and ENOENTs.
const here = dirname(fileURLToPath(import.meta.url))

primeLucideIcons(
  resolve(here, '../../../../crates/deck/deck-core/assets/lucide-icons.json'),
  primeIconCache,
)
initJsdomPolyfills()
