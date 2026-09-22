<controller>
// Collection detail view — `/studio/collections/:id`.
//
// Hub port of `apps/shell/routes/studio/collections/[id]/controller.ts`. The
// shell read `ship.collections.detail` (a live op with NO registered mux
// resolver → mux/subscribe 404 → empty/error). This reads the unary Connect
// rpc `ops.collections.get` ({ id } → CollectionDetail { id, schemaJson: bytes,
// itemCount }) wrapped in `live(() => …)` so the template's ``
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
// Client seam: `ops` is AMBIENT — pulled off `ctx.ops` (the host's
// entry.gen.ts constructs it from src/lib/client.ts and the deck-runtime maps
// `clients.<name>` into view scope). No relative `src/lib/client` import.

import { controller, signal, derived, live } from '@ship/sail-runtime'
import type { ViewScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'

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
    const ops = ctx.ops
    const collectionId = derived(() => route.params.id() ?? '')
    // UNARY `collections.get` over the ambient Connect client. `live()` wraps
    // the promise so `` gets loading/error/ok.
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
</controller>

<div>

  <if cond={!collectionId()}>
    <div>
      <empty-state>
        <span slot="title">No collection selected</span>
        <span slot="description">The URL did not include a collection id.</span>
      </empty-state>
    </div>
  <else>

    <div>
      <a href={'/studio/collections'}
        on:click={(e) => { e.preventDefault(); navigate('/studio/collections') }}>
        <icon name="chevron-left" size="sm"></icon>
        <span>Collections</span>
      </a>

      
        <while-loading of="colDetail">
        <div>Loading {collectionId()}&hellip;</div>
      
        </while-loading><if cond={colDetail().kind === 'ok'}>
<if cond={detailValue()}>
        <div>
          <span><icon name="database" size="sm"></icon></span>
          <h1>{detailValue().schema.name || collectionId()}</h1>
          <span>{detailValue().schema.id}</span>
          <if cond={detailValue().schema.builtin}>
            <span><icon name="lock" size="xs"></icon> built-in</span>
          </if>
        </div>
        <div>
          <span>
            <span>source</span>
            <span>{sourcePath()}</span>
          </span>
          <span>
            <span>schema</span>
            <span>{fieldsSummary()}</span>
          </span>
          <span>
            <span>rows</span>
            <span>{detailValue().total_items}</span>
          </span>
        </div>
        <row gap="3" align="center" justify="between">
          <toggle-group variant="segmented" type="single" size="sm" value={tab()} on:sail-change={(e) => tab.set(e.detail.value)}>
            <toggle value="schema"><icon name="columns" size="sm"></icon>Schema</toggle>
            <toggle value="rows"><icon name="list-check" size="sm"></icon>Rows</toggle>
            <toggle value="jsonc"><icon name="file" size="sm"></icon>JSONC</toggle>
          </toggle-group>
          <button size="sm" tone="primary" variant="solid" on:click={openNewRow}>
            <icon slot="leading" name="plus"></icon>New row
          </button>
        </row>
      </if>
      </if>

      <if cond={colDetail().kind === 'error'}>
        <div>
          <icon name="alert-triangle" size="sm"></icon>
          Failed to load collection &ldquo;{collectionId()}&rdquo;.
        </div>
      </if>
    </div>

    <if cond={detailValue()}>
      <div>

        <if cond={tab() === 'schema'}>
          <div>
            <div>
              <h2>Fields</h2>
              <span>{fieldsSummary()}</span>
            </div>
            <if cond={detailValue().schema.builtin}>
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
                </div>
              </for>
            </column>
          </div>
        </if>

        <if cond={tab() === 'rows'}>
          <div>
            <if cond={detailValue().items.length === 0}>
              <empty-state>
                <span slot="title">No rows in {detailValue().schema.name || collectionId()}</span>
                <span slot="description">Click &ldquo;New row&rdquo; to add the first item.</span>
              </empty-state>
            <else>
              <table>
                <thead>
                  <tr>
                    <th>id</th>
                    <for as="f" of={detailValue().schema.fields} key={f.name}><th>{f.name}</th></for>
                  </tr>
                </thead>
                <tbody>
                  <for as="row" of={rowsView()} key={row.id}>
                    <tr>
                      <td>{row.id}</td>
                      <for as="cell" of={row.cells} key={cell.kind + ':' + cell.value}>
                        <if cond={cell.kind === 'status'}>
                          <td>
                            <button type="button"
                              aria-label={'Cycle status for ' + row.id}
                              on:click={() => cycleStatus(row.raw)}>
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
                    </tr>
                  </for>
                </tbody>
              </table>
            </else></if>
          </div>
        </if>

        <if cond={tab() === 'jsonc'}>
          <div>
            <div>
              <h2>Raw config</h2>
              <span>{sourcePath()}</span>
            </div>
            <pre>{jsoncSource()}</pre>
          </div>
        </if>

      </div>
    </if>

    <dialog open={newRowOpen()} size="md" on:sail-close={closeNewRow}>
      <span slot="title">New row</span>
      <if cond={detailValue()}>
        <span slot="description">Add a new item to {detailValue().schema.name || collectionId()}</span>
        <column gap="3">
          <for as="f" of={detailValue().schema.fields} key={f.name}>
            <column gap="1">
              <label>
                {f.name}<if cond={f.required}><span>*</span></if>
                <span>{f.field_type}</span>
              </label>
              <if cond={f.field_type === 'boolean'}>
                <switch checked={rowDraft()[f.name]}
                  on:change={(e) => setDraftField(f.name, e.target.checked)}></switch>
              <else-if cond={f.field_type === 'markdown'}>
                <textarea rows="4" placeholder={f.name}
                  value={rowDraft()[f.name] ?? ''}
                  on:input={(e) => setDraftField(f.name, e.target.value)}></textarea>
              </else-if><else>
                <input type={f.field_type === 'number' ? 'number' : 'text'}
                  value={rowDraft()[f.name] ?? ''}
                  placeholder={f.name}
                  on:input={(e) => setDraftField(f.name, e.target.value)}>
              </else></if>
            </column>
          </for>
          <if cond={drawerError()}><div>{drawerError()}</div></if>
        </column>
        <div slot="footer">
          <button variant="ghost" on:click={closeNewRow}>Cancel</button>
          <button tone="primary" variant="solid" on:click={saveNewRow}>Save</button>
        </div>
      </if>
    </dialog>

  </else></if>

</div>
