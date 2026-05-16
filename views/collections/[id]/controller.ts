// Collection detail view — `/collections/:id`.
//
// Reads `route.params.id()` (the shell pre-registers `/collections/:id`
// in apps/shell/src/main.ts so the param resolves on first paint).
// Pulls the collection schema + items via `ship.collections.detail`.
// Tabs: Schema (field list), Rows (table of stored data), JSONC (raw
// pretty-printed schema). New-row writes through the documented REST
// route `/api/v1/collections/<id>/items` — same surface the index view
// uses.

import { controller, signal, derived, live, unwrap, defineRoutes } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

defineRoutes([{ pattern: '/collections/:id' }])

interface SchemaField {
  name: string
  field_type: string
  required?: boolean
  values?: string[]
  default?: unknown
}

interface CollectionDetail {
  schema: {
    id: string
    name: string
    fields: SchemaField[]
    builtin: boolean
  }
  items: Array<{ id: string; data?: Record<string, unknown> }>
  total_items: number
}

export default controller<Scope>({
  scope: ({ ship, route }) => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const s = ship as any
    const collectionId = derived(() => route.params.id() ?? '')
    const colDetail = live(() => s.collections.detail({ id: collectionId() }))
    const tab = signal<'schema' | 'rows' | 'jsonc'>('schema')
    const newRowOpen = signal(false)
    const rowDraft = signal<Record<string, unknown>>({})
    const drawerError = signal('')

    const detailValue = derived<CollectionDetail | null>(() => {
      const v = colDetail() as { kind: string; value?: CollectionDetail }
      return v.kind === 'ok' && v.value ? v.value : null
    })

    const schemaFieldsView = derived(() => {
      const d = unwrap(colDetail) as CollectionDetail | undefined
      const fields = d?.schema?.fields ?? []
      return fields.map(f => ({
        ...f,
        values_pretty: (f.values ?? []).join(' | '),
      }))
    })

    const sourcePath = derived(() => {
      const d = detailValue()
      if (!d) return ''
      return d.schema.builtin
        ? 'built-in (runtime)'
        : `.ship/collections/${d.schema.id}.jsonc`
    })

    const fieldsSummary = derived(() => {
      const d = detailValue()
      if (!d) return ''
      const n = d.schema.fields.length
      return `${n} field${n === 1 ? '' : 's'}`
    })

    const jsoncSource = derived(() => {
      const d = detailValue()
      if (!d) return ''
      const out = {
        id: d.schema.id,
        name: d.schema.name,
        fields: d.schema.fields.map(f => {
          const row: Record<string, unknown> = { name: f.name, type: f.field_type }
          if (f.required) row.required = true
          if (f.values && f.values.length > 0) row.values = f.values
          if (f.default !== undefined && f.default !== null) row.default = f.default
          return row
        }),
      }
      return JSON.stringify(out, null, 2)
    })

    return {
      collectionId, colDetail, detailValue,
      tab, newRowOpen, rowDraft, drawerError,
      schemaFieldsView, sourcePath, fieldsSummary, jsoncSource,
    }
  },
  on: {
    setDraftField: (state, name: string, value: unknown) => {
      state.rowDraft = { ...(state.rowDraft as Record<string, unknown>), [name]: value }
    },
    openNewRow: (state) => {
      state.rowDraft = {}
      state.drawerError = ''
      state.newRowOpen = true
    },
    closeNewRow: (state) => {
      state.newRowOpen = false
      state.drawerError = ''
    },
    saveNewRow: async (state) => {
      const id = state.collectionId as string
      if (!id) { state.drawerError = 'No collection id'; return }
      const res = await fetch(`/api/v1/collections/${id}/items`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({ data: state.rowDraft }),
      })
      if (!res.ok) {
        state.drawerError = await res.text().catch(() => 'Request failed')
        return
      }
      state.newRowOpen = false
      state.rowDraft = {}
      state.drawerError = ''
    },
  },
})
