import { controller, derived, live, signal, unwrap } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'
import { AGE_BUCKETS, bucketFor, decorate, previewKindFor, type ArtifactRow } from './decorate'

interface ArtifactsListResponse { ok: boolean; data: ArtifactRow[]; count: number }
type CollectionRow = { id: string; name?: string | null }

export default controller({
  scope: (ctx: ViewScope<Scope>) => {
    const { ship } = ctx
    // Shell-injected navigator. Runtime's stock `navigate` only does
    // pushState; the shell's wrapper also triggers mountRoute.
    const navigate = (ctx as { navigate?: (p: string) => void }).navigate
      ?? ((p: string) => { if (typeof window !== 'undefined') window.location.href = p })
    const items = signal<ArtifactRow[]>([])
    const loading = signal<boolean>(true)
    const loadError = signal<string>('')

    const mimeFilter = signal<string>('')
    const collectionFilter = signal<string>('')
    const actorFilter = signal<string>('')
    const ageFilter = signal<string>('')
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
    const dragActive = signal<boolean>(false)

    const collectionsLive = live(() => ship.collections.list())
    const collectionOptions = derived<CollectionRow[]>(() => {
      const unwrapped = unwrap(collectionsLive) as { items?: CollectionRow[] } | undefined
      return unwrapped?.items ?? []
    })

    const actorOptions = derived<string[]>(() => {
      const set = new Set<string>()
      for (const r of items()) if (r.owner_actor) set.add(r.owner_actor)
      return Array.from(set).sort()
    })

    const visibleItems = derived<ArtifactRow[]>(() => {
      const q = searchQuery().trim().toLowerCase()
      const mb = mimeFilter()
      const cf = collectionFilter()
      const af = actorFilter()
      const age = ageFilter()
      const ageMs = age ? (AGE_BUCKETS[age] ?? 0) : 0
      const cutoff = ageMs > 0 ? Date.now() - ageMs : 0
      return items().filter((r) => {
        if (mb && bucketFor(r.mime) !== mb) return false
        if (cf && (r.collection_id ?? '') !== cf) return false
        if (af && (r.owner_actor ?? '') !== af) return false
        if (cutoff > 0) {
          const t = Date.parse(r.created_at)
          if (!Number.isFinite(t) || t < cutoff) return false
        }
        if (q) {
          const hay = `${r.name ?? ''} ${r.id} ${r.mime} ${r.owner_actor ?? ''}`.toLowerCase()
          if (!hay.includes(q)) return false
        }
        return true
      })
    })

    const totalCount = derived(() => items().length)
    const visibleCount = derived(() => visibleItems().length)
    const hasActiveFilters = derived(() =>
      mimeFilter() !== ''
      || collectionFilter() !== ''
      || actorFilter() !== ''
      || ageFilter() !== ''
      || searchQuery().trim() !== '',
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
        const text = await ship.artifacts.get({ id: item.id })
        previewText.set(text.length > 50_000 ? text.slice(0, 50_000) + '\n…(truncated)' : text)
      } catch (e) {
        previewError.set((e as Error).message)
      } finally {
        previewLoading.set(false)
      }
    }

    void refresh()

    const uploadArtifact = (args: { file: File; collection_id?: string }) =>
      ship.artifacts.upload(args)

    async function activateRow(id: string): Promise<void> {
      const row = items().find((r) => r.id === id) ?? null
      if (!row) return
      if (row.viewer_url) {
        navigate(row.viewer_url)
        return
      }
      detailItem.set(row)
      detailOpen.set(true)
      previewText.set('')
      previewError.set('')
      await loadPreview(row)
    }

    async function uploadDirect(file: File): Promise<void> {
      uploading.set(true)
      uploadError.set('')
      try {
        await uploadArtifact({ file })
        await refresh()
      } catch (e) {
        uploadError.set((e as Error).message)
        uploadOpen.set(true)
      } finally {
        uploading.set(false)
      }
    }

    return {
      items, loading, loadError,
      mimeFilter, collectionFilter, actorFilter, ageFilter, searchQuery,
      detailItem, detailOpen, previewText, previewLoading, previewError,
      uploadOpen, uploadFiles, uploadCollection, uploadError, uploading, dragActive,
      collectionsLive, collectionOptions, actorOptions,
      visibleItems, totalCount, visibleCount, hasActiveFilters, detailUrl, previewKind,
      refresh, loadPreview, uploadArtifact, uploadDirect, activateRow,
    }
  },

  on: {
    setMimeFilter: (s, v: string) => { s.mimeFilter = v },
    setCollectionFilter: (s, v: string) => { s.collectionFilter = v },
    setActorFilter: (s, v: string) => { s.actorFilter = v },
    setAgeFilter: (s, v: string) => { s.ageFilter = v },
    setSearchQuery: (s, v: string) => { s.searchQuery = v },
    clearFilters: (s) => {
      s.mimeFilter = ''
      s.collectionFilter = ''
      s.actorFilter = ''
      s.ageFilter = ''
      s.searchQuery = ''
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
        const col = (s.uploadCollection as string).trim()
        const args: { file: File; collection_id?: string } = { file: files[0]! }
        if (col) args.collection_id = col
        await (s.uploadArtifact as (a: { file: File; collection_id?: string }) => Promise<unknown>)(args)
        s.uploadOpen = false
        s.uploadFiles = []
        s.uploadCollection = ''
        await s.refresh()
      } catch (e) {
        s.uploadError = (e as Error).message
      } finally {
        s.uploading = false
      }
    },

    // dragover MUST preventDefault — without it the browser rejects
    // the drop and navigates to the file URL.
    onDragOver: (s, e: DragEvent) => {
      const dt = e.dataTransfer
      if (!dt || !Array.from(dt.types || []).includes('Files')) return
      e.preventDefault()
      dt.dropEffect = 'copy'
      s.dragActive = true
    },
    onDragLeave: (s, e: DragEvent) => {
      const t = e.currentTarget as HTMLElement | null
      const related = e.relatedTarget as Node | null
      if (t && related && t.contains(related)) return
      s.dragActive = false
    },
    onDrop: async (s, e: DragEvent) => {
      e.preventDefault()
      s.dragActive = false
      const files = e.dataTransfer?.files
      if (!files || files.length === 0) return
      const first = files[0]
      if (!first) return
      await s.uploadDirect(first)
    },
  },
})
