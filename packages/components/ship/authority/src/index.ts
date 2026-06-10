// @ship/authority — Ship Authority pack (ship-* tags for the approval flow).
//
// Each <tag>.compiled.js calls customElements.define() on import, so a single
// `import '@ship/authority'` registers every tag globally.
//
// These components are presentational: they render an authority request and
// emit the operator's decision (`ship-decide`, `ship-inspect`, `ship-dismiss`).
// The host (e.g. views/authority) owns the ship.authority.pending()
// subscription and the ship.authority.decide() mutation, and composes
// @ship/sui-primitives (sui-badge, sui-button, sui-dialog), a declared
// dependency the host registers via `import '@ship/sui-primitives'`.
import './ship-approval-card/ship-approval-card.compiled.js'
import './ship-capability-request/ship-capability-request.compiled.js'
import './sui-auth-dialog/sui-auth-dialog.compiled.js'

export type { AuthorityRequest } from './ship-approval-card/ship-approval-card.controller'
