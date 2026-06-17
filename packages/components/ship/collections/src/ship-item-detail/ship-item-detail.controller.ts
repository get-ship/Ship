import { controller, derived, live, unwrapOr } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import { ship } from '@ship/sdk-client'
import type { Scope } from './ship-item-detail.sui'

// <ship-item-detail> — one item rendered by its typed fields + raw fallback.
//
// ship.collections.get_item({collection, item_id}) is a Promise (not a
// LiveSource) per the SDK surface; live() handles Promises by resolving
// them to the `ok` state and rejecting to the `error` state — so the
// template's `{#while-loading of item}` and `{#while-error err of item}`
// branches work without extra wrapping. The runtime re-fires the thunk
// whenever a tracked dep inside it changes (collection() or id()), so
// a new id just reruns the call.

interface ItemEnvelope {
  id: string
  data: Record<string, unknown>
}

interface SchemaShape {
  name: string
  fields: Array<{ name: string; type: string; required?: boolean }>
}


interface FieldRow {
  name: string
  type: string
  value: string
}

const FIELD_TYPES = ['text', 'integer', 'real', 'bool', 'timestamp', 'json'] as const
type FieldType = (typeof FIELD_TYPES)[number]

function isFieldType(t: unknown): t is FieldType {
  return typeof t === 'string' && (FIELD_TYPES as readonly string[]).includes(t)
}

function formatValue(value: unknown, type: FieldType): string {
  if (value === undefined || value === null) return '—'
  if (type === 'bool') return value ? 'true' : 'false'
  if (type === 'json') return JSON.stringify(value)
  if (type === 'timestamp' && typeof value === 'string') return value
  if (typeof value === 'string') return value
  return String(value)
}


export default controller<Scope>({
  scope: ({ collection, id }: ViewScope<Scope>, host) => {
    // live() returns a LiveSignal<{kind, ...}>; the Promise path resolves
    // to `{ kind: 'ok', value }` or `{ kind: 'error', error }`. The
    // `{#while-loading/while-error/if ok}` blocks in the template work
    // directly against that signal.
    const item = live<{ ok: true; data: ItemEnvelope } | null>(() => {
      const c = collection()
      const i = id()
      if (!c || !i) return Promise.resolve(null)
      return ship.collections.get_item({ collection: c, item_id: i })
    })

    const state = live<unknown>(() => ship.collections.state({ active: collection() || undefined }))

    const schema = derived<SchemaShape | null>(() => {
      const s = unwrapOr(state, null) as { schema?: SchemaShape } | null
      return s?.schema ?? null
    })

    const itemValue = derived<ItemEnvelope | null>(() => {
      const v = unwrapOr(item, null) as { ok: true; data: ItemEnvelope } | null
      return v?.data ?? null
    })

    const fieldRows = derived<FieldRow[]>(() => {
      const s = schema()
      const v = itemValue()
      if (!v || !s) return []
      const out: FieldRow[] = []
      for (const f of s.fields) {
        const t: FieldType = isFieldType(f.type) ? f.type : 'text'
        const raw = v.data?.[f.name]
        out.push({ name: f.name, type: f.type, value: formatValue(raw, t) })
      }
      return out
    })

    const rawJson = derived<string>(() => {
      const v = itemValue()
      if (!v) return '{}'
      return JSON.stringify(v.data ?? {}, null, 2)
    })

    function setView(v: 'typed' | 'raw'): () => void {
      return (): void => { host?.setAttribute('view', v) }
    }
    function edit(): void {
      host?.dispatchEvent(
        new CustomEvent('ship-item-edit', {
          detail: { collection: collection(), id: id() },
          bubbles: true,
          composed: true,
        }),
      )
    }


    return { item, itemValue, schema, fieldRows, rawJson, setView, edit }
  },
})
