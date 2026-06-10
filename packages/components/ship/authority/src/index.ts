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

// Shared authority vocabulary — the blast tier + gate verdict atoms the rest of
// the pack (and op-card) compose. Registered before the components that compose
// them.
import './ship-blast-tag/ship-blast-tag.compiled.js'
import './ship-gate-badge/ship-gate-badge.compiled.js'

// Authority surfaces — batched inbox, the standing-grant readouts, the audit log.
import './ship-approvals-queue/ship-approvals-queue.compiled.js'
import './ship-capability-bar/ship-capability-bar.compiled.js'
import './ship-policy-matrix/ship-policy-matrix.compiled.js'
import './ship-principal-list/ship-principal-list.compiled.js'
import './ship-gate-log/ship-gate-log.compiled.js'

export type { AuthorityRequest } from './ship-approval-card/ship-approval-card.controller'
export type { GateLogEntry } from './ship-gate-log/ship-gate-log.controller'
export type { Grant, Denial } from './ship-capability-bar/ship-capability-bar.controller'
export type { PolicyRow } from './ship-policy-matrix/ship-policy-matrix.controller'
export type { Principal } from './ship-principal-list/ship-principal-list.controller'
