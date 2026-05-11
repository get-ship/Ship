import { controller, effect } from '@ship/sui-runtime'
import type { Scope } from './sui-label.sui'

export default controller<Scope>({
  mount(state, _host) {
    const scope = state as unknown as Scope

    const stopFor = effect(() => {
      const labelEl = scope.labelEl() as HTMLLabelElement | null
      if (!labelEl) return
      // bracket notation avoids reserved-word TS parsing issues
      labelEl.htmlFor = (scope as Record<string, () => string>)['for']?.() ?? ''
    })

    return () => {
      stopFor()
    }
  },
})
