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

import './set-status-dot/set-status-dot.compiled.js'
import './set-spinner/set-spinner.compiled.js'
import './set-badge/set-badge.compiled.js'
import './set-chip/set-chip.compiled.js'
import './set-card/set-card.compiled.js'
import './set-link/set-link.compiled.js'
import './set-divider/set-divider.compiled.js'
import './set-skeleton/set-skeleton.compiled.js'
import './set-stat/set-stat.compiled.js'
import './set-mark/set-mark.compiled.js'
import './set-kbd/set-kbd.compiled.js'
import './set-avatar/set-avatar.compiled.js'
import './set-progress/set-progress.compiled.js'
import './set-button/set-button.compiled.js'
import './set-tabs/set-tabs.compiled.js'
import './set-tab/set-tab.compiled.js'
import './set-empty-state/set-empty-state.compiled.js'
import './set-input/set-input.compiled.js'

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
