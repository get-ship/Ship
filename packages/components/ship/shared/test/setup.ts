// Vitest setup for `@ship/primitives` (ship/shared). JSDOM polyfills so the
// shared pack's components see the DOM helpers they need (ResizeObserver,
// matchMedia, …) without each test re-importing them.

import { initJsdomPolyfills } from '@ship/test-setup'

initJsdomPolyfills()
