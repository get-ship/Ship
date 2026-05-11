import { controller, effect } from '@ship/sui-runtime'
import type { Scope } from './sui-kbd.sui'

export default controller<Scope>({
  mount(state, host) {
    const scope = state as unknown as Scope
    if (!host.hasAttribute('role')) host.setAttribute('role', 'group')
    const stopSync = effect(() => {
      const k = (scope.keys as unknown as () => string)()
      if (k) host.setAttribute('aria-label', `Keyboard shortcut: ${k}`)
      else host.removeAttribute('aria-label')
    })
    return () => stopSync()
  },
})
