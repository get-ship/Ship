import { controller, signal, derived, effect, ref } from '@ship/ui-runtime'
import type { Scope } from './sui-tree-item.sui'
import type { Signal } from '@ship/ui-runtime'
import type { TreeNode } from './sui-tree-view.controller'

export default controller<Scope>({
  scope: ({ label }) => {
    const node = signal<TreeNode | null>(null)
    const nodeLabel = derived(() => {
      const n = node()
      return n ? n.label : (label as unknown as Signal<string>)()
    })
    const nodeChildren = derived(() => {
      const n = node()
      return n?.children ?? []
    })
    const rowEl = ref<HTMLDivElement>(null)
    return { node, nodeLabel, nodeChildren, rowEl }
  },

  mount(state, host) {
    const scope = state as unknown as Scope

    if (!host.hasAttribute('role')) host.setAttribute('role', 'treeitem')

    // Wire node property so prop:node={n} from parent updates the signal.
    const nodeSignal = scope.node as unknown as Signal<TreeNode | null>
    const initNode = (host as any).node
    if (initNode != null) nodeSignal.set(initNode)
    Object.defineProperty(host, 'node', {
      configurable: true,
      get: () => nodeSignal(),
      set: (v: TreeNode | null) => nodeSignal.set(v),
    })

    // Sync leaf attribute from node data (node with no children → leaf).
    const stopLeafSync = effect(() => {
      const n = nodeSignal()
      if (n !== null) {
        const isLeaf = !n.children || n.children.length === 0
        if (isLeaf) host.setAttribute('leaf', '')
        else host.removeAttribute('leaf')
      }
    })

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
      stopLeafSync()
      syncAria()
      stopRow()
    }
  },
})
