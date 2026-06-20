// @ship/chrome — Ship Chrome pack (ship-* tags for shell chrome).
//
// Each <tag>.compiled.js calls customElements.define() on import, so a single
// `import '@ship/chrome'` registers every tag globally.
//
// These components are presentational: they render shell chrome (command bar,
// activity bar, file tree, status bar, dock, nav-top) and emit operator intents
// (ship-open/close, ship-mode-change, ship-submit, ship-select,
// ship-context-change, ship-new, ship-collapse-all, ship-toggle,
// ship-open-runs, ship-launch).
// <ship-status-bar> is the launch-chrome status strip (migrated from
// apps/shell/src/components/sui-status-bar); it emits sui:dock:toggle,
// sui:project-switcher:open, sui:theme-picker:open for its three clickable
// segments and subscribes to sui:status:set / sui:status:clear on document
// for the view-extensible status zone. The host owns wiring to the daemon and
// composes @ship/sui-primitives (sui-icon), a declared dependency the host
// registers via `import '@ship/sui-primitives'`.
import './ship-command-bar/ship-command-bar.compiled.js'
import './ship-activity-bar/ship-activity-bar.compiled.js'
import './ship-file-tree/ship-file-tree.compiled.js'
import './ship-tree-item/ship-tree-item.compiled.js'
import './ship-status-bar/ship-status-bar.compiled.js'
import './ship-dock/ship-dock.compiled.js'
import './ship-dock-app/ship-dock-app.compiled.js'
import './ship-nav-top/ship-nav-top.compiled.js'
// <ship-nav-top> composes these two domain affordances; the pack owns them so
// any host that mounts the nav bar gets the project switcher + account menu
// registered (the hub previously kept app-local copies — now consolidated here).
import './sui-project-switcher/sui-project-switcher.compiled.js'
import './sui-account-menu/sui-account-menu.compiled.js'
import './ship-rail/ship-rail.compiled.js'
import './ship-rail/ship-rail-item.compiled.js'
import './ship-rail/ship-rail-section.compiled.js'
