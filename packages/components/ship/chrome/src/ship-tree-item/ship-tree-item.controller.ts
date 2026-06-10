import { controller } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './ship-tree-item.sui'

// One file-tree row. Folders toggle (ship-toggle {open}); files select
// (ship-select). The host owns expanding/collapsing children and selection.
export default controller({
  scope: ({ kind, open }: ViewScope<Scope>, host) => {
    function activate(): void {
      if (kind() === 'folder') {
        const next = !open()
        open.set(next)
        host?.dispatchEvent(
          new CustomEvent('ship-toggle', {
            detail: { open: next },
            bubbles: true,
            composed: true,
          }),
        )
      } else {
        host?.dispatchEvent(
          new CustomEvent('ship-select', { bubbles: true, composed: true }),
        )
      }
    }
    return { activate }
  },
})
