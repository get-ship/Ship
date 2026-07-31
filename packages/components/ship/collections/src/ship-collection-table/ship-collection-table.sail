<controller>
import { controller, derived, live, unwrapOr } from '@ship/sail-runtime'
import type { LiveSource, ViewScope } from '@ship/sail-runtime'
import { ship } from '@ship/sdk-client'
import type { Scope } from './ship-collection-table.sail'

// Schema-driven columns for the data explorer.
//
// ship.collections.state({collection}) returns the schema + total_items;
// ship.collections.items({collection}) returns paginated rows. The SDK
// types the row payload as `unknown[]` (the field set is dynamic), so
// we project to a flat Row shape the underlying <sui-table> consumes.

interface ColumnDef {
  field: string
  label: string
}

interface SchemaField {
  name: string
  type: string
  required?: boolean
}

interface SchemaShape {
  name: string
  fields: SchemaField[]
}

interface StatePayload {
  schema: SchemaShape
  total_items: number
}

interface ItemRecord {
  id: string
  data: Record<string, unknown>
}

interface RowRecord {
  id: string
  [key: string]: unknown
}

const EMPTY_STATE: StatePayload = { schema: { name: '', fields: [] }, total_items: 0 }
const EMPTY_ITEMS = { items: [] as unknown[] }

function typeLabel(type: string): string {
  switch (type) {
    case 'text': return 'string'
    case 'integer': return 'int'
    case 'real': return 'real'
    case 'bool': return 'bool'
    case 'timestamp': return 'date'
    case 'json': return 'json'
    default: return type
  }
}

export default controller(({ collection }: ViewScope<Scope>, host) => {

    const state = live(() => ship.collections.state(collection() ? { active: collection() } : {}))
    const items = live(() => ship.collections.items({ collection: collection() }) as unknown as LiveSource<typeof EMPTY_ITEMS>)

    const stateValue = derived<StatePayload>(
      () => unwrapOr(state, EMPTY_STATE) as StatePayload,
    )

    // Columns: id column first (substrate envelope), then one column per
    // schema field. The label is the human title; the type annotates the
    // header so readers know the shape at a glance.
    const columnDefs = derived<ColumnDef[]>(() => {
      const cols: ColumnDef[] = [{ field: 'id', label: 'id' }]
      const s = stateValue().schema
      if (s && Array.isArray(s.fields)) {
        for (const f of s.fields) {
          cols.push({ field: f.name, label: `${f.name} · ${typeLabel(f.type)}` })
        }
      }
      return cols
    })

    // Flatten item envelopes to row shape {id, ...data}. Daemon returns
    // {items: [{id, data: {...}}, ...]}; the table primitive reads fields
    // off the row, not the nested data object.
    const rows = derived<RowRecord[]>(() => {
      const payload = unwrapOr(items, EMPTY_ITEMS) as { items: unknown[] }
      const out: RowRecord[] = []
      for (const raw of payload.items ?? []) {
        if (raw && typeof raw === 'object' && 'id' in (raw as Record<string, unknown>)) {
          const item = raw as ItemRecord
          const flat: RowRecord = { id: item.id }
          if (item.data && typeof item.data === 'object') {
            for (const [k, v] of Object.entries(item.data)) flat[k] = v
          }
          out.push(flat)
        }
      }
      return out
    })

    // Row click → emit ship-collection-row-select. The <sui-table>
    // primitive doesn't speak our event names, so we delegate to the
    // host's shadow root via composed event listening on `click`.
    const onRowClick = (e: Event): void => {
      if (!host) return
      for (const node of e.composedPath()) {
        if (!(node instanceof Element)) continue
        const tr = node.closest('tr[data-row-id]') as HTMLElement | null
        if (tr) {
          const id = tr.getAttribute('data-row-id')
          if (id) {
            host.dispatchEvent(
              new CustomEvent('ship-collection-row-select', {
                detail: { collection: collection(), id },
                bubbles: true,
                composed: true,
              }),
            )
          }
          return
        }
      }
    }
    host?.addEventListener('click', onRowClick)

    return {
      state, items, rows, columnDefs, stateValue,
      unmount: (): void => {
        host?.removeEventListener('click', onRowClick)
      },
    }
})
</controller>

<!--
  <ship-collection-table> — @ship/collections · the data explorer.
  Reads ship.collections.state({collection}) for the schema (columns)
  and ship.collections.items({collection}) for paginated rows. Emits
  ship-collection-row-select { collection, id } on row click.

  Column derivation is schema-driven: each field in the schema becomes
  a column on the underlying <sail-table>. The `id` column is added by
  default (the substrate envelope) so rows are selectable.

  Live updates are gated by the daemon's Watch; until the SDK exposes
  ship.collections.watch, this surface re-queries on each data call.
  See KNOWN_GAPS.md.

  Tag prefix `ship-*` marks this component as depending on the Ship
  runtime: mounting without a host that has called setShipClient()
  will throw on the first SDK access.
-->
<!-- <prop name="collection" type="string" default="" /> -->

<!-- style block removed: use sx / external CSS -->

<while-loading of="state">
  <spinner shape="line" size="lg"></spinner>
</while-loading>
<while-error as="err" of="state">
  <error-state title="Couldn't load collection">{err.message}</error-state>
</while-error>
<if cond={state().kind === 'ok' && state().value}>
  <while-loading of="items">
    <spinner shape="line" size="md"></spinner>
  </while-loading>
  <while-error as="err" of="items">
    <error-state title="Couldn't load items" variant="inline">{err.message}</error-state>
  </while-error>
  <if cond={items().kind === 'ok'}>
    <if cond={rows().length === 0}>
      <div data-test="empty">No items in {state().value.schema.name}.</div>
    <else>
      <sail-table
        variant="data"
        rows={rows()}
        columns={columnDefs()}
        hoverable
        striped
        data-test="grid"
      ></sail-table>
    </else></if>
    <div data-test="footer">
      <span>{rows().length} / {state().value.total_items}</span>
      <span>{collection()}</span>
    </div>
  </if>
</if>
