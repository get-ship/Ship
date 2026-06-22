// @ship/shared — Ship Shared pack (internal ship-* components shared across
// Ship's own apps: hub, spaces). INTERNAL — not a published domain pack.
//
// Each <tag>.compiled.js calls customElements.define() on import, so a single
// `import '@ship/shared'` registers every tag globally.
//
// These are tiny status / affordance atoms. They are purely presentational —
// the host passes state via reflected attributes and the component renders.
// They compose @ship/sui-primitives (sui-badge) when called for; the host is
// responsible for registering @ship/sui-primitives via `import
// '@ship/sui-primitives'`.
import './ship-watch-badge/ship-watch-badge.compiled.js'
