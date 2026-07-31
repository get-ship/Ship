// @ship/artifacts — Ship Artifacts pack (SDK-free subset).
//
// This copy contains only the three presentational (SDK-free) components:
//   ship-artifact-actions, ship-artifact-filters, ship-artifact-upload.
//
// SDK-coupled components (ship-artifact-list, ship-artifact-preview) were
// NOT copied — they remain in Ship and extract with the apps later.
//
// Host requirement: register the sui primitives pack
// (`import '@ship/sail-primitives'`) before mounting any of these tags.

// ─── Atoms (presentational; no live data) ─────────────────────────────
import './ship-artifact-actions/ship-artifact-actions.compiled.js'
import './ship-artifact-filters/ship-artifact-filters.compiled.js'
import './ship-artifact-upload/ship-artifact-upload.compiled.js'

// ─── Shared decoration + viewer registry helpers ──────────────────────
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
