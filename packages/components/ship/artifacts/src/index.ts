// @ship/artifacts — Ship Artifacts pack (ship-* tags, pre-bound to ship.artifacts.*).
//
// Each <tag>.compiled.js calls customElements.define() on import, so a
// single `import '@ship/artifacts'` registers every tag globally.
//
// Host requirement: the host (shell main.ts) must call setShipClient()
// before any of these tags mount, otherwise the lazy `ship` Proxy will
// throw on first access. The host also registers the sui primitives
// pack (`import '@ship/sui-primitives'`) — the artifact list composes
// <sui-table> · <sui-skeleton> · <sui-error-state> · <sui-empty-state>,
// the actions/upload components compose <sui-button> · <sui-dialog>
// · <sui-file-upload> · <sui-input> · <sui-select> · <sui-icon>.
//
// Composition order: presentational atoms first, then the live-data
// composite. <ship-artifact-list> depends on no other ship-* component
// (it composes <sui-table> from the primitives pack), so it could
// come first — but the presentational building blocks land first by
// convention so a host can mount any one in isolation.

// ─── Atoms (presentational; no live data) ─────────────────────────────
import './ship-artifact-actions/ship-artifact-actions.compiled.js'
import './ship-artifact-filters/ship-artifact-filters.compiled.js'
import './ship-artifact-preview/ship-artifact-preview.compiled.js'
import './ship-artifact-upload/ship-artifact-upload.compiled.js'

// ─── Live-data composite ──────────────────────────────────────────────
import './ship-artifact-list/ship-artifact-list.compiled.js'

// ─── Shared decoration + viewer registry helpers ──────────────────────
// MOVED from `views/artifacts/` as part of the pack extraction. The
// view will refactor to import from this pack on a follow-up change.
export {
  AGE_BUCKETS,
  bucketFor,
  decorate,
  formatSize,
  previewKindFor,
  type ArtifactRow,
} from './decorate'
export { viewerFor } from './viewers'
export type { ViewerHit } from './viewers.types'
