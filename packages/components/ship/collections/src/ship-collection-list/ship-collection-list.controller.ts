import { controller, derived, live, unwrapOr } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import { ship } from '@ship/sdk-client'
import type { Scope } from './ship-collection-list.sui'

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

export default controller({
  scope: (_ctx: ViewScope<Scope>, host) => {
    const list = live(() => ship.collections.list())
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
  },
})
