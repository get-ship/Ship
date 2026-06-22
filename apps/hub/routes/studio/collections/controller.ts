// Hub version of apps/shell/routes/studio/collections/controller.ts
//
// Client seam changes from shell:
//   - `ship` + `ops` are AMBIENT — pulled off `ctx.clients.<name>` (the
//     host's `entry.gen.ts` constructs them from `src/lib/client.ts`
//     and the deck-runtime maps `clients.<name>` into view scope). The
//     relative `src/lib/client` import is gone; the client lives in
//     the view mount context, not a per-view module path.
//   - `collections.list` → LIVE `ship.collections.list()` (registered resolver).
//   - `collections.detail` → unary `ops.collections.get({ id })`. The
//     connect service has no `detail` rpc; the schema+item-count
//     read is `Get` (proto), which the connect client surfaces as
//     `get` returning `CollectionDetail { id, schemaJson: bytes, itemCount }`.
//     `ship.collections.detail` SSE-404s (no registered mux resolver
//     for the `collections.detail` tuple), and the SDK has no
//     wrapper for `get`. A `colDetailValue` derived decodes the
//     `schemaJson` bytes and projects to the controller's
//     `ColDetailShape` (`name`/`builtin`/`fields`/`total_items`).
//     Items are NOT returned by `Get` — the rows tab degrades to
//     an empty state until a JS-friendly items op lands (see
//     `.artifacts/hub-studio-fixes-notes.md`).
//   - Mutations → `ship.collections.{create,update_schema,create_item,
//     update_item,delete_item}` (the SDK's snake_case wrappers, which
//     adapt to the connect client internally and keep this controller
//     from hand-rolling the `dataJson`/`schemaJson` bytes-encoding).
import { controller, signal, derived, live } from '@ship/sui-runtime'
import { route as routeFactory } from '@ship/deck/route'
import type { LiveSignal, ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

type SchemaInput = { name: string; type: string; required?: boolean; values?: string[] }

type SchemaField = {
  name: string
  field_type: string
  required?: boolean
  values?: string[]
}

type ColRow = {
  id: string
  name?: string
  field_count: number
  fields?: SchemaField[]
  builtin?: boolean
  item_count?: number
  status_field?: string | null
  status_values?: string[]
  status_counts?: Record<string, number>
  last_activity?: string | null
}

// Shape exposed after the wire's `ops.collections.get` is projected in
// `colDetailValue`. `items` is always empty here — the proto's `Get`
// returns only schema + count; rows need a separate items op the
// connect surface does not expose in a JS-friendly form.
type ColDetailShape = {
  schema: {
    name: string
    builtin: boolean
    fields: SchemaField[]
  }
  items: Item[]
  total_items: number
}

type Example = {
  id: string
  name: string
  icon: string
  description: string
  fields: Record<string, { type: string; required?: boolean; values?: string[] }>
}

const EXAMPLES: Example[] = [
  {
    id: 'reading_list',
    name: 'Reading list',
    icon: 'book-open',
    description: 'Track articles, papers, books to read.',
    fields: {
      title: { type: 'string', required: true },
      url: { type: 'string' },
      status: { type: 'enum', values: ['todo', 'in-progress', 'done'] },
      notes: { type: 'markdown' },
    },
  },
  {
    id: 'onboarding_plan',
    name: 'Onboarding plan',
    icon: 'list-check',
    description: 'Checklist for ramping into a new project.',
    fields: {
      step: { type: 'string', required: true },
      owner: { type: 'string' },
      status: { type: 'enum', values: ['todo', 'in-progress', 'blocked', 'done'] },
      due: { type: 'date' },
    },
  },
  {
    id: 'experiments',
    name: 'Experiments',
    icon: 'beaker',
    description: 'Track hypotheses, outcomes, follow-ups.',
    fields: {
      hypothesis: { type: 'string', required: true },
      status: { type: 'enum', values: ['todo', 'in-progress', 'blocked', 'done', 'cancelled'] },
      result: { type: 'markdown' },
    },
  },
]

function refIconFor(value: string): string {
  if (value.startsWith('agent:')) return 'cpu'
  if (value.startsWith('skill:')) return 'sparkles'
  if (value.startsWith('pipeline:')) return 'git-branch'
  if (value.startsWith('artifact:')) return 'file'
  if (value.startsWith('job:')) return 'play'
  if (value.startsWith('http')) return 'external-link'
  return 'link'
}

function refLabelFor(value: string): string {
  return value.length > 64 ? value.slice(0, 64) + '…' : value
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
):
  | { kind: 'status'; value: string; class_name: string; raw: typeof item }
  | { kind: 'ref'; value: string; icon: string; label: string }
  | { kind: 'text'; value: string }
{
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
    return { kind: 'ref', value: raw, icon: refIconFor(raw), label: refLabelFor(raw) }
  }
  if (raw === null || raw === undefined) return { kind: 'text', value: '' }
  if (typeof raw === 'object') return { kind: 'text', value: JSON.stringify(raw) }
  return { kind: 'text', value: String(raw) }
}

function shortDate(iso: string): string {
  if (!iso) return ''
  const then = Date.parse(iso)
  if (Number.isNaN(then)) return iso.slice(0, 10)
  const delta = (Date.now() - then) / 1000
  if (delta < 60) return 'just now'
  if (delta < 3600) return `${Math.round(delta / 60)}m ago`
  if (delta < 86_400) return `${Math.round(delta / 3600)}h ago`
  if (delta < 604_800) return `${Math.round(delta / 86_400)}d ago`
  return new Date(then).toISOString().slice(0, 10)
}
export default controller({
  scope: (ctx: ViewScope<Scope>) => {
    // AMBIENT clients (host-owned): the entry.gen.ts bootstrap constructs
    // `ship` + `ops` from `src/lib/client.ts` and injects them as
    // `clients.ship` / `clients.ops`. Reading off `ctx.clients` keeps the
    // view free of any relative `../../../lib/client` import.
    const ship = ctx.clients.ship
    const ops = ctx.clients.ops
    const route = routeFactory()
    const activeId = derived(() => route.query.collection() || '')
    // LIVE: collections.list has a registered live resolver.
    const collections = live(() => ship.collections.list())
    // UNARY: collections.get is the real Connect rpc (the proto's `Get`
    // returns `CollectionDetail { id, schemaJson: bytes, itemCount }`).
    // `ship.collections.detail` SSE-404s (no mux resolver for the
    // `collections.detail` tuple), and there is no SDK `get` wrapper.
    // The schema comes as JSON bytes (`TextEncoder(JSON.stringify(SchemaDoc))`);
    // the `colDetailValue` derived decodes + projects below.
    const colDetail = live(() => ops.collections.get({ id: activeId() })) as LiveSignal<unknown>
    // Project the wire's `CollectionDetail` to the controller's
    // `ColDetailShape`. Defensive on every step so a malformed
    // payload degrades to null rather than throwing — the live
    // signal hands `error` envelopes to the template otherwise.
    const colDetailValue = derived<ColDetailShape | null>(() => {
      const r = colDetail()
      if (r.kind !== 'ok') return null
      const v = r.value as { id?: string; schemaJson?: Uint8Array | string; itemCount?: number } | null | undefined
      if (!v) return null
      let schema: { id?: string; title?: string; system?: boolean; fields?: Record<string, SchemaField> } | null = null
      const raw = v.schemaJson
      if (raw instanceof Uint8Array) {
        try { schema = JSON.parse(new TextDecoder().decode(raw)) } catch { schema = null }
      } else if (typeof raw === 'string') {
        try { schema = JSON.parse(raw) } catch { schema = null }
      }
      if (!schema) return null
      const fields = schema.fields
        ? Object.entries(schema.fields).map(([name, f]) => ({
            name,
            field_type: (f as { type?: string }).type ?? 'string',
            ...((f as { required?: boolean }).required === undefined ? {} : { required: (f as { required?: boolean }).required }),
            values: Array.isArray((f as { values?: unknown }).values) ? ((f as { values?: unknown }).values as string[]) : undefined,
          }))
        : []
      return {
        schema: {
          name: schema.title ?? schema.id ?? v.id ?? activeId(),
          builtin: schema.system === true,
          fields,
        },
        items: [],
        total_items: typeof v.itemCount === 'number' ? v.itemCount : 0,
      }
    })

    const indexQuery = signal('')
    const statusFilter = signal('')


    const activeCollection = derived<ColRow | null>(() => {
      const v = collections() as { kind: string; value?: { items: ColRow[] } }
      if (v.kind !== 'ok' || !v.value) return null
      return v.value.items.find(c => c.id === activeId()) ?? null
    })

    const hasStatusField = derived(() => {
      const detail = colDetailValue()
      const f = detail?.schema?.fields?.find(x => x.name === 'status')
      return !!(f && f.field_type === 'enum' && (f.values?.length ?? 0) > 0)
    })

    const statusValues = derived<string[]>(() => {
      return colDetailValue()?.schema?.fields?.find(x => x.name === 'status')?.values ?? []
    })

    const filteredIndex = derived<Array<ColRow & { activity_label: string; status_summary: Array<{ value: string; count: number }> }>>(() => {
      const v = collections() as { kind: string; value?: { items: ColRow[] } }
      if (v.kind !== 'ok' || !v.value) return []
      const q = indexQuery().trim().toLowerCase()
      const rows = q
        ? v.value.items.filter(c =>
            (c.name ?? '').toLowerCase().includes(q) || c.id.toLowerCase().includes(q),
          )
        : v.value.items
      return rows.map(c => ({
        ...c,
        activity_label: c.last_activity ? shortDate(c.last_activity) : 'no activity',
        status_summary: (c.status_values ?? []).map(value => ({
          value,
          count: (c.status_counts ?? {})[value] ?? 0,
        })),
      }))
    })

    type CellView =
      | { kind: 'status'; value: string; class_name: string; raw: Item }
      | { kind: 'ref'; value: string; icon: string; label: string }
      | { kind: 'text'; value: string }
    type RowView = { id: string; raw: Item; cells: CellView[] }

    const filteredRows = derived<RowView[]>(() => {
      const detail = colDetailValue()
      const items = detail?.items ?? []
      const fields = detail?.schema?.fields ?? []
      const filter = statusFilter()
      const filtered = filter
        ? items.filter(r => (r.data?.['status'] as string | undefined) === filter)
        : items
      return filtered.map(item => ({
        id: item.id,
        raw: item,
        cells: fields.map(f => buildCell(f, item)),
      }))
    })

    const schemaFieldsView = derived(() => {
      const fields = colDetailValue()?.schema?.fields ?? []
      return fields.map(f => ({ ...f, values_pretty: (f.values ?? []).join(' | ') }))
    })

    const examples = derived(() => EXAMPLES)

    const tab = signal('rows')
    const newRowOpen = signal(false)
    const editOpen = signal(false)
    const editId = signal('')
    const newColOpen = signal(false)
    const rowDraft = signal<Record<string, unknown>>({})
    const editDraft = signal<Record<string, unknown>>({})
    const newColDraftId = signal('')
    const newColDraftName = signal('')
    const drawerError = signal('')

    // The SDK's `ship.collections.*` methods are the snake_case wrappers
    // around the connect client's camelCase rpcs; they handle the
    // `dataJson`/`schemaJson` bytes-encoding internally so the
    // controller can pass plain objects. (`ops` only exposes the
    // raw connect surface — no friendly wrappers for the
    // collections writes, so the snake_case call shape lives here.)
    const createItem = (a: { collection: string; data: Record<string, unknown> }) =>
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      (ship as any).collections.create_item(a)
    const updateItem = (a: { collection: string; item_id: string; data: Record<string, unknown> }) =>
      (ship as any).collections.update_item(a)
    const deleteItem = (a: { collection: string; item_id: string }) =>
      (ship as any).collections.delete_item(a)
    const updateSchema = (a: {
      id: string
      fields: Array<{ name: string; type: string; required?: boolean; values?: string[] }>
    }) => (ship as any).collections.update_schema(a)
    const createCol = (a: { id: string; name?: string; fields?: Record<string, unknown> }) =>
      (ship as any).collections.create(a)

    return {
      activeId, activeCollection, collections, colDetail, colDetailValue,
      schemaFieldsView, tab,
      indexQuery, filteredIndex, statusFilter, statusValues, hasStatusField, filteredRows,
      examples,
      newRowOpen, editOpen, editId, newColOpen,
      rowDraft, editDraft, newColDraftId, newColDraftName,
      drawerError,
      createItem, updateItem, deleteItem, updateSchema, createCol,
    }
  },
  on: {
    setDraftField: (s, which: 'row' | 'edit', name: string, value: unknown) => {
      const key: 'rowDraft' | 'editDraft' = which === 'row' ? 'rowDraft' : 'editDraft'
      s[key] = { ...s[key], [name]: value }
    },
    openEditRow: (s, item: { id: string; data?: Record<string, unknown> }) => {
      s.editId = item.id
      s.editDraft = { ...(item.data ?? {}) }
      s.drawerError = ''
      s.editOpen = true
    },
    saveNewRow: async (s) => {
      const id = s.activeId
      try {
        await s.createItem({ collection: id, data: s.rowDraft })
      } catch (err) {
        s.drawerError = err instanceof Error ? err.message : 'Request failed'
        return
      }
      s.newRowOpen = false
      s.rowDraft = {}
      s.drawerError = ''
    },
    saveEdit: async (s) => {
      const colId = s.activeId
      const rowId = s.editId
      try {
        await s.updateItem({ collection: colId, item_id: rowId, data: s.editDraft })
      } catch (err) {
        s.drawerError = err instanceof Error ? err.message : 'Request failed'
        return
      }
      s.editOpen = false
      s.editDraft = {}
      s.drawerError = ''
    },
    deleteRow: async (s) => {
      const colId = s.activeId
      const rowId = s.editId
      try {
        await s.deleteItem({ collection: colId, item_id: rowId })
      } catch { /* swallow — UI reflects via SSE refresh */ }
      s.editOpen = false
      s.editDraft = {}
      s.drawerError = ''
    },
    removeSchemaField: async (s, fieldName: string) => {
      // Use colDetailValue (already decoded) rather than unwrapping the LiveSignal.
      const detail = s.colDetailValue as ColDetailShape | null
      const fields = detail?.schema?.fields ?? []
      const fs: SchemaInput[] = fields
        .filter(x => x.name !== fieldName)
        .map(x => ({
          name: x.name,
          type: x.field_type,
          ...(x.required === undefined ? {} : { required: x.required }),
          values: x.values ?? [],
        }))
      const colId = s.activeId
      try {
        await s.updateSchema({ id: colId, fields: fs })
      } catch { /* surface via existing error path on next render */ }
    },
    cycleStatus: async (s, item: { id: string; data?: Record<string, unknown> }) => {
      const values = s.statusValues
      if (!values || values.length === 0) return
      const current = (item.data?.['status'] as string | undefined) ?? ''
      const idx = values.indexOf(current)
      const next = values[(idx + 1) % values.length]
      const colId = s.activeId
      try {
        await s.updateItem({
          collection: colId, item_id: item.id,
          data: { ...(item.data ?? {}), status: next },
        })
      } catch { /* SSE will reflect actual state on success/failure */ }
    },
    createCollection: async (s) => {
      const id = s.newColDraftId.trim()
      if (!id) { s.drawerError = 'Collection ID is required'; return }
      try {
        await s.createCol({
          id,
          name: s.newColDraftName.trim() || id,
          fields: {},
        })
      } catch (err) {
        s.drawerError = err instanceof Error ? err.message : 'Request failed'
        return
      }
      s.newColOpen = false
      s.newColDraftId = ''
      s.newColDraftName = ''
      s.drawerError = ''
      s.navigate('/studio/collections?collection=' + id)
    },
    createFromExample: async (s, exampleId: string) => {
      const ex = EXAMPLES.find(e => e.id === exampleId)
      if (!ex) return
      try {
        await s.createCol({
          id: ex.id,
          name: ex.name,
          fields: ex.fields,
        })
      } catch (err) {
        s.drawerError = err instanceof Error ? err.message : 'Request failed'
        return
      }
      s.navigate('/studio/collections?collection=' + ex.id)
    },
  },
})
