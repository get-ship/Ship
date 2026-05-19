import { controller } from '@ship/sui-runtime'
import type { Scope } from './sui-navigation-item.sui'

export default controller<Scope>({
  mount(state, host) {
    const scope = state as unknown as Scope

    const onClick = (e: Event) => {
      if (scope.disabled()) return
      if (!scope.href()) e.preventDefault()
      host.dispatchEvent(new CustomEvent('sui-nav-select', {
        detail: { value: scope.value() },
        bubbles: true,
        composed: true,
      }))
    }

    ;(host as unknown as { focusLink(): void }).focusLink = () => {
      const el = host.shadowRoot?.querySelector('.link') as HTMLElement | null
      el?.focus()
    }

    host.addEventListener('click', onClick)
    return () => host.removeEventListener('click', onClick)
  },
})
