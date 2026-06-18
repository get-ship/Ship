import { controller } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './ship-file-tree.sui'

// Titled navigator panel. Presentational except for two header intents: the
// host creates a new entry (ship-new) or collapses every row (ship-collapse-all).
export default controller({
  scope: (_props: ViewScope<Scope>, host) => {
    function emit(type: string): void {
      host?.dispatchEvent(new CustomEvent(type, { bubbles: true, composed: true }))
    }
    function newItem(): void { emit('ship-new') }
    function collapseAll(): void { emit('ship-collapse-all') }
    return { newItem, collapseAll }
  },
})
