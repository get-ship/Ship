import { controller, signal, derived } from '@ship/sui-runtime'
import type { Scope } from './template.sui'

export default controller<Scope>({
  scope: () => {
    const count = signal(0)
    const pageTitle = derived(() => `count: ${count()} — SUI tutorial`)

    return {
      count, pageTitle,
      bumpUp: (): void => count.set(count() + 1),
      bumpDown: (): void => count.set(count() - 1),
      reset: (): void => count.set(0),
    }
  },
})
