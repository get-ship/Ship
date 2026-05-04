import { controller } from '@ship/ui-runtime'
import type { Scope } from './sui-tree-view.sui'

export default controller<Scope>({
  mount(_state, host) {
    if (!host.hasAttribute('role')) host.setAttribute('role', 'tree')
  },
})
