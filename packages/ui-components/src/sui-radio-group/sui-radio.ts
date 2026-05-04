import { controller, effect } from '@ship/ui-runtime'
import type { Scope } from './sui-radio.sui'

export default controller<Scope>({
  mount(state, host) {
    const scope = state as unknown as Scope

    const stop = effect(() => {
      const input = scope.inputEl() as HTMLInputElement | null
      if (!input) return

      const onClick = () => {
        if (scope.disabled()) return
        host.dispatchEvent(new CustomEvent('sui-radio-select', {
          detail: { value: scope.value() },
          bubbles: true,
          composed: true,
        }))
      }

      const onKeydown = (e: KeyboardEvent) => {
        const navKeys = ['ArrowDown', 'ArrowRight', 'ArrowUp', 'ArrowLeft']
        if (!navKeys.includes(e.key)) return
        e.preventDefault()
        const forward = e.key === 'ArrowDown' || e.key === 'ArrowRight'
        host.dispatchEvent(new CustomEvent('sui-radio-nav', {
          detail: { direction: forward ? 'next' : 'prev' },
          bubbles: true,
          composed: true,
        }))
      }

      input.addEventListener('click', onClick)
      input.addEventListener('keydown', onKeydown)
      return () => {
        input.removeEventListener('click', onClick)
        input.removeEventListener('keydown', onKeydown)
      }
    })

    // Expose focus helper for group keyboard navigation.
    ;(host as unknown as { focusInput(): void }).focusInput = () => {
      const input = host.shadowRoot?.querySelector('input') as HTMLInputElement | null
      input?.focus()
    }

    return () => stop()
  },
})
