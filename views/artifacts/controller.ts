import { controller, derived, live, signal, unwrap } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

interface ArtifactRow {
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
  created_at: string
}

interface ArtifactsListResponse { ok: boolean; data: ArtifactRow[]; count: number }
type CollectionRow = { id: string; name?: string | null }

const MIME_BUCKETS: Array<{ value: string; prefix: string }> = [
  { value: 'image', prefix: 'image/' },
  { value: 'text', prefix: 'text/' },
  { value: 'json', prefix: 'application/json' },
]

function bucketFor(mime: string): string {
  for (const b of MIME_BUCKETS) if (mime.startsWith(b.prefix)) return b.value
  return ''
}

function previewKindFor(mime: string): 'image' | 'html' | 'text' | 'binary' {
  if (mime.startsWith('image/')) return 'image'
  if (mime === 'text/html') return 'html'
  if (mime.startsWith('text/') || mime === 'application/json' || mime === 'application/xml') return 'text'
  return 'binary'
}

function formatSize(bytes: number): string {
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

function decorate(rows: ArtifactRow[]): ArtifactRow[] {
  for (const r of rows) {
    r.size_label = formatSize(r.size)
    r.id_short = (r.id ?? '').slice(0, 12)
    r.created_short = formatShortDate(r.created_at)
  }
  return rows
}

export default controller<Scope>({
  scope: ({ ship }) => {
    const items = signal<ArtifactRow[]>([])
    const loading = signal<boolean>(true)
    const loadError = signal<string>('')

    const mimeFilter = signal<string>('')
    const collectionFilter = signal<string>('')
    const searchQuery = signal<string>('')

    const detailItem = signal<ArtifactRow | null>(null)
    const detailOpen = signal<boolean>(false)
    const previewText = signal<string>('')
    const previewLoading = signal<boolean>(false)
    const previewError = signal<string>('')

    const uploadOpen = signal<boolean>(false)
    const uploadFiles = signal<File[]>([])
    const uploadCollection = signal<string>('')
    const uploadError = signal<string>('')
    const uploading = signal<boolean>(false)

    const collectionsLive = live(() => ship.collections.list())
    const collectionOptions = derived<CollectionRow[]>(() => {
      const unwrapped = unwrap(collectionsLive) as { items?: CollectionRow[] } | undefined
      return unwrapped?.items ?? []
    })

    const visibleItems = derived<ArtifactRow[]>(() => {
      const q = searchQuery().trim().toLowerCase()
      const mb = mimeFilter()
      const cf = collectionFilter()
      return items().filter((r) => {
        if (mb && bucketFor(r.mime) !== mb) return false
        if (cf && (r.collection_id ?? '') !== cf) return false
        if (q) {
          const hay = `${r.name ?? ''} ${r.id} ${r.mime}`.toLowerCase()
          if (!hay.includes(q)) return false
        }
        return true
      })
    })

    const totalCount = derived(() => items().length)
    const visibleCount = derived(() => visibleItems().length)
    const hasActiveFilters = derived(() =>
      mimeFilter() !== '' || collectionFilter() !== '' || searchQuery().trim() !== '',
    )
    const detailUrl = derived(() => detailItem() ? `/api/v1/artifacts/${detailItem()!.id}` : '')
    const previewKind = derived(() => detailItem() ? previewKindFor(detailItem()!.mime) : 'binary')

    async function refresh(): Promise<void> {
      loading.set(true)
      loadError.set('')
      try {
        const resp = await fetch('/api/v1/artifacts?limit=200', { credentials: 'same-origin' })
        if (!resp.ok) { loadError.set(`HTTP ${resp.status}`); items.set([]); return }
        const body = (await resp.json()) as ArtifactsListResponse
        const rows = Array.isArray(body.data) ? body.data : []
        items.set(decorate(rows))
      } catch (e) {
        loadError.set((e as Error).message)
        items.set([])
      } finally {
        loading.set(false)
      }
    }

    async function loadPreview(item: ArtifactRow): Promise<void> {
      const kind = previewKindFor(item.mime)
      if (kind !== 'text') return
      previewLoading.set(true)
      previewError.set('')
      previewText.set('')
      try {
        const resp = await fetch(`/api/v1/artifacts/${item.id}`, { credentials: 'same-origin' })
        if (!resp.ok) { previewError.set(`HTTP ${resp.status}`); return }
        const text = await resp.text()
        previewText.set(text.length > 50_000 ? text.slice(0, 50_000) + '\n…(truncated)' : text)
      } catch (e) {
        previewError.set((e as Error).message)
      } finally {
        previewLoading.set(false)
      }
    }

    void refresh()

    return {
      items, loading, loadError,
      mimeFilter, collectionFilter, searchQuery,
      detailItem, detailOpen, previewText, previewLoading, previewError,
      uploadOpen, uploadFiles, uploadCollection, uploadError, uploading,
      collectionsLive, collectionOptions,
      visibleItems, totalCount, visibleCount, hasActiveFilters, detailUrl, previewKind,
      refresh, loadPreview,
    }
  },

  on: {
    setMimeFilter: (s, v: string) => { s.mimeFilter = v },
    setCollectionFilter: (s, v: string) => { s.collectionFilter = v },
    clearFilters: (s) => { s.mimeFilter = ''; s.collectionFilter = ''; s.searchQuery = '' },

    openDetail: async (s, id: string) => {
      const row = (s.items as ArtifactRow[]).find((r) => r.id === id) ?? null
      if (!row) return
      s.detailItem = row
      s.detailOpen = true
      s.previewText = ''
      s.previewError = ''
      await (s as any).loadPreview(row)
    },
    closeDetail: (s) => {
      s.detailOpen = false
      s.detailItem = null
      s.previewText = ''
      s.previewError = ''
    },

    openUpload: (s) => {
      s.uploadOpen = true
      s.uploadError = ''
      s.uploadFiles = []
      s.uploadCollection = ''
    },
    closeUpload: (s) => {
      if (s.uploading) return
      s.uploadOpen = false
      s.uploadError = ''
      s.uploadFiles = []
    },
    onUploadFiles: (s, files: File[]) => {
      s.uploadFiles = Array.isArray(files) ? files : []
      s.uploadError = ''
    },
    onUploadCollection: (s, v: string) => { s.uploadCollection = v },
    submitUpload: async (s) => {
      const files = s.uploadFiles as File[]
      if (!files.length) { s.uploadError = 'Choose a file to upload.'; return }
      s.uploading = true
      s.uploadError = ''
      try {
        const fd = new FormData()
        fd.append('file', files[0]!)
        const col = (s.uploadCollection as string).trim()
        if (col) fd.append('collection_id', col)
        const res = await fetch('/api/v1/artifacts', { method: 'POST', credentials: 'same-origin', body: fd })
        if (!res.ok) {
          const txt = await res.text().catch(() => '')
          s.uploadError = txt || `Upload failed (HTTP ${res.status})`
          return
        }
        s.uploadOpen = false
        s.uploadFiles = []
        s.uploadCollection = ''
        await (s as any).refresh()
      } catch (e) {
        s.uploadError = (e as Error).message
      } finally {
        s.uploading = false
      }
    },
  },
})
