// Vitest config for the @ship/agents pack.
//
// Uses the shared @ship/test-setup factory — one source of truth, no inlined
// copy. The factory's `compiled-js-test-shim` resolves each `<tag>.compiled.js`
// from its sibling `.sui` (WASM-compiled in this JS-only repo, native in the
// sui monorepo), and `resolveSuiAliases` pins whichever @ship/* instances
// resolve here. Defaults: jsdom + the honesty guard + `./test/setup.ts` (the
// pack's ResizeObserver stub) + the shared fork/timeout budget.
import { suiComponentVitestConfig } from '@ship/test-setup/vitest'

export default suiComponentVitestConfig()
