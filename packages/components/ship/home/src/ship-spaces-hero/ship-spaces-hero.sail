<scope>
  props: {
    spaces = '[]'
    limit = 8
  }
</scope>
<controller>
// <ship-spaces-hero> — @ship/home · the spaces grid on the hub `/`.
//
// The home dashboard's spaces surface. Pure-derived presenter — the
// source of truth is the `spaces` attribute (JSON-encoded list). The
// hub controller passes `[]` today (the list-all-spaces SDK method is
// a known BACKEND-GAP; see .artifacts/hub-buildout-flags.md), and the
// empty-state takes over. When the SDK lands, the controller switches
// to `live(() => ship.spaces.list())` and the host pipes the result
// into the attribute; this controller stays stable.

import type { Scope } from './ship-spaces-hero.sail'

interface SpaceRow {
  id: string
  name: string
  path: string
}

function parseSpaces(raw: string): SpaceRow[] {
  if (!raw) return []
  try {
    const v = JSON.parse(raw) as unknown
    if (!Array.isArray(v)) return []
    const out: SpaceRow[] = []
    for (const it of v) {
      if (it && typeof it === 'object') {
        const row = it as Partial<SpaceRow>
        if (
          typeof row.id === 'string' &&
          typeof row.name === 'string' &&
          typeof row.path === 'string'
        ) {
          out.push({ id: row.id, name: row.name, path: row.path })
        }
      }
    }
    return out
  } catch {
    return []
  }
}

export default controller((ctx: Scope, host?: Element) => {
  const rows = derived<SpaceRow[]>(() => {
    const all = parseSpaces(ctx.spaces())
    const n = Number.isFinite(ctx.limit()) ? Math.max(0, Math.floor(ctx.limit())) : 8
    return all.slice(0, n)
  })
  const countLabel = derived<string>(() => {
    const n = rows().length
    return n === 0
      ? 'no spaces registered'
      : n === 1
        ? '1 space'
        : `${n} spaces`
  })

  const onOpen = (e: MouseEvent, id: string): void => {
    e.preventDefault()
    // ship-space-open — the host routes to the space surface. We
    // preventDefault so the hub's router (rather than a full-page
    // navigation to /spaces/:id) handles the click.
    host?.dispatchEvent(
      new CustomEvent('ship-space-open', {
        detail: { id },
        bubbles: true,
        composed: true,
      }),
    )
  }
  const onAdd = (): void => {
    host?.dispatchEvent(
      new CustomEvent('ship-space-add', {
        bubbles: true,
        composed: true,
      }),
    )
  }

  return { rows, countLabel, onOpen, onAdd }
})
</controller>

<!--
  <ship-spaces-hero> — @ship/home · the spaces grid on the hub `/`.

  The home dashboard surfaces the user's spaces (a Ship `space` is an
  independent workspace — its own daemon-backed directory). Today the
  Connect SDK exposes only `ship.spaces.config({ id })` (per-space
  runtime config) — there is no list-all-spaces method. This is a
  known `BACKEND-GAP` recorded in .artifacts/hub-buildout-flags.md.

  Until the gap closes, the host passes the spaces list via the
  `spaces` attribute as a JSON-encoded array of `{ id, name, path }`
  rows (the host's controller may source it from a future SDK call or
  from shell-scoped discovery; today's controller passes `[]`). When
  the attribute is empty / unparseable, the component renders the
  empty-state with a "register a space" affordance.

  When the SDK ships a list method, the controller switches to
  `live(() => ship.spaces.list())` and pipes the result into the
  attribute; this controller stays stable.

  Prop / event
  ─────────────────────────────────────────────────────────────────────
  spaces  (json string, '[]')   JSON-encoded list of {id,name,path}
  limit   (number, 8)           max cards rendered
  ─────────────────────────────────────────────────────────────────────

  Emits (bubbling, composed):
    ship-space-open  { detail: { id: string } }   click on a card
    ship-space-add   {}                          click on the empty-state CTA
-->

<section data-test="home-spaces">
  <h2>Spaces</h2>
  <span>{countLabel()}</span>
  <button type="button" data-action="add" on:click={onAdd} title="Register a space">+ register</button>
</section>

<if {rows().length === 0}>
  <empty-state variant="inline" size="md" tone="muted" data-test="home-spaces-empty">
    <icon slot="icon" name="layout-grid"></icon>
    <span slot="title">No spaces yet</span>
    A Ship <b>space</b> is an independent workspace with its own daemon-backed directory. Register one to get started.
    <button slot="action" type="button" on:click={onAdd}>
      Register a space
    </button>
  </empty-state>
{else =>
  <div data-test="home-spaces-grid">
    <for {rows() as s} key={s.id}>
      <a href={'/spaces/' + s.id}
         on:click={(e) => onOpen(e, s.id)}
         aria-label={'Open space ' + s.name}>
        <column gap="2">
          <span title={s.name}>{s.name}</span>
          <span title={s.path}>{s.path}</span>
          <row gap="2" align="center">
            <status-dot tone="success" size="sm" label="ready"></status-dot>
            <badge tone="muted" size="sm">space</badge>
          </row>
        </column>
      </a>
    </for>
  </div>
}</if>
