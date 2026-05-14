import { controller, signal, derived, effect } from '@ship/sui-runtime'
import type { Scope } from './sui-timestamp.sui'
import { absoluteIso, formatRelative } from './sui-timestamp'

const MINUTE = 60_000

export default controller<Scope>({
  scope: ({ value }) => {
    const tick = signal(0)
    const absStr = derived<string>(() => {
      const v = (value as () => string)()
      return v ? absoluteIso(v) : ''
    })
    const relStr = derived<string>(() => {
      tick()
      const v = (value as () => string)()
      return v ? formatRelative(v) : ''
    })
    return { tick, absStr, relStr }
  },

  mount(state, host) {
    const stopTitle = effect(() => {
      const v = state.value()
      if (v) host.title = state.absStr()
    })

    const timer = setInterval(() => state.tick.set(state.tick() + 1), MINUTE)

    return () => {
      stopTitle()
      clearInterval(timer)
    }
  },
})
