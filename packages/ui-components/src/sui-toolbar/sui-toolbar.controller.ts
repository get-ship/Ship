import { controller } from '@ship/sui-runtime'
import type { Scope } from './sui-toolbar.sui'

export default controller<Scope>({
  mount(state, host) {
    const scope = state as unknown as Scope

    if (!host.hasAttribute('role')) host.setAttribute('role', 'toolbar')
    if (!host.hasAttribute('aria-orientation')) host.setAttribute('aria-orientation', 'horizontal')

    const focusable = (): HTMLElement[] => {
      const slot = scope.defaultSlot() as HTMLSlotElement | null
      const els = (slot?.assignedElements({ flatten: true }) ?? []) as HTMLElement[]
      return els.filter((el) => !el.hasAttribute('disabled'))
    }

    const onKey = (e: KeyboardEvent) => {
      if (e.key !== 'ArrowRight' && e.key !== 'ArrowLeft') return
      const items = focusable()
      if (!items.length) return
      const active =
        (e.composedPath().find((n) => items.includes(n as HTMLElement)) as HTMLElement) || null
      const i = active ? items.indexOf(active) : -1
      const next =
        e.key === 'ArrowRight'
          ? (i + 1) % items.length
          : (i - 1 + items.length) % items.length
      items[next]?.focus()
      e.preventDefault()
    }

    host.addEventListener('keydown', onKey)
    return () => host.removeEventListener('keydown', onKey)
  },
})
