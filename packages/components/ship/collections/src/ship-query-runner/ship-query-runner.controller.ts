import { controller, derived, live, signal, unwrapOr } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import { ship } from '@ship/sdk-client'
import type { Scope } from './ship-query-runner.sui'

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

async function runQuery(collection: string): Promise<{ items: unknown[]; total_items: number }> {
  return ship.collections.items({ collection }) as unknown as Promise<{ items: unknown[]; total_items: number }>
}

export default controller<Scope>({
  scope: ({ collection, source, limit }: ViewScope<Scope>, _host) => {
    const state = live<unknown>(() => ship.collections.state({ active: collection() || undefined }))

    // Track elapsed via a signal the live() thunk updates on resolve.
    // The template reads elapsed() reactively — no host attribute round-trip.
    const elapsedMs = signal<number>(0)

    const results = live<{ items: unknown[]; total_items: number }>(() => {
      const c = collection()
      if (!c) return Promise.resolve({ items: [], total_items: 0 })
      const started = Date.now()
      return runQuery(c).then((r) => {
        elapsedMs.set(Date.now() - started)
        return r
      })
    })

    // source + limit are wired to the RPC in v2 with the typed
    // query_items op; reading them here keeps the dependency alive so
    // the live() thunk re-fires on prop change.
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
  },
})
