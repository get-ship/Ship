// Row decoration + small format helpers shared between the controller
// and any future server-rendered version of the artifacts list.
//
// MOVED from `views/artifacts/decorate.ts` as part of the
// `@ship/artifacts` pack extraction. The view will be refactored
// separately to compose this pack; until then `views/artifacts/`
// continues to import from this location.

import { viewerFor } from './viewers'

export interface ArtifactRow {
  id: string
  name: string
  mime: string
  size: number
  size_label: string
  id_short: string
  created_short: string
  path: string
  scope: string
  collection_id?: string | null
  item_id?: string | null
  owner_actor?: string | null
  created_at: string
  viewer_label: string
  viewer_url: string | null
}

const MIME_BUCKETS: Array<{ value: string; prefix: string }> = [
  { value: 'image', prefix: 'image/' },
  { value: 'text',  prefix: 'text/' },
  { value: 'json',  prefix: 'application/json' },
]

export function bucketFor(mime: string): string {
  for (const b of MIME_BUCKETS) if (mime.startsWith(b.prefix)) return b.value
  return ''
}

export function previewKindFor(mime: string): 'image' | 'html' | 'text' | 'binary' {
  if (mime.startsWith('image/')) return 'image'
  if (mime === 'text/html') return 'html'
  if (mime.startsWith('text/') || mime === 'application/json' || mime === 'application/xml') return 'text'
  return 'binary'
}

export function formatSize(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes <= 0) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB'] as const
  let n = bytes
  let i = 0
  while (n >= 1024 && i < units.length - 1) { n /= 1024; i++ }
  const unit = units[i] ?? 'B'
  return `${n.toFixed(n < 10 && i > 0 ? 1 : 0)} ${unit}`
}

function formatShortDate(iso: string): string {
  if (!iso) return ''
  const d = new Date(iso)
  if (Number.isNaN(d.getTime())) return iso
  return d.toLocaleString(undefined, { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' })
}

export function decorate(rows: ArtifactRow[]): ArtifactRow[] {
  for (const r of rows) {
    r.size_label = formatSize(r.size)
    r.id_short = (r.id ?? '').slice(0, 12)
    r.created_short = formatShortDate(r.created_at)
    const hit = viewerFor(r)
    r.viewer_label = hit?.label ?? 'download only'
    r.viewer_url = hit?.url ?? null
  }
  return rows
}

export const AGE_BUCKETS: Record<string, number> = {
  '24h': 24 * 60 * 60 * 1000,
  '7d':  7 * 24 * 60 * 60 * 1000,
  '30d': 30 * 24 * 60 * 60 * 1000,
}
