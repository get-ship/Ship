// Vitest setup for `@ship/viewer`. The dispatcher composes `@ship/sui-primitives`
// (sui-markdown, sui-code, sui-json-view, sui-empty-state), so the shared
// Web-API shims from `@ship/test-setup` (ResizeObserver, matchMedia, Popover,
// Dialog, ToggleEvent, PointerEvent) must be installed before the dispatcher
// mounts. This pack carries no icon registry of its own — `sui-icon`, when
// composed, primes its cache lazily — so only the polyfills are wired here.

import { initJsdomPolyfills } from '@ship/test-setup'

initJsdomPolyfills()
