<controller>
// Hub version of apps/shell/routes/studio/collections/controller.ts
//
// Client seam changes from shell:
//   - `ship` + `ops` are AMBIENT — pulled off `ctx.<name>` (the
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
import { controller, signal, derived, live } from '@ship/sail-runtime'
import { route as routeFactory } from '@ship/deck-router'
import type { LiveSignal, ViewScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'

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
    const ship = ctx.ship
    const ops = ctx.ops
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
</controller>

<data name="collections" source="ship.collections.list()" live />

<div data-test="studio-view">
  <ship-studio-rail></ship-studio-rail>
  <main data-test="studio-content">
<div>

  <if cond={!activeId()}>
    <div>
      <div>
        <div>
          <h1>Collections</h1>
          <div>
            Typed buckets for tracking specs, skills, artifacts, jobs, or anything else — stored in <span>~/.ship/data.db</span>.
          </div>
        </div>
        <div>
          <button size="sm" tone="primary" variant="solid" on:click={() => newColOpen.set(true)}>
            <icon slot="leading" name="plus"></icon>New collection
          </button>
        </div>
      </div>
      <if cond={collections().kind === 'ok' && collections().value && collections().value.items.length === 0}>
        <empty-state role="status">
          <span slot="title">No collections yet</span>
          <span slot="description">Pick a template to start, or click &ldquo;New collection&rdquo; for a blank one.</span>
        </empty-state>
        <div aria-label="Example collections">
          <for as="ex" of={examples()} key={ex.id}>
            <button type="button" aria-label={'Create ' + ex.name + ' collection'}
              on:click={() => createFromExample(ex.id)}>
              <span><icon name={ex.icon}></icon></span>
              <span>{ex.name}</span>
              <span>{ex.description}</span>
            </button>
          </for>
        </div>
      <else-if cond={collections().kind === 'ok' && collections().value}>
        <div>
          <input value={indexQuery()} placeholder="Filter collections by name…"
            on:input={(e) => indexQuery.set(e.target.value)}>
            <icon slot="leading" name="search" size="sm"></icon>
          
          <span>
            {filteredIndex().length} of {collections().value.items.length}
          </span>
        </div>
        <if cond={filteredIndex().length === 0}>
          <div role="status">No collections match &ldquo;{indexQuery()}&rdquo;.</div>
        </if>
        <div>
          <for as="c" of={filteredIndex()} key={c.id}>
            <a href={'/studio/collections?collection=' + c.id}
              on:click={(e) => { e.preventDefault(); navigate('/studio/collections?collection=' + c.id) }}>
              <row gap="2" align="center">
                <span><icon name="database"></icon></span>
                <div>
                  <div>{c.name || c.id}</div>
                  <div>{c.builtin ? 'built-in' : '.ship/collections/' + c.id + '.jsonc'}</div>
                </div>
                <span>
                  {c.item_count ?? 0} item{(c.item_count ?? 0) !== 1 ? 's' : ''}
                </span>
              </row>
              <if cond={c.status_summary && c.status_summary.length > 0}>
                <row gap="1">
                  <for as="sv" of={c.status_summary} key={sv.value}>
                    <span>
                      <span aria-hidden="true"></span>
                      <span>{sv.value}</span>
                      <span>{sv.count}</span>
                    </span>
                  </for>
                </row>
              <else-if cond={c.fields && c.fields.length > 0}>
                <row gap="1">
                  <for as="f" of={c.fields.slice(0, 5)} key={f.name}>
                    <span>
                      <badge variant="soft" size="sm">{f.type}</badge>
                      <span>{f.name}</span>
                      <if cond={f.required}><span>*</span></if>
                    </span>
                  </for>
                  <if cond={c.fields.length > 5}>
                    <span>+{c.fields.length - 5} more</span>
                  </if>
                </row>
              </else-if></if>
              <div>
                <span>
                  <icon name="columns" size="sm"></icon>
                  {c.field_count} field{c.field_count !== 1 ? 's' : ''}
                </span>
                <span aria-hidden="true">·</span>
                <span>
                  <icon name="clock" size="sm"></icon>
                  {c.activity_label}
                </span>
              </div>
            </a>
          </for>
        </div>
      </else-if></if>
    </div>

  <else>
    <div>
      <aside>
        <div>
          <span>Collections</span>
          <span title="New collection" style="cursor:pointer"
            on:click={() => newColOpen.set(true)}>
            <icon name="plus" size="sm"></icon>
          </span>
        </div>
        <if cond={collections().kind === 'ok' && collections().value}>
          <if cond={collections().value.items.length === 0}><div role="status"><span slot="title">No collections defined</span></div></if>
          <for as="c" of={collections().value.items} key={c.id}>
            <a data-is-active={c.id === activeId()}
              href={'/studio/collections?collection=' + c.id}
              on:click={(e) => { e.preventDefault(); navigate('/studio/collections?collection=' + c.id) }}>
              <span><icon name="database" size="sm"></icon></span>
              <span>{c.name || c.id}</span>
              <span>{c.item_count ?? 0}</span>
            </a>
            <if cond={c.id === activeId()}>
              <div>
                <span></span>
                <span>Default view</span>
              </div>
            </if>
          </for>
        </if>
      </aside>

      <div>
        <div>
          <row gap="2" align="center">
            <span><icon name="database" size="sm"></icon></span>
            <if cond={colDetailValue()}>
              <span>{colDetailValue()?.schema?.name || activeId()}</span>
              <span>{activeId()}</span>
              <span>{colDetailValue()?.items?.length ?? 0} of {colDetailValue()?.total_items ?? 0}</span>
            </if>
          </row>
          <div>
            <if cond={hasStatusField()}>
              <toggle-group variant="segmented" type="single" size="sm" value={statusFilter()} on:sail-change={(e) => statusFilter.set(e.detail.value)}>
                <toggle value="">All</toggle>
                <for as="sv" of={statusValues()} key={sv}>
                  <toggle value={sv}>{sv}</toggle>
                </for>
              </toggle-group>
              <span aria-hidden="true"></span>
            </if>
            <toggle-group variant="segmented" type="single" size="sm" value={tab()} on:change={(e) => tab.set(e.detail.value)}>
              <toggle value="rows"><icon name="list-check" size="sm"></icon>Rows</toggle>
              <toggle value="schema"><icon name="columns" size="sm"></icon>Schema</toggle>
            </toggle-group>
            <span aria-hidden="true"></span>
            <button size="sm" tone="primary" variant="solid"
              on:click={() => { rowDraft.set({}); newRowOpen.set(true) }}>
              <icon slot="leading" name="plus"></icon>New row
            </button>
          </div>
        </div>

        <if cond={tab() === 'rows'}>
          <div>
            
              <while-loading of="colDetail">
              <div role="status" aria-live="polite" aria-busy="true" style="padding:var(--space-4);color:var(--muted-foreground)">Loading&hellip;</div>

              </while-loading><while-error as="err" of="colDetail">
              <error-state role="alert" title="Couldn't load collection" data-test="col-detail-error">
                {err.message}
                <button slot="action" tone="primary" variant="solid" on:click={() => window.location.reload()}>Retry</button>
              </error-state>

              </while-error><if cond={colDetail().kind === 'ok'}>
<if cond={colDetailValue()}>
              <if cond={filteredRows().length === 0 && (colDetailValue()?.items?.length ?? 0) === 0}>
                <empty-state role="status" style="padding:var(--space-8)">
                  <span slot="title">No items in {colDetailValue()?.schema?.name || activeId()}</span>
                  <span slot="description">Click &ldquo;New row&rdquo; to add the first item.</span>
                </empty-state>
              <else-if cond={filteredRows().length === 0}>
                <div role="status">
                  No items with status &ldquo;{statusFilter()}&rdquo;.
                </div>
              </else-if><else>
                <table>
                  <thead>
                    <tr>
                      <th>id <span aria-label="sorted ascending">↑</span></th>
                      <for as="f" of={colDetailValue()?.schema?.fields ?? []} key={f.name}><th>{f.name}</th></for>
                      <th></th>
                    </tr>
                  </thead>
                  <tbody>
                    <for as="row" of={filteredRows()} key={row.id}>
                      <tr>
                        <td>{row.id}</td>
                        <for as="cell" of={row.cells} key={cell.kind + ':' + cell.value}>
                          <if cond={cell.kind === 'status'}>
                            <td>
                              <button type="button"
                                aria-label={'Cycle status for ' + row.id}
                                on:click={(e) => { e.stopPropagation(); cycleStatus(row.raw) }}>
                                <span aria-hidden="true"></span>
                                <span>{cell.value}</span>
                              </button>
                            </td>
                          <else-if cond={cell.kind === 'ref'}>
                            <td>
                              <span>
                                <icon name={cell.icon} size="xs"></icon>
                                <span>{cell.label}</span>
                              </span>
                            </td>
                          </else-if><else>
                            <td>{cell.value}</td>
                          </else></if>
                        </for>
                        <td>
                          <button size="sm" variant="ghost" aria-label={'Edit row ' + row.id}
                            on:click={() => openEditRow(row.raw)}>
                            <icon name="edit" size="xs"></icon>
                          </button>
                        </td>
                      </tr>
                    </for>
                  </tbody>
                </table>
              </else></if>
            </if>
            </if>
          </div>
        </if>

        <if cond={tab() === 'schema'}>
          <div>
            <if cond={colDetailValue()}>
              <div>
                <div>
                  <div>
                    <h3>{colDetailValue()?.schema?.name || activeId()}</h3>
                    <div>
                      {colDetailValue()?.schema?.builtin
                        ? 'built-in'
                        : '.ship/collections/' + activeId() + '.jsonc'}
                    </div>
                  </div>
                  <span>
                    {colDetailValue()?.schema?.fields?.length ?? 0} field{(colDetailValue()?.schema?.fields?.length ?? 0) !== 1 ? 's' : ''}
                  </span>
                </div>
                <if cond={colDetailValue()?.schema?.builtin}>
                  <div>
                    <icon name="lock" size="sm"></icon>
                    Built-in collections ship with the runtime. Fields are listed for reference only.
                  </div>
                </if>
                <column gap="1">
                  <for as="f" of={schemaFieldsView()} key={f.name}>
                    <div>
                      <span>{f.field_type}</span>
                      <span>{f.name}</span>
                      <if cond={f.required}><span>required</span></if>
                      <if cond={f.values && f.values.length > 0}>
                        <span>{f.values_pretty}</span>
                      </if>
                      <if cond={!colDetailValue()?.schema?.builtin}><button size="sm" variant="ghost" aria-label="Remove field" on:click={() => removeSchemaField(f.name)}><icon name="trash" size="xs"></icon></button></if>
                    </div>
                  </for>
                </column>
              </div>
            </if>
          </div>
        </if>
      </div>
    </div>
  </else></if>

  <dialog open={newRowOpen()} size="md"
    on:sail-close={() => { newRowOpen.set(false); drawerError.set('') }}>
    <span slot="title">New row</span>
    <if cond={colDetailValue()}>
      <span slot="description">Add a new item to {colDetailValue()?.schema?.name || activeId()}</span>
      <column gap="3">
        <for as="f" of={colDetailValue()?.schema?.fields ?? []} key={f.name}>
          <column gap="1">
            <label>
              {f.name}<if cond={f.required}><span>*</span></if>
              <span>{f.field_type}</span>
            </label>
            <if cond={f.field_type === 'boolean'}>
              <switch checked={rowDraft()[f.name]}
                on:change={(e) => setDraftField('row', f.name, e.target.checked)}></switch>
            <else-if cond={f.field_type === 'markdown'}>
              <textarea rows="4" placeholder={f.name + '…'}
                value={rowDraft()[f.name] ?? ''}
                on:input={(e) => setDraftField('row', f.name, e.target.value)}></textarea>
            </else-if><else>
              <input type={f.field_type === 'number' ? 'number' : 'text'}
                value={rowDraft()[f.name] ?? ''} placeholder={f.name}
                on:input={(e) => setDraftField('row', f.name, e.target.value)}>
            </else></if>
          </column>
        </for>
        <if cond={drawerError()}><div>{drawerError()}</div></if>
      </column>
      <div slot="footer">
        <button variant="ghost" on:click={() => { newRowOpen.set(false); drawerError.set('') }}>Cancel</button>
        <button tone="primary" variant="solid" on:click={saveNewRow}>Save</button>
      </div>
    </if>
  </dialog>

  <dialog open={editOpen()} size="md"
    on:sail-close={() => { editOpen.set(false); drawerError.set('') }}>
    <span slot="title">Edit row</span>
    <if cond={colDetailValue()}>
      <span slot="description">Editing {editId()}</span>
      <column gap="3">
        <for as="f" of={colDetailValue()?.schema?.fields ?? []} key={f.name}>
          <column gap="1">
            <label>
              {f.name}<if cond={f.required}><span>*</span></if>
              <span>{f.field_type}</span>
            </label>
            <if cond={f.field_type === 'boolean'}>
              <switch checked={editDraft()[f.name]}
                on:change={(e) => setDraftField('edit', f.name, e.target.checked)}></switch>
            <else-if cond={f.field_type === 'markdown'}>
              <textarea rows="4" placeholder={f.name + '…'}
                value={editDraft()[f.name] ?? ''}
                on:input={(e) => setDraftField('edit', f.name, e.target.value)}></textarea>
            </else-if><else>
              <input type={f.field_type === 'number' ? 'number' : 'text'}
                value={editDraft()[f.name] ?? ''} placeholder={f.name}
                on:input={(e) => setDraftField('edit', f.name, e.target.value)}>
            </else></if>
          </column>
        </for>
        <if cond={drawerError()}><div>{drawerError()}</div></if>
      </column>
      <div slot="footer">
        <button tone="destructive" variant="soft" style="margin-right:auto"
          on:click={deleteRow}>Delete</button>
        <button variant="ghost" on:click={() => { editOpen.set(false); drawerError.set('') }}>Cancel</button>
        <button tone="primary" variant="solid" on:click={saveEdit}>Save</button>
      </div>
    </if>
  </dialog>

  <dialog open={newColOpen()} size="md"
    on:sail-close={() => { newColOpen.set(false); drawerError.set('') }}>
    <span slot="title">New collection</span>
    <span slot="description">Define a typed dataset stored in data.db</span>
    <column gap="3">
      <column gap="1">
        <label>
          Collection ID <span>*</span>
          <span>snake_case</span>
        </label>
        <input bind:value={newColDraftId} placeholder="my_collection" required
          style="font-family:var(--font-mono)">
      </column>
      <column gap="1">
        <label>Display name</label>
        <input bind:value={newColDraftName} placeholder="My Collection">
      </column>
      <if cond={drawerError()}><div>{drawerError()}</div></if>
    </column>
    <div slot="footer">
      <button variant="ghost" on:click={() => { newColOpen.set(false); drawerError.set('') }}>Cancel</button>
      <button tone="primary" variant="solid" on:click={createCollection}>Create collection</button>
    </div>
  </dialog>

</div>
  </main>
</div>
