// Chrome pack test setup. Runs under happy-dom (see vitest.config.ts), which
// provides ResizeObserver / matchMedia / PointerEvent natively and a CSS parser
// that tolerates the modern scoped styles these components inject.
//
// Several chrome components compose <sui-icon name="…">, which in production
// fetches the SVG from the daemon (`/_deck/icons/<name>.svg`). Under test there
// is no daemon, so we prime sui-primitives' per-icon cache from the same
// committed lucide JSON the daemon serves — icons render synchronously and no
// test makes a network call.
import { primeLucideIcons } from '@ship/test-setup'
// Deep relative import: the registry's `primeIconCache` is package-internal
// (sui-primitives only exports its barrel), so reach the file directly — vite's
// FS guard is widened to the repo root by suiComponentVitestConfig.
import { primeIconCache } from '../../../sui/primitives/src/sui-icon/registry'

primeLucideIcons(primeIconCache)
