import { controller, effect } from '@ship/sui-runtime'
import type { Scope } from './sui-toggle.sui'

export default controller<Scope>({
  mount(state, host) {
    const scope = state as unknown as Scope

    const stop = effect(() => {
      const btn = scope.buttonEl() as HTMLButtonElement | null
      if (!btn) return
      const onClick = () => {
        if (scope.disabled()) return
        scope.pressed.set(!scope.pressed())
        host.dispatchEvent(new CustomEvent('sui-toggle', {
          detail: { pressed: scope.pressed(), value: scope.value() },
          bubbles: true,
          composed: true,
        }))
      }
      btn.addEventListener('click', onClick)
      return () => btn.removeEventListener('click', onClick)
    })

    // Group calls this to move keyboard focus.
    ;(host as unknown as { focusButton(): void }).focusButton = () => {
      const btn = host.shadowRoot?.querySelector('button') as HTMLButtonElement | null
      btn?.focus()
    }

    return () => stop()
  },
})
