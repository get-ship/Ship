import { controller, derived, effect } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-pipeline-definition.sui'

export default controller({
  scope: (ctx: ViewScope<Scope>) => {
    const stepsLabel = derived<string>(() => {
      const n = ctx.steps()
      return `${n} step${n === 1 ? '' : 's'}`
    })
    return { stepsLabel }
  },

  mount(state, host) {
    if (!host.hasAttribute('role')) host.setAttribute('role', 'article')

    const stopLabel = effect(() => {
      const name = state.name
      const steps = state.steps
      const desc = state.description
      const suffix = steps === 1 ? '' : 's'
      const tail = desc ? `: ${desc}` : ''
      host.setAttribute(
        'aria-label',
        `Pipeline ${name}, ${steps} step${suffix}${tail}`,
      )
    })

    return () => {
      stopLabel()
    }
  },
})
