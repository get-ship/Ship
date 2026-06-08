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
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'
import { ops } from '../../../ops'

defineRoutes([{ pattern: '/studio/collections/:id' }])

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

type CellView =
  | { kind: 'status'; value: string; class_name: string; raw: { id: string; data?: Record<string, unknown> } }
  | { kind: 'ref'; value: string; icon: string; label: string }
  | { kind: 'text'; value: string }

function refIconFor(value: string): string {
  if (value.startsWith('agent:')) return 'cpu'
  if (value.startsWith('skill:')) return 'sparkles'
  if (value.startsWith('pipeline:')) return 'git-branch'
  if (value.startsWith('artifact:')) return 'file'
  if (value.startsWith('job:')) return 'play'
  if (value.startsWith('http')) return 'external-link'
  return 'link'
}

function isRefValue(fieldName: string, value: unknown): value is string {
  if (typeof value !== 'string' || !value) return false
  if (fieldName === 'url' || fieldName === 'href') return true
  if (fieldName === 'ref' || fieldName === 'item' || fieldName === 'target') return true
  return /^(https?:|agent:|skill:|pipeline:|artifact:|job:)/.test(value)
}

function buildCell(
  field: SchemaField,
  item: { id: string; data?: Record<string, unknown> },
): CellView {
  const raw = item.data?.[field.name]
  if (field.name === 'status' && field.field_type === 'enum') {
    const value = typeof raw === 'string' ? raw : ''
    return {
      kind: 'status',
      value: value || '—',
      class_name: 'status-chip status-chip-' + (value || 'unset'),
      raw: item,
    }
  }
  if (isRefValue(field.name, raw)) {
    return {
      kind: 'ref',
      value: raw,
      icon: refIconFor(raw),
      label: raw.length > 64 ? raw.slice(0, 64) + '…' : raw,
    }
  }
  if (raw === null || raw === undefined) return { kind: 'text', value: '' }
  if (typeof raw === 'object') return { kind: 'text', value: JSON.stringify(raw) }
  return { kind: 'text', value: String(raw) }
}

export default controller({
  scope: ({ ship, route }: ViewScope<Scope>) => {
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

    const rowsView = derived<Array<{ id: string; raw: { id: string; data?: Record<string, unknown> }; cells: CellView[] }>>(() => {
      const d = detailValue()
      if (!d) return []
      return d.items.map(item => ({
        id: item.id,
        raw: item,
        cells: d.schema.fields.map(f => buildCell(f, item)),
      }))
    })

    return {
      collectionId, colDetail, detailValue,
      tab, newRowOpen, rowDraft, drawerError,
      schemaFieldsView, sourcePath, fieldsSummary, jsoncSource, rowsView,
    }
  },
  on: {
    setDraftField: (state, name: string, value: unknown) => {
      state.rowDraft = { ...state.rowDraft, [name]: value }
    },
    cycleStatus: async (state, item: { id: string; data?: Record<string, unknown> }) => {
      const detail = state.detailValue
      const statusField = detail?.schema.fields?.find((f: SchemaField) => f.name === 'status')
      const values = statusField?.values ?? []
      if (values.length === 0) return
      const current = (item.data?.['status'] as string | undefined) ?? ''
      const idx = values.indexOf(current)
      const next = values[(idx + 1) % values.length]
      const id = state.collectionId
      await ops.collections.updateItem({
        collection: id,
        id: item.id,
        dataJson: new TextEncoder().encode(JSON.stringify({ ...(item.data ?? {}), status: next })),
      }).catch(() => undefined)
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
      const id = state.collectionId
      if (!id) { state.drawerError = 'No collection id'; return }
      try {
        await ops.collections.createItem({
          collection: id,
          dataJson: new TextEncoder().encode(JSON.stringify(state.rowDraft)),
        })
      } catch (e) {
        state.drawerError = (e as Error).message ?? 'Request failed'
        return
      }
      state.newRowOpen = false
      state.rowDraft = {}
      state.drawerError = ''
    },
  },
})
