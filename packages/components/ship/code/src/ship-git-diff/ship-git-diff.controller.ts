import { controller, derived, effect, signal } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './ship-git-diff.sui'

type GitDiffFile = HTMLElement & { collapsed: boolean }

export default controller({
  scope: (ctx: ViewScope<Scope>, host) => {
    const allCollapsed = signal<boolean>(false)

    const showEmpty = derived<boolean>(() => ctx.empty() || ctx.files() === 0)
    const filesLabel = derived<string>(() => {
      const n = ctx.files()
      return `${n} file${n === 1 ? '' : 's'} changed`
    })
    const toggleIcon = derived<string>(() =>
      allCollapsed() ? 'chevrons-up-down' : 'chevrons-down-up',
    )
    const toggleLabel = derived<string>(() =>
      allCollapsed() ? 'Expand all' : 'Collapse all',
    )
    const toggleTitle = derived<string>(() =>
      allCollapsed() ? 'Expand all files' : 'Collapse all files',
    )

    function toggleAll(): void {
      const next = !allCollapsed()
      allCollapsed.set(next)
      const kids = host?.querySelectorAll<GitDiffFile>('ship-git-diff-file')
      kids?.forEach((f) => {
        if (next) f.setAttribute('collapsed', '')
        else f.removeAttribute('collapsed')
      })
    }

    return {
      allCollapsed,
      showEmpty,
      filesLabel,
      toggleIcon,
      toggleLabel,
      toggleTitle,
      toggleAll,
    }
  },

  mount(state, host) {
    if (!host.hasAttribute('role')) host.setAttribute('role', 'region')

    const stopLabel = effect(() => {
      const f = state.files
      const a = state.additions
      const d = state.deletions
      host.setAttribute(
        'aria-label',
        `Diff: ${f} file${f === 1 ? '' : 's'} changed, ${a} additions, ${d} deletions`,
      )
    })

    return () => {
      stopLabel()
    }
  },
})
