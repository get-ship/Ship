import { controller, effect } from '@ship/sui-runtime'
import type { Scope } from './sui-kbd.sui'

export default controller<Scope>({
  mount(state, host) {
    if (!host.hasAttribute('role')) host.setAttribute('role', 'group')
    const stopSync = effect(() => {
      const k = (state.$signals.keys as unknown as () => string)()
      if (k) host.setAttribute('aria-label', `Keyboard shortcut: ${k}`)
      else host.removeAttribute('aria-label')
    })
    return () => stopSync()
  },
})
