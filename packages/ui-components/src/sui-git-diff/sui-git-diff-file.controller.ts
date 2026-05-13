import { controller, derived, effect } from '@ship/sui-runtime'
import type { Signal } from '@ship/sui-runtime'
import type { Scope } from './sui-git-diff-file.sui'

export default controller<Scope>({
  scope: (state) => {
    const s = state as unknown as {
      collapsed: Signal<boolean>
    }
    const chevIcon = derived<string>(() =>
      s.collapsed() ? 'chevron-right' : 'chevron-down',
    )
    const expandedLabel = derived<string>(() => (s.collapsed() ? 'false' : 'true'))

    function toggle(): void {
      s.collapsed.set(!s.collapsed())
    }

    return { chevIcon, expandedLabel, toggle }
  },

  mount(state, host) {
    const scope = state as unknown as {
      path: Signal<string>
      additions: Signal<number>
      deletions: Signal<number>
    }
    if (!host.hasAttribute('role')) host.setAttribute('role', 'group')

    const stopLabel = effect(() => {
      const path = scope.path()
      const a = scope.additions()
      const d = scope.deletions()
      host.setAttribute('aria-label', `${path}: +${a} -${d}`)
    })

    return () => {
      stopLabel()
    }
  },
})
