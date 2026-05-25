import { controller, derived, effect } from '@ship/sui-runtime'
import type { Signal } from '@ship/sui-runtime'
import type { Scope } from './sui-pipeline-definition.sui'

export default controller<Scope>({
  scope: (state) => {
    const s = state as unknown as {
      steps: Signal<number>
    }
    const stepsLabel = derived<string>(() => {
      const n = s.steps()
      return `${n} step${n === 1 ? '' : 's'}`
    })
    return { stepsLabel }
  },

  mount(state, host) {
    const scope = state as unknown as {
      name: Signal<string>
      steps: Signal<number>
      description: Signal<string>
    }
    if (!host.hasAttribute('role')) host.setAttribute('role', 'article')

    const stopLabel = effect(() => {
      const name = scope.name()
      const steps = scope.steps()
      const desc = scope.description()
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
