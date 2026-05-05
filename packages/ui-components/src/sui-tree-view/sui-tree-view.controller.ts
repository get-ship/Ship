import { controller, signal } from '@ship/ui-runtime'
import type { Scope } from './sui-tree-view.sui'
import type { Signal } from '@ship/ui-runtime'

export interface TreeNode {
  id: string
  label: string
  children?: TreeNode[]
}

export default controller<Scope>({
  scope: () => {
    const nodes = signal<TreeNode[]>([])
    return { nodes }
  },

  mount(state, host) {
    const scope = state as unknown as Scope
    const nodesSignal = scope.nodes as unknown as Signal<TreeNode[]>

    const initNodes = (host as any).nodes
    if (initNodes != null) nodesSignal.set(initNodes)

    Object.defineProperty(host, 'nodes', {
      configurable: true,
      get: () => nodesSignal(),
      set: (v: TreeNode[]) => nodesSignal.set(v),
    })

    // Slot-based usage: set role=tree on host for accessibility.
    if (!host.hasAttribute('role')) host.setAttribute('role', 'tree')
  },
})
