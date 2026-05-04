import { controller, effect } from '@ship/ui-runtime'
import type { Scope } from './sui-checkbox.sui'

export default controller<Scope>({
  mount(state, host) {
    const scope = state as unknown as Scope

    // Keep native indeterminate property in sync — it's not reflectable via attrs.
    const stopIndet = effect(() => {
      const el = scope.inputEl()
      if (!el) return
      el.indeterminate = scope.indeterminate()
    })

    // Dispatch sui-change and clear indeterminate on user interaction.
    // bind:checked already syncs the signal before this handler fires.
    const stopChange = effect(() => {
      const el = scope.inputEl()
      if (!el) return
      const onNativeChange = () => {
        scope.indeterminate.set(false)
        host.dispatchEvent(new CustomEvent('sui-change', {
          detail: { checked: el.checked, value: scope.value() },
          bubbles: true,
          composed: true,
        }))
      }
      el.addEventListener('change', onNativeChange)
      return () => el.removeEventListener('change', onNativeChange)
    })

    return () => {
      stopIndet()
      stopChange()
    }
  },
})
