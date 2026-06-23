// Collection detail view — `/studio/collections/:id`.
//
// Hub port of `apps/shell/routes/studio/collections/[id]/controller.ts`. The
// shell read `ship.collections.detail` (a live op with NO registered mux
// resolver → mux/subscribe 404 → empty/error). This reads the unary Connect
// rpc `ops.collections.get` ({ id } → CollectionDetail { id, schemaJson: bytes,
// itemCount }) wrapped in `live(() => …)` so the template's `{#async colDetail}`
// block drives loading/error/ok. The schema arrives as JSON bytes
// (`TextEncoder(JSON.stringify(SchemaDoc))`); `detailValue` decodes + projects
// it to the template's `{ schema: { id, name, fields, builtin }, items,
// total_items }` shape — mirroring the decode the hub collections LIST
// controller already does for the same `get` rpc.
//
// Items: `CollectionDetail` from `Get` carries only schema + count, never the
// rows (the proto exposes rows via `Query`, which needs a Kysely AST the
// frontend doesn't build). So the Rows tab degrades to its empty state and
// `total_items` shows the real count from `itemCount`. Mutations
// (createItem/updateItem) are the unary Connect writes off the ambient `ops`
// client; they succeed against the daemon, but the unary read won't reflect a
// new/changed row without a reload (no live re-read on this surface).
//
// Client seam: `ops` is AMBIENT — pulled off `ctx.clients.ops` (the host's
// entry.gen.ts constructs it from src/lib/client.ts and the deck-runtime maps
// `clients.<name>` into view scope). No relative `src/lib/client` import.

import { controller, signal, derived, live } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

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

// Raw schema-doc shape decoded from `CollectionDetail.schemaJson`.
type SchemaDoc = {
  id?: string
  title?: string
  system?: boolean
  fields?: Record<string, { type?: string; required?: boolean; values?: unknown; default?: unknown }>
}

export default controller({
  scope: (ctx: ViewScope<Scope>) => {
    const { route } = ctx
    const ops = ctx.clients.ops
    const collectionId = derived(() => route.params.id() ?? '')
    // UNARY `collections.get` over the ambient Connect client. `live()` wraps
    // the promise so `{#async colDetail}` gets loading/error/ok.
    const colDetail = live(() => ops.collections.get({ id: collectionId() }))
    const tab = signal<'schema' | 'rows' | 'jsonc'>('schema')
    const newRowOpen = signal(false)
    const rowDraft = signal<Record<string, unknown>>({})
    const drawerError = signal('')

    // Decode `schemaJson` bytes and project to the template's CollectionDetail
    // shape. Defensive on every step so a malformed payload degrades to null
    // rather than throwing.
    const detailValue = derived<CollectionDetail | null>(() => {
      const r = colDetail()
      if (r.kind !== 'ok') return null
      const v = r.value as { id?: string; schemaJson?: Uint8Array | string; itemCount?: number } | null | undefined
      if (!v) return null
      let schema: SchemaDoc | null = null
      const raw = v.schemaJson
      if (raw instanceof Uint8Array) {
        try { schema = JSON.parse(new TextDecoder().decode(raw)) as SchemaDoc } catch { schema = null }
      } else if (typeof raw === 'string') {
        try { schema = JSON.parse(raw) as SchemaDoc } catch { schema = null }
      }
      if (!schema) return null
      const fields: SchemaField[] = schema.fields
        ? Object.entries(schema.fields).map(([name, f]) => ({
            name,
            field_type: f.type ?? 'string',
            ...(f.required === undefined ? {} : { required: f.required }),
            values: Array.isArray(f.values) ? (f.values as string[]) : undefined,
            ...(f.default === undefined ? {} : { default: f.default }),
          }))
        : []
      return {
        schema: {
          id: schema.id ?? v.id ?? collectionId(),
          name: schema.title ?? schema.id ?? v.id ?? collectionId(),
          fields,
          builtin: schema.system === true,
        },
        items: [],
        total_items: typeof v.itemCount === 'number' ? v.itemCount : 0,
      }
    })

    const schemaFieldsView = derived(() => {
      const fields = detailValue()?.schema?.fields ?? []
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
      await state.clients.ops.collections.updateItem({
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
        await state.clients.ops.collections.createItem({
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
