import { controller, effect } from '@ship/ui-runtime'
import type { Scope } from './sui-accordion.sui'

export default controller<Scope>({
  mount(state, host) {
    const scope = state as unknown as Scope

    const getItems = (): Element[] =>
      Array.from(host.querySelectorAll(':scope > sui-accordion-item'))

    const getEnabledItems = (): Element[] =>
      getItems().filter(i => !i.hasAttribute('disabled'))

    const getOpenValues = (): Set<string> => {
      const v = scope.value()
      return new Set(v ? v.split(',').filter(Boolean) : [])
    }

    const syncItems = () => {
      const open = getOpenValues()
      for (const item of getItems()) {
        const val = item.getAttribute('value') || ''
        if (open.has(val)) {
          item.setAttribute('open', '')
        } else {
          item.removeAttribute('open')
        }
      }
    }

    const stopEffect = effect(() => {
      scope.value()
      syncItems()
    })

    const onToggle = (e: Event) => {
      const ev = e as CustomEvent
      ev.stopPropagation()
      const itemVal = ev.detail?.value as string
      const open = getOpenValues()

      if (scope.type() === 'single') {
        scope.value.set(open.has(itemVal) ? '' : itemVal)
      } else {
        if (open.has(itemVal)) {
          open.delete(itemVal)
        } else {
          open.add(itemVal)
        }
        scope.value.set(Array.from(open).join(','))
      }

      host.dispatchEvent(
        new CustomEvent('sui-change', {
          detail: { value: scope.value() },
          bubbles: true,
          composed: true,
        }),
      )
    }

    const onNav = (e: Event) => {
      const ev = e as CustomEvent
      ev.stopPropagation()
      const enabled = getEnabledItems()
      if (!enabled.length) return

      const current = (ev.target as Element | null)?.closest('sui-accordion-item')
      const idx = current ? enabled.indexOf(current) : -1
      let target: Element | undefined

      switch (ev.detail?.direction) {
        case 'next':  target = enabled[(idx + 1) % enabled.length]; break
        case 'prev':  target = enabled[(idx - 1 + enabled.length) % enabled.length]; break
        case 'first': target = enabled[0]; break
        case 'last':  target = enabled[enabled.length - 1]; break
      }

      if (target) {
        const btn = target.shadowRoot?.querySelector('button') as HTMLButtonElement | null
        btn?.focus()
      }
    }

    host.addEventListener('sui-accordion-item-toggle', onToggle)
    host.addEventListener('sui-accordion-nav', onNav)

    return () => {
      stopEffect()
      host.removeEventListener('sui-accordion-item-toggle', onToggle)
      host.removeEventListener('sui-accordion-nav', onNav)
    }
  },
})
