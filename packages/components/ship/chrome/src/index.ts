// @ship/chrome — Ship Chrome pack (ship-* tags for shell chrome).
//
// Each <tag>.compiled.js calls customElements.define() on import, so a single
// `import '@ship/chrome'` registers every tag globally.
//
// These components are presentational: they render shell chrome (command bar,
// activity bar, file tree, status bar, dock) and emit operator intents
// (ship-open/close, ship-mode-change, ship-submit, ship-select,
// ship-context-change, ship-new, ship-collapse-all, ship-toggle,
// ship-open-runs, ship-launch). The host owns wiring to the daemon and
// composes @ship/sui-primitives (sui-icon), a declared dependency the host
// registers via `import '@ship/sui-primitives'`.
import './ship-command-bar/ship-command-bar.compiled.js'
import './ship-activity-bar/ship-activity-bar.compiled.js'
import './ship-file-tree/ship-file-tree.compiled.js'
import './ship-tree-item/ship-tree-item.compiled.js'
import './ship-status-bar/ship-status-bar.compiled.js'
import './ship-dock/ship-dock.compiled.js'
import './ship-dock-app/ship-dock-app.compiled.js'
