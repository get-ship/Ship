// @ship/shared — Ship Shared pack (internal ship-* components shared across
// Ship's own apps: hub, spaces). INTERNAL — not a published domain pack.
//
// Each <tag>.compiled.js calls customElements.define() on import, so a single
// `import '@ship/shared'` registers every tag globally.
//
// These are presentational atoms + layout shells. They are purely
// presentational and RUNTIME-FREE — the host passes data + permissions via
// props/reflected attributes and the component renders; nothing here imports
// @ship/sail-runtime, @ship/sdk(-client), or the contract (enforced by `bun run
// guard`). They compose @ship/sail-primitives (sui-icon, sui-tree, …) and
// runtime packs (@ship/chrome's <ship-nav-top>) when called for; the host is
// responsible for registering @ship/sail-primitives via `import
// '@ship/sail-primitives'` and any composed runtime pack.
import './ship-watch-badge/ship-watch-badge.compiled.js'
import './ship-error-state/ship-error-state.compiled.js'
// <ship-app-chrome> — the per-app root layout wrapper (hub's routes/_layout.sail
// names it as the LAYOUTS tag). Pure assembler: composes @ship/chrome's
// <ship-nav-top> and exposes the router's `route` slot + a `command-bar`
// pass-through. No controller, no runtime import.
import './ship-app-chrome/ship-app-chrome.compiled.js'
// <ship-file-tree> — titled navigator panel that composes the generic
// <sui-tree> primitive (per-node file-type icons). Data arrives as the
// `nodes` JSON attribute (or a host-set `nodes` property on the inner tree);
// selection/toggle events bubble from <sui-tree>. Header actions are a slot
// the host fills + wires. No controller, no runtime import.
import './ship-file-tree/ship-file-tree.compiled.js'
