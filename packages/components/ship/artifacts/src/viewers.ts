// ship-artifacts viewer registry. Maps mime → viewer route + badge label per
// .artifacts/canvas-design/spec.md. The real per-view manifest-derived
// registry lands later; this hardcodes the known Ship types so /artifacts
// can route rows to the right viewer today.

import type { ViewerHit } from './viewers.types'

export type { ViewerHit } from './viewers.types'

const EXACT: Record<string, (id: string) => ViewerHit> = {
  'text/html':                 (id) => ({ label: 'canvas:read',   url: `/canvas/${id}` }),
  'text/markdown':             (id) => ({ label: 'canvas:doc',    url: `/canvas/${id}` }),
  'application/x-sui-scaffold':(id) => ({ label: 'canvas:read',   url: `/canvas/${id}` }),
  'application/x-excalidraw':  (id) => ({ label: 'canvas:sketch', url: `/canvas/${id}?mode=sketch` }),
  'application/x-git-diff':    ()   => ({ label: 'view/git',      url: '/git' }),
  'text/x-diff':               ()   => ({ label: 'view/git',      url: '/git' }),
}

/** Returns the registered viewer for the artifact, or null when no
 *  viewer is wired up and the row should fall back to "download only".
 *  Mime parameters (`;charset=...`) are stripped before lookup. */
export function viewerFor(row: { id: string; mime: string }): ViewerHit | null {
  const m = (row.mime || '').toLowerCase().split(';')[0]!.trim()
  const hit = EXACT[m]
  if (hit) return hit(row.id)
  if (m.startsWith('image/')) return { label: 'canvas:image', url: `/canvas/${row.id}` }
  return null
}
