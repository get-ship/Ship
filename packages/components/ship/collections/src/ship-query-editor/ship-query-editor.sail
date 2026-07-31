<controller>
import { controller, derived, live, unwrapOr } from '@ship/sail-runtime'
import type { ViewScope } from '@ship/sail-runtime'
import { ship } from '@ship/sdk-client'
import type { Scope } from './ship-query-editor.sail'

// <ship-query-editor> — a plain editor with field-name completion from
// the schema. The full Kysely-TS autocomplete is deferred per spec
// ("waits on @ship/db toolchain"); for now we only suggest field names
// derived from the active collection's schema.
//
// On every input we emit ship-query-change { ast, sql } so a paired
// runner (or named-query save flow) can react. The "ast" is a
// minimal captured shape { collection, source }; "sql" is the same
// string (the runner compiles to SQL internally).

interface SchemaShape {
  name: string
  fields: Array<{ name: string; type: string }>
}

const EMPTY = { schema: { name: '', fields: [] } as SchemaShape }

interface Suggestion {
  name: string
  type: string
}

// Lightweight token-based completion: when the cursor is at the end of
// the textarea and the trailing token is a partial identifier, we look
// up matching field names. This is a v0 — the rich completion lives
// behind the @ship/db toolchain.
function lastIdentifier(text: string): string {
  const m = /([A-Za-z_][A-Za-z0-9_]*)$/.exec(text)
  return m?.[1] ?? ''
}

export default controller(({ value, collection }: ViewScope<Scope>, host) => {

    const state = live<unknown>(() => ship.collections.state(collection() ? { active: collection() } : {}))
    const schema = derived<SchemaShape | null>(() => {
      const s = unwrapOr(state, EMPTY) as { schema?: SchemaShape }
      return s.schema ?? null
    })

    const suggestions = derived<Suggestion[]>(() => {
      const token = lastIdentifier(value())
      if (token.length < 1) return []
      const s = schema()
      if (!s) return []
      return s.fields
        .filter((f) => f.name.toLowerCase().startsWith(token.toLowerCase()))
        .slice(0, 8)
        .map((f) => ({ name: f.name, type: f.type }))
    })

    const statusText = derived<string>(() => {
      const v = value().trim()
      if (v.length === 0) return 'empty'
      return `len ${v.length} · ${collection()}`
    })

    // Re-emit on user input. The textarea is host-owned; listening on
    // the host's `input` event keeps the wire simple and matches the
    // spec's "captured query" contract.
    const onInput = (): void => {
      const v = value()
      host?.dispatchEvent(
        new CustomEvent('ship-query-change', {
          detail: { ast: { collection: collection(), source: v }, sql: v },
          bubbles: true,
          composed: true,
        }),
      )
    }
    host?.addEventListener('input', onInput)

    function acceptSuggestion(s: Suggestion): (ev: Event) => void {
      return (ev: Event): void => {
        ev.preventDefault()
        const current = value()
        const token = lastIdentifier(current)
        const next = token
          ? current.slice(0, current.length - token.length) + s.name
          : current + s.name
        // value is a Signal<string> in the scope (textarea bind:value).
        value.set(next)
        host?.dispatchEvent(
          new CustomEvent('ship-query-change', {
            detail: { ast: { collection: collection(), source: next }, sql: next },
            bubbles: true,
            composed: true,
          }),
        )
      }
    }

    return {
      state, schema, suggestions, statusText, acceptSuggestion,
      unmount: (): void => {
        host?.removeEventListener('input', onInput)
      },
    }
})
</controller>

<!--
  <ship-query-editor> — @ship/collections · write a query against a
  collection, with field/op autocomplete from the schema. Per spec
  the rich Kysely-TS autocomplete is deferred (waits on @ship/db
  toolchain); today this is a plain editor with field-name
  completion from the schema.

  Emits ship-query-change { ast, sql } on every edit so a paired
  runner (or named-query save flow) can react. The "ast" is a
  minimal shape the runner understands; the "sql" is the compiled
  string shown back in the editor footer.

  Tag prefix `ship-*` marks this component as depending on the Ship
  runtime: mounting without a host that has called setShipClient()
  will throw on the first SDK access.
-->
<!-- <prop name="collection" type="string" default="" /> -->
<!-- <prop name="value" type="string" default="SELECT * FROM items" /> -->

<!-- style block removed: use sx / external CSS -->

<div>
  <span>Query</span>
  <span data-test="collection">{collection()}</span>
</div>

<div data-test="editor">
  <textarea
    data-test="textarea"
    placeholder="SELECT * FROM items"
    aria-label="Query editor"
    bind:value={value}
  ></textarea>
  <div>
    <span data-test="status">{statusText()}</span>
  </div>
</div>

<if cond={suggestions().length > 0}>
  <div data-test="suggestions">
    <div role="listbox">
      <for as="s" of={suggestions()} key={s.name}>
        <div role="option" data-test="suggestion" aria-selected="false" on:mousedown={() => acceptSuggestion(s)}>
          <strong>{s.name}</strong>
          <span> · {s.type}</span>
        </div>
      </for>
    </div>
  </div>
</if>
