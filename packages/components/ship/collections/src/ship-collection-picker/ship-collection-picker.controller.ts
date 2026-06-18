import { controller, derived, live, signal, unwrapOr } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import { ship } from '@ship/sdk-client'
import type { Scope } from './ship-collection-picker.sui'

// Compact picker over ship.collections.list(). Mirrors the @ship/spaces
// switcher shape: button-as-trigger + listbox of options. Picking a
// row emits `ship-collection-pick { id }` and the host flips the active
// pointer.
interface CollectionRow {
  id: string
  name: string
  item_count: number
  system: boolean
}

const EMPTY = { items: [] as { id: string; name: string | null; item_count: number; builtin?: boolean }[] }

export default controller<Scope>({
  scope: (_ctx, host) => {
    const list = live(() => ship.collections.list())
    const rows = derived<CollectionRow[]>(() =>
      unwrapOr(list, EMPTY).items.map((c) => ({
        id: c.id,
        name: c.name ?? c.id,
        item_count: c.item_count,
        system: Boolean(c.builtin),
      })),
    )
    const open = signal(false)
    const activeLabel = derived(() => {
      const found = rows().find((c) => c.id === (host?.getAttribute('active') ?? ''))
      return found?.name ?? ''
    })

    const close = (): void => { open.set(false) }

    const onClick = (e: Event): void => {
      if (!host) return
      for (const node of e.composedPath()) {
        if (!(node instanceof Element)) continue
        if (node.matches('[data-action="toggle"]')) {
          open.set(!open())
          return
        }
        if (node.matches('.row[data-id]')) {
          const id = node.getAttribute('data-id')
          if (id) {
            host.dispatchEvent(
              new CustomEvent('ship-collection-pick', {
                detail: { id },
                bubbles: true,
                composed: true,
              }),
            )
          }
          close()
          return
        }
      }
    }
    const onDocClick = (e: Event): void => {
      if (!open()) return
      if (!e.composedPath().includes(host)) close()
    }
    const onKey = (e: KeyboardEvent): void => {
      if (e.key === 'Escape' && open()) close()
    }

    host?.addEventListener('click', onClick)
    document.addEventListener('click', onDocClick, true)
    document.addEventListener('keydown', onKey)

    // Reflect the `active` attribute back to the host (consumers can
    // either read it directly or hook the ship-collection-pick event).
    const observer = new MutationObserver(() => {
      const a = host?.getAttribute('active') ?? ''
      const found = rows().find((c) => c.id === a)
      host?.setAttribute('data-label', found?.name ?? '')
    })
    if (host) observer.observe(host, { attributes: true, attributeFilter: ['active'] })

    return {
      list, rows, open, activeLabel,
      unmount: (): void => {
        host?.removeEventListener('click', onClick)
        document.removeEventListener('click', onDocClick, true)
        document.removeEventListener('keydown', onKey)
        observer.disconnect()
      },
    }
  },
})
