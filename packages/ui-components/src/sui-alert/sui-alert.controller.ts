import { controller, effect } from '@ship/sui-runtime'
import type { Scope } from './sui-alert.sui'

export default controller<Scope>({
  mount(state, host) {
    const scope = state as unknown as Scope

    const stopDismiss = effect(() => {
      const btn = scope.closeBtn() as HTMLButtonElement | null
      if (!btn) return
      const onClick = () => {
        if (!scope.dismissible()) return
        host.dispatchEvent(new CustomEvent('sui-dismiss', { bubbles: true, composed: true }))
        host.remove()
      }
      btn.addEventListener('click', onClick)
      return () => btn.removeEventListener('click', onClick)
    })

    return () => {
      stopDismiss()
    }
  },
})
