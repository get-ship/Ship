// @ship/primitives — Ship Primitives pack (small, presentational ship-* atoms).
//
// Each <tag>.compiled.js calls customElements.define() on import, so a single
// `import '@ship/primitives'` registers every tag globally.
//
// These are tiny status / affordance atoms. They are purely presentational —
// the host passes state via reflected attributes and the component renders.
// They compose @ship/sui-primitives (sui-badge) when called for; the host is
// responsible for registering @ship/sui-primitives via `import
// '@ship/sui-primitives'`.
import './ship-watch-badge/ship-watch-badge.compiled.js'
import './ship-error-state/ship-error-state.compiled.js'
