// @ship/chrome — Ship Chrome pack (ship-* tags for shell chrome).
//
// Each <tag>.sail calls customElements.define() on import, so a single
// `import '@ship/chrome'` registers every tag globally.
//
// These components are runtime-coupled shell chrome: they own controllers that
// reach for the SDK / router / live data and emit operator intents
// (ship-open/close, ship-mode-change, ship-submit, ship-select,
// ship-context-change, ship-new, ship-collapse-all, ship-toggle,
// ship-open-runs, ship-launch). Purely-presentational shells (the layout
// wrapper <ship-app-chrome>, the <sui-tree>-composing <ship-file-tree>) and the
// SDK-free affordances (<sui-account-menu>, <sui-project-switcher>) no longer
// live here: the wrappers moved to @ship/shared (runtime-free) and the
// affordances are now the @ship/sail-primitives versions, wired host-side
// (<ship-nav-top> supplies their account/space data + handles their events).
// <ship-status-bar> is the launch-chrome status strip; it emits sui:dock:toggle,
// sui:project-switcher:open, sui:theme-picker:open for its three clickable
// segments and subscribes to sui:status:set / sui:status:clear on document
// for the view-extensible status zone. The host owns wiring to the daemon and
// composes @ship/sail-primitives (sui-icon), a declared dependency the host
// registers via `import '@ship/sail-primitives'`.
import './ship-command-bar/ship-command-bar.compiled.js'
import './ship-activity-bar/ship-activity-bar.compiled.js'
import './ship-status-bar/ship-status-bar.compiled.js'
import './ship-dock/ship-dock.compiled.js'
import './ship-dock-app/ship-dock-app.compiled.js'
// <ship-home-dock> — the desktop launcher tray; composes <ship-dock-app> tiles
// for the hub `/` home. Route-agnostic: emits the tile's `ship-launch { label }`
// upward, the host maps label → route.
import './ship-home-dock/ship-home-dock.compiled.js'
// <ship-nav-top> — the filled top bar. Composes the @ship/sail-primitives
// <sui-account-menu> (SDK-free); this controller supplies its account data +
// handles its sui-account-* events (sign in/out, settings, billing).
import './ship-nav-top/ship-nav-top.compiled.js'
import './ship-rail/ship-rail.compiled.js'
import './ship-rail/ship-rail-item.compiled.js'
import './ship-rail/ship-rail-section.compiled.js'
// <ship-command-palette> — the ⌘K palette composition (trigger pill + dialog).
// It provisions a shared scope its two inner panels consume via host-walk:
// <ship-command-panel> (find/command/mention/create) and <ship-command-chat>
// (the unified ai mode). Distinct from the <ship-command-bar> primitive above.
import './ship-command-palette/ship-command-palette.compiled.js'
import './ship-command-palette/ship-command-panel.compiled.js'
import './ship-command-palette/ship-command-chat.compiled.js'
