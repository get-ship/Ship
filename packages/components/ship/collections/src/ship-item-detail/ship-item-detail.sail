<controller>
import { controller, derived, live, unwrapOr } from '@ship/sail-runtime'
import type { ViewScope } from '@ship/sail-runtime'
import { ship } from '@ship/sdk-client'
import type { Scope } from './ship-item-detail.sail'

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


export default controller(({ collection, id }: ViewScope<Scope>, host) => {

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

    const state = live<unknown>(() => ship.collections.state(collection() ? { active: collection() } : {}))

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
})
</controller>

<!--
  <ship-item-detail> — @ship/collections · one item rendered by its
  typed fields, with a raw-JSON fallback. Calls
  ship.collections.get_item({collection, item_id}) via the controller
  and emits ship-item-edit { collection, id } on Edit click.

  The typed view projects each schema field to a row (label · value).
  The raw view is a <sail-json-view> over the full data envelope — the
  fallback when the type doesn't render cleanly (or the user wants
  to see what shipped).

  Tag prefix `ship-*` marks this component as depending on the Ship
  runtime: mounting without a host that has called setShipClient()
  will throw on the first SDK access.
-->
<!-- <prop name="collection" type="string" default="" /> -->
<!-- <prop name="id" type="string" default="" /> -->
<!-- <prop name="view" values="typed|raw" default="typed" reflect /> -->

<!-- style block removed: use sx / external CSS -->

<while-loading of="item">
  <spinner shape="line" size="lg"></spinner>
  <div></div>
  <spinner shape="line" size="md"></spinner>
</while-loading>
<while-error as="err" of="item">
  <error-state title="Couldn't load item" variant="inline">{err.message}</error-state>
</while-error>
<if cond={item().kind === 'ok' && item().value}>
  <div>
    <span data-test="item-id">{id()}</span>
    <div role="tablist">
      <button type="button" role="tab" data-test="view-typed" aria-pressed={view() === 'typed' ? 'true' : 'false'} on:click={() => setView('typed')}>typed</button>
      <button type="button" role="tab" data-test="view-raw" aria-pressed={view() === 'raw' ? 'true' : 'false'} on:click={() => setView('raw')}>raw</button>
    </div>
    <button type="button" data-test="edit" on:click={edit}>Edit</button>
  </div>
  <if cond={view() === 'raw'}>
    <div data-test="raw">
      <sail-json-view data={rawJson()}></sail-json-view>
    </div>
  <else>
    <dl data-test="rows">
      <if cond={fieldRows().length === 0}>
        <dd>No fields in this schema.</dd>
      </if>
      <for as="f" of={fieldRows()} key={f.name}>
        <dt>{f.name} <span>· {f.type}</span></dt>
        <dd data-test={f.name}>{f.value}</dd>
      </for>
    </dl>
  </else></if>
</if>
