import { controller, signal } from '@ship/ui-runtime'
import type { Signal } from '@ship/ui-runtime'
import type { Scope } from './sui-breadcrumb.sui'

interface Item {
  slot: string
}

export default controller<Scope>({
  scope: () => {
    const items = signal<Item[]>([])
    const lastSlot = signal<string>('')
    return { items, lastSlot }
  },

  mount(state, host) {
    const scope = state as unknown as Scope
    const items = scope.items as unknown as Signal<Item[]>
    const lastSlot = scope.lastSlot as unknown as Signal<string>

    const sync = () => {
      const children = Array.from(
        host.querySelectorAll<Element>(':scope > sui-breadcrumb-item'),
      )
      const next: Item[] = children.map((el, i) => {
        const slot = `i${i}`
        el.setAttribute('slot', slot)
        return { slot }
      })
      lastSlot.set(next.length > 0 ? next[next.length - 1]!.slot : '')
      items.set(next)
    }

    sync()
    const observer = new MutationObserver(sync)
    observer.observe(host, { childList: true })

    return () => observer.disconnect()
  },
})
