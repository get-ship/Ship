// Vitest setup for `@ship/sui-viewer`. The viewing components compose
// `@ship/sui-primitives` (toggle-group, tabs, switch, dialog-backed surfaces,
// pointer-driven controls), so the shared Web-API shims from `@ship/test-setup`
// (ResizeObserver, matchMedia, Popover, Dialog, ToggleEvent, PointerEvent) must
// be installed before any specimen mounts. The viewer pack carries no icon
// registry of its own — `sui-icon`, when composed, primes its cache lazily — so
// only the polyfills are wired here (the terminal/chat-elements precedent).

import { initJsdomPolyfills } from '@ship/test-setup'

initJsdomPolyfills()
