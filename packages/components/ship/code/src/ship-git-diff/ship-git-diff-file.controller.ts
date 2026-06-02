import { controller, derived, effect } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './ship-git-diff-file.sui'

export default controller({
  scope: (ctx: ViewScope<Scope>) => {
    const chevIcon = derived<string>(() =>
      ctx.collapsed() ? 'chevron-right' : 'chevron-down',
    )
    const expandedLabel = derived<string>(() => (ctx.collapsed() ? 'false' : 'true'))

    function toggle(): void {
      ctx.collapsed.set(!ctx.collapsed())
    }

    return { chevIcon, expandedLabel, toggle }
  },

  mount(state, host) {
    if (!host.hasAttribute('role')) host.setAttribute('role', 'group')

    const stopLabel = effect(() => {
      const path = state.path
      const a = state.additions
      const d = state.deletions
      host.setAttribute('aria-label', `${path}: +${a} -${d}`)
    })

    return () => {
      stopLabel()
    }
  },
})
