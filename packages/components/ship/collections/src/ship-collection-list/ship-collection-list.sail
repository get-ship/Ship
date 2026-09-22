<scope>
  props: {
    active = ''
  }
</scope>
<controller>
import { controller, derived, query, unwrapOr } from '@ship/sail-runtime'
import { ship } from '@ship/sdk-client'
import type { Scope } from './ship-collection-list.sail'

// ship.collections.list() returns `{ items: CollectionSummary[] }` per
// the SDK surface (see ship-sdk.ts:98). The `name` field on each row
// is optional (the daemon leaves it null for legacy collections), so
// fall back to the `id` for display.
interface CollectionRow {
  id: string
  name: string
  item_count: number
  system: boolean
}

const EMPTY = { items: [] as { id: string; name: string | null; item_count: number; builtin?: boolean }[] }

export default controller((_ctx: Scope, host?: Element) => {

    const list = query(() => ship.collections.list())
    const rows = derived<CollectionRow[]>(() =>
      unwrapOr(list, EMPTY).items.map((c) => ({
        id: c.id,
        name: c.name ?? c.id,
        item_count: c.item_count,
        // `builtin` is the SDK's flag for read-only system collections;
        // a positive `system` mirrors the spec's collection-list contract.
        system: Boolean(c.builtin),
      })),
    )

    function select(id: string, system: boolean): void {
      host?.dispatchEvent(
        new CustomEvent('ship-collection-select', {
          detail: { id, system },
          bubbles: true,
          composed: true,
        }),
      )
    }

    return { list, rows, select }
})
</controller>

<!--
  <ship-collection-list> — @ship/collections · sidebar / picker of the
  space's collections. Calls ship.collections.list() via the controller
  and emits ship-collection-select { id, system } on row click. System
  collections (feedback, annotations) are flagged in the title slot
  so the row reads at a glance.

  Tag prefix `ship-*` (vs `sui-*`) marks this component as depending
  on the Ship runtime: mounting without a host that has called
  setShipClient() will throw on the first SDK access.
-->
<!-- <prop name="active" type="string" default="" /> -->

<match {list()}>
  {loading =>
    <column gap="1" data-test="skeleton">
      <spinner shape="line" size="sm"></spinner>
      <spinner shape="line" size="sm"></spinner>
      <spinner shape="line" size="sm"></spinner>
    </column>
  }
  {error as err => <error-state title="Couldn't load collections" body={err?.message ?? 'Request failed'}></error-state>}
  {ok as _ => }
</match>
<if {list().kind === 'ok'}>
  <column gap="1" role="listbox">
    <for {rows() as c} key={c.id}>
      <button
        type="button"
        data-id={c.id}
        data-test="row"
        role="option"
        aria-selected={c.id === active() ? 'true' : 'false'}
        on:click={() => select(c.id, c.system)}
      >
        <span>{c.name}</span>
        <if {c.system}>
          <span data-test="system">system</span>
        </if>
        <span>{c.item_count}</span>
      </button>
    </for>
  </column>
</if>
