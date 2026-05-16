import { controller, signal, derived } from '@ship/sui-runtime'
import type { Scope } from './template.sui'

export default controller<Scope>({
  scope: () => {
    const errorRate = signal(2.3)
    const alarming = derived(() => errorRate() > 5)

    return {
      errorRate, alarming,
      bumpRate: (n: number): void => errorRate.set(Math.max(0, errorRate() + n)),
      reset: (): void => errorRate.set(0),
    }
  },
})
