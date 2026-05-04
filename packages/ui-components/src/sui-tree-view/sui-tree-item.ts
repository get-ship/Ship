import { controller, effect } from '@ship/ui-runtime'
import type { Scope } from './sui-tree-item.sui'

export default controller<Scope>({
  mount(state, host) {
    const scope = state as unknown as Scope

    if (!host.hasAttribute('role')) host.setAttribute('role', 'treeitem')

    const syncAria = effect(() => {
      const row = scope.rowEl() as HTMLElement | null
      if (!row) return
      if (!scope.leaf()) {
        row.setAttribute('aria-expanded', String(scope.expanded()))
      } else {
        row.removeAttribute('aria-expanded')
      }
      row.setAttribute('aria-selected', String(scope.selected()))
    })

    const toggle = (e: Event) => {
      if (scope.leaf()) return
      e.stopPropagation()
      const next = !scope.expanded()
      scope.expanded.set(next)
      host.dispatchEvent(
        new CustomEvent('sui-toggle', {
          detail: { expanded: next },
          bubbles: true,
          composed: true,
        }),
      )
    }

    const select = () => {
      host.dispatchEvent(
        new CustomEvent('sui-select', {
          bubbles: true,
          composed: true,
          detail: {},
        }),
      )
    }

    const onRowClick = (e: MouseEvent) => {
      select()
      toggle(e)
    }

    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Enter' || e.key === ' ') {
        e.preventDefault()
        select()
      } else if (e.key === 'ArrowRight' && !scope.leaf() && !scope.expanded()) {
        e.preventDefault()
        toggle(e)
      } else if (e.key === 'ArrowLeft' && scope.expanded()) {
        e.preventDefault()
        toggle(e)
      }
    }

    const stopRow = effect(() => {
      const row = scope.rowEl() as HTMLElement | null
      if (!row) return
      row.addEventListener('click', onRowClick)
      row.addEventListener('keydown', onKey)
      return () => {
        row.removeEventListener('click', onRowClick)
        row.removeEventListener('keydown', onKey)
      }
    })

    return () => {
      syncAria()
      stopRow()
    }
  },
})
