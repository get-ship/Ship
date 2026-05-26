import { controller, effect } from '@ship/sui-runtime'
import type { Scope } from './sui-pipeline-step.sui'

export default controller<Scope>({
  mount(state, host) {
    if (!host.hasAttribute('role')) host.setAttribute('role', 'listitem')

    const stopLabel = effect(() => {
      const name = state.name
      const type = state.stepType
      const agent = state.agent
      const s = state.state
      const summary = `${type} step ${name}${agent ? ` (${agent})` : ''}: ${s}`
      host.setAttribute('aria-label', summary)
    })

    return () => {
      stopLabel()
    }
  },
})
