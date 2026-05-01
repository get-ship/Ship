// @ship/ui-components — ported primitive registry.
//
// Each component lives under `src/<tag>/<tag>.html.set` (template +
// `<component>` directive + scoped `<style>`) plus `<tag>.ts` (a
// side-effect import of the AOT-compiled output produced by
// `ship-set-compiler` — see `scripts/build-bundles.mjs`).
//
// Importing this index module loads every component's `.ts`, which in
// turn loads its `<tag>.compiled.js` — the compiled module's
// top-level `component(tag, …)` call registers the custom element
// with the global registry. No template parser is invoked at runtime.
//
// The daemon serves a pre-built bundle of this module from
// `/_ship-set/components.js`; the shell loads it before any view, so
// every `<set-*>` tag is defined globally before any view template
// instantiates one.
//
// Authors don't import individual register functions — they just
// drop `<set-status-dot>` (or whatever) into their `.html.set` files.

import './set-status-dot/set-status-dot'
import './set-spinner/set-spinner'
import './set-badge/set-badge'
import './set-chip/set-chip'
import './set-card/set-card'
import './set-link/set-link'
import './set-divider/set-divider'
import './set-skeleton/set-skeleton'
import './set-stat/set-stat'
import './set-mark/set-mark'
import './set-kbd/set-kbd'
import './set-avatar/set-avatar'
import './set-progress/set-progress'
// set-button has a paired controller (set-button.ts is the controller,
// not a side-effect entry); import the compiled.js directly to register.
import './set-button/set-button.compiled.js'
import './set-tabs/set-tabs'
import './set-tab/set-tab'
import './set-empty-state/set-empty-state'
import './set-input/set-input'

/**
 * @deprecated Components are now AOT-compiled by `ship-set-compiler`
 * and register themselves as a side-effect of importing this module.
 * Calling `registerAll()` is a no-op kept for back-compat with
 * pre-Arc-B-1 callers; remove your call sites — the import alone is
 * enough.
 */
export function registerAll(): void {
  // intentionally empty — registration happens at module load
}
