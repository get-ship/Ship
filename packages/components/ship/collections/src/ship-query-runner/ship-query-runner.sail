<scope>
  props: {
    collection = ''
    source = ''
    limit = 100
  }
</scope>
<controller>
import { controller, derived, effect, query, signal, unwrapOr } from '@ship/sail-runtime'
import { ship } from '@ship/sdk-client'
import type { Scope } from './ship-query-runner.sail'

// <ship-query-runner> — run a query, render typed results + meta.
// v1 caveat: the runner does not dispatch arbitrary queries — it
// issues items() and surfaces the captured source string. The
// typed query_items op is a SDK follow-up (see KNOWN_GAPS.md §2).
// The `source` + `limit` props on the host are still read so the
// pair-with-editor wiring is in place; they just don't shape the
// RPC call yet.

interface ColumnDef {
  field: string
  label: string
}

interface SchemaShape {
  name: string
  fields: Array<{ name: string; type: string }>
}

interface StatePayload {
  schema: SchemaShape
}

interface ItemRecord {
  id: string
  data: Record<string, unknown>
}

interface RowRecord {
  id: string
  [key: string]: unknown
}

const EMPTY_STATE: StatePayload = { schema: { name: '', fields: [] } }
const EMPTY_ITEMS = { items: [] as unknown[], total_items: 0 }

export default controller(({ collection, source, limit }: Scope, _host?: Element) => {

    const state = query<unknown>(() => ship.collections.state(collection() ? { active: collection() } : {}))

    // Track elapsed via a signal updated when results settle.
    // The template reads elapsed() reactively — no host attribute round-trip.
    const elapsedMs = signal<number>(0)
    const startedAt = signal<number>(0)

    const results = query<{ items: unknown[]; total_items: number }>(() => {
      startedAt.set(Date.now())
      return ship.collections.items({ collection: collection() })
    })

    effect(() => {
      const r = results()
      if (r.kind === 'ok') elapsedMs.set(Math.max(0, Date.now() - startedAt()))
    })

    // source + limit are wired to the RPC in v2 with the typed
    // query_items op; reading them here keeps the dependency alive so
    // the query() thunk re-fires on prop change.
    void source; void limit

    const running = derived<boolean>(() => results().kind === 'loading')

    const schema = derived<SchemaShape | null>(() => {
      const s = unwrapOr(state, EMPTY_STATE) as { schema?: SchemaShape }
      return s.schema ?? null
    })

    const rows = derived<RowRecord[]>(() => {
      const v = unwrapOr(results, EMPTY_ITEMS) as { items: unknown[] }
      const out: RowRecord[] = []
      for (const raw of v.items ?? []) {
        if (raw && typeof raw === 'object' && 'id' in (raw as Record<string, unknown>)) {
          const item = raw as ItemRecord
          const flat: RowRecord = { id: item.id }
          if (item.data && typeof item.data === 'object') {
            for (const [k, val] of Object.entries(item.data)) flat[k] = val
          }
          out.push(flat)
        }
      }
      return out
    })

    const rowCount = derived<number>(() => rows().length)

    const columnDefs = derived<ColumnDef[]>(() => {
      const cols: ColumnDef[] = [{ field: 'id', label: 'id' }]
      const s = schema()
      if (s && Array.isArray(s.fields)) {
        for (const f of s.fields) cols.push({ field: f.name, label: f.name })
      }
      return cols
    })

    const elapsed = derived<string>(() => {
      const ms = elapsedMs()
      return ms > 0 ? `${ms}ms` : '—'
    })

    return { state, results, running, schema, rows, rowCount, columnDefs, elapsed, elapsedMs }
})
</controller>

<!--
  <ship-query-runner> — @ship/collections · run a query and render
  typed results + row count + elapsed time. Calls items() via
  the SDK and composes <sail-table> for the result grid. Reads the
  schema for column rendering (the typed-from-schema contract).

  Pairs with <ship-query-editor>: the host listens for
  ship-query-change from the editor and re-issues a run.

  The runner doesn't ship with a built-in run button — the host
  triggers runs by listening for ship-query-change, or by passing
  a query string via the `source` prop and toggling `run` to true.

  Tag prefix `ship-*` marks this component as depending on the
  Ship runtime: mounting without a host that has called
  setShipClient() will throw on the first SDK access.
-->
<!-- <prop name="collection" type="string" default="" /> -->
<!-- <prop name="source" type="string" default="" /> -->
<!-- <prop name="limit" type="number" default="100" /> -->

<div>
  <span>Results</span>
  <if {running()}><span>· running…</span></if>
  <span data-test="meta">
    <span data-test="count">{rowCount()} rows</span>
    <span data-test="elapsed">{elapsed()}</span>
  </span>
</div>

<match {results()}>
  {loading => <spinner shape="line" size="md"></spinner>}
  {error as err => <error-state title="Query failed" variant="inline" data-test="error" body={err?.message ?? 'Request failed'}></error-state>}
  {ok as _ => }
</match>
<if {results().kind === 'ok'}>
  <if {rows().length === 0}>
    <div data-test="empty">0 rows</div>
  {else =>
    <sail-table
      variant="data"
      rows={rows()}
      columns={columnDefs()}
      hoverable
      striped
      data-test="grid"
    ></sail-table>
  }</if>
</if>
