import { controller, effect } from '@ship/sui-runtime'
import type { Scope } from './sui-tab-panel.sui'

export default controller<Scope>({
  mount(state, host) {
    const scope = state as unknown as Scope

    if (!host.hasAttribute('role')) host.setAttribute('role', 'tabpanel')
    if (!host.hasAttribute('tabindex')) host.setAttribute('tabindex', '0')

    const stopIdSync = effect(() => {
      const name = scope.name()
      if (!name) return
      if (!host.id) host.id = `sui-tabpanel-${name}`
      host.setAttribute('aria-labelledby', `sui-tab-${name}`)
    })

    return () => {
      stopIdSync()
    }
  },
})
