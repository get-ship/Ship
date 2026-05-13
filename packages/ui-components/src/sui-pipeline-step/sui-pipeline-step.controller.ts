import { controller, effect } from '@ship/sui-runtime'
import type { Scope } from './sui-pipeline-step.sui'

export default controller<Scope>({
  mount(state, host) {
    const scope = state as unknown as Scope
    if (!host.hasAttribute('role')) host.setAttribute('role', 'listitem')

    const stopLabel = effect(() => {
      const name = scope.name()
      const type = scope.stepType()
      const agent = scope.agent()
      const s = scope.state()
      const summary = `${type} step ${name}${agent ? ` (${agent})` : ''}: ${s}`
      host.setAttribute('aria-label', summary)
    })

    return () => {
      stopLabel()
    }
  },
})
