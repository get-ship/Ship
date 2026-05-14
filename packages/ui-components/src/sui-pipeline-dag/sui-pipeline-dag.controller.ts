import { controller } from '@ship/sui-runtime'
import type { Scope } from './sui-pipeline-dag.sui'

export default controller<Scope>({
  mount(_state, host) {
    if (!host.hasAttribute('role')) host.setAttribute('role', 'list')
    if (!host.hasAttribute('aria-label')) {
      host.setAttribute('aria-label', 'pipeline steps')
    }
  },
})
