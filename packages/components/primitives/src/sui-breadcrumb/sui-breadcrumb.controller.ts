import { controller, signal } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-breadcrumb.sui'

interface Item {
  slot: string
}

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    const items = signal<Item[]>([])
    const lastSlot = signal<string>('')
    return { items, lastSlot }
  },

  mount(state, host) {
    const sync = () => {
      const children = Array.from(
        host.querySelectorAll<Element>(':scope > sui-breadcrumb-item'),
      )
      const next: Item[] = children.map((el, i) => {
        const slot = `i${i}`
        el.setAttribute('slot', slot)
        return { slot }
      })
      state.lastSlot = next.length > 0 ? next[next.length - 1]!.slot : ''
      state.items = next
    }

    sync()
    const observer = new MutationObserver(sync)
    observer.observe(host, { childList: true })

    return () => observer.disconnect()
  },
})
