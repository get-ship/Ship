import { controller, signal, derived, effect } from '@ship/ui-runtime'
import type { Signal } from '@ship/ui-runtime'
import type { Scope } from './sui-timestamp.sui'
import { absoluteIso, formatRelative } from './sui-timestamp'

const MINUTE = 60_000

export default controller<Scope>({
  scope: ({ value }) => {
    const tick = signal(0)
    const absStr = derived(() => {
      const v = (value as () => string)()
      return v ? absoluteIso(v) : ''
    })
    const relStr = derived(() => {
      tick()
      const v = (value as () => string)()
      return v ? formatRelative(v) : ''
    })
    return { tick, absStr, relStr }
  },

  mount(state, host) {
    const scope = state as unknown as Scope
    const tick = scope.tick as unknown as Signal<number>

    const stopTitle = effect(() => {
      const v = scope.value() as string
      if (v) host.title = scope.absStr() as string
    })

    const timer = setInterval(() => tick.set(tick() + 1), MINUTE)

    return () => {
      stopTitle()
      clearInterval(timer)
    }
  },
})
