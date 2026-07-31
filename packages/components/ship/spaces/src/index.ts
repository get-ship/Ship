// @ship/spaces — Ship Spaces pack (ship-* tags for the SHELL side of spaces).
//
// Each <tag>.compiled.js calls customElements.define() on import, so a single
// `import '@ship/spaces'` registers every tag globally.
//
// These are the SHELL surfaces — they speak the REGISTRY concepts (List,
// active_id, Add, Activate), NOT the self-referential Space-window surface.
// They are presentational: rows are fed via the `spaces` JSON attribute and the
// active pointer via `active`; choosing a space emits `ship-activate {id}`,
// registering emits `ship-add {path}`. The host (e.g. the shell) owns the
// SpaceService.List subscription and the Activate/Add mutations, and composes
// @ship/sail-primitives (sui-icon), a declared dependency the host registers via
// `import '@ship/sail-primitives'`.
//
// `active`/`active_id` is the daemon-global DEFAULT POINTER (where context-free
// invocations route) — activating a space re-points it; it does not swap pane
// content. Switching surfaces a space's OS window.
import './ship-space-active-badge/ship-space-active-badge.compiled.js'
import './ship-space-switcher/ship-space-switcher.compiled.js'
import './ship-space-registry/ship-space-registry.compiled.js'
import './ship-space-add-dialog/ship-space-add-dialog.compiled.js'
