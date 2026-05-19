import { controller, derived, effect, signal } from '@ship/sui-runtime'
import type { Signal } from '@ship/sui-runtime'
import type { Scope } from './sui-git-diff.sui'

type GitDiffFile = HTMLElement & { collapsed: boolean }

export default controller<Scope>({
  scope: (state, host) => {
    const s = state as unknown as {
      files: Signal<number>
      additions: Signal<number>
      deletions: Signal<number>
      empty: Signal<boolean>
    }

    const allCollapsed = signal<boolean>(false)

    const showEmpty = derived<boolean>(() => s.empty() || s.files() === 0)
    const filesLabel = derived<string>(() => {
      const n = s.files()
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
      const kids = host.querySelectorAll<GitDiffFile>('sui-git-diff-file')
      kids.forEach((f) => {
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
    const scope = state as unknown as {
      files: Signal<number>
      additions: Signal<number>
      deletions: Signal<number>
    }
    if (!host.hasAttribute('role')) host.setAttribute('role', 'region')

    const stopLabel = effect(() => {
      const f = scope.files()
      const a = scope.additions()
      const d = scope.deletions()
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
