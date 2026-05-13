import { controller, signal } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

interface ArtifactRow {
  id: string
  name: string
  mime: string
  size: number
  /** Pre-formatted human-readable size ("12.3 KB"). Computed in the
   *  controller because the renderable-subset rejects function calls
   *  with arguments. */
  size_label: string
  path: string
  scope: string
  collection_id?: string | null
  item_id?: string | null
  created_at: string
}

interface ArtifactsListResponse {
  ok: boolean
  data: ArtifactRow[]
  count: number
}

// `ship.artifacts.list()` is not registered in the daemon's OPS catalog
// (see apps/shipd/src/services/schema.rs). Until that surface lands,
// fetch the REST list endpoint directly. Documented in
// .artifacts/sui-arc/shell-launch-priority.md.
export default controller<Scope>({
  scope: () => {
    const items = signal<ArtifactRow[]>([])
    const loading = signal<boolean>(true)
    const loadError = signal<string>('')

    async function refresh(): Promise<void> {
      loading.set(true)
      loadError.set('')
      try {
        const resp = await fetch('/api/v1/artifacts', { credentials: 'same-origin' })
        if (!resp.ok) {
          loadError.set(`HTTP ${resp.status}`)
          items.set([])
          return
        }
        const body = (await resp.json()) as ArtifactsListResponse
        const rows = Array.isArray(body.data) ? body.data : []
        for (const r of rows) r.size_label = formatSize(r.size)
        items.set(rows)
      } catch (e) {
        loadError.set((e as Error).message)
        items.set([])
      } finally {
        loading.set(false)
      }
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

    void refresh()

    return { items, loading, loadError, refresh }
  },
})
