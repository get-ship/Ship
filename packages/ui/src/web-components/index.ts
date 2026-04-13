import { registerBadge } from './badge'
import { registerStepCard } from './step-card'
import { registerFileRow } from './file-row'
import { registerEmptyState } from './empty-state'
import { registerTabs } from './tabs'

export { ShipBadge } from './badge'
export { ShipStepCard } from './step-card'
export { ShipFileRow } from './file-row'
export { ShipEmptyState } from './empty-state'
export { ShipTabs, ShipTab } from './tabs'
export {
  registerBadge,
  registerStepCard,
  registerFileRow,
  registerEmptyState,
  registerTabs,
}

export function registerShipComponents(): void {
  registerBadge()
  registerStepCard()
  registerFileRow()
  registerEmptyState()
  registerTabs()
}

// Auto-register when loaded in a browser context (side-effect import).
if (typeof window !== 'undefined' && typeof customElements !== 'undefined') {
  registerShipComponents()
}
