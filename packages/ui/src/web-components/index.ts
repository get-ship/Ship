/**
 * @ship/ui/elements — Lit-authored web components.
 *
 * Importing this module registers every component via its `@customElement`
 * decorator as a side-effect. Callers that want explicit control can
 * instead import per-component files.
 */

import './ship-alert'
import './ship-alert-dialog'
import './ship-badge'
import './ship-button'
import './ship-card'
import './ship-checkbox'
import './ship-dialog'
import './ship-dropdown-menu'
import './ship-empty-state'
import './ship-field'
import './ship-input'
import './ship-label'
import './ship-list-item'
import './ship-popover'
import './ship-progress'
import './ship-select'
import './ship-separator'
import './ship-spinner'
import './ship-stat'
import './ship-status-dot'
import './ship-switch'
import './ship-tabs'
import './ship-textarea'
import './ship-theme-toggle'
import './ship-toolbar'
import './ship-tooltip'

// Domain
import './domain/ship-pipeline-step'
import './domain/ship-pipeline-run'
import './domain/ship-pipeline-definition'
import './domain/ship-pipeline-dag'
import './domain/ship-history-row'
import './domain/ship-spec-row'

export { ShipAlert } from './ship-alert'
export { ShipAlertDialog } from './ship-alert-dialog'
export { ShipBadge } from './ship-badge'
export { ShipButton } from './ship-button'
export { ShipCard } from './ship-card'
export { ShipCheckbox } from './ship-checkbox'
export { ShipDialog } from './ship-dialog'
export { ShipDropdownMenu } from './ship-dropdown-menu'
export { ShipEmptyState } from './ship-empty-state'
export { ShipField } from './ship-field'
export { ShipInput } from './ship-input'
export { ShipLabel } from './ship-label'
export { ShipListItem } from './ship-list-item'
export { ShipPopover } from './ship-popover'
export { ShipProgress } from './ship-progress'
export { ShipSelect } from './ship-select'
export { ShipSeparator } from './ship-separator'
export { ShipSpinner } from './ship-spinner'
export { ShipStat } from './ship-stat'
export { ShipStatusDot } from './ship-status-dot'
export { ShipSwitch } from './ship-switch'
export { ShipTabs, ShipTab, ShipTabPanel } from './ship-tabs'
export { ShipTextarea } from './ship-textarea'
export { ShipThemeToggle } from './ship-theme-toggle'
export { ShipToolbar } from './ship-toolbar'
export { ShipTooltip } from './ship-tooltip'

export { ShipPipelineStep } from './domain/ship-pipeline-step'
export { ShipPipelineRun } from './domain/ship-pipeline-run'
export { ShipPipelineDefinition } from './domain/ship-pipeline-definition'
export { ShipPipelineDag } from './domain/ship-pipeline-dag'
export { ShipHistoryRow } from './domain/ship-history-row'
export { ShipSpecRow } from './domain/ship-spec-row'

export type { Tone, Variant, Size } from './util'

/**
 * Legacy entry-point retained for call-site compatibility with Phase-0
 * imports: registers every component. The `@customElement` decorators
 * already auto-register at module import time, so this is a no-op after
 * side-effect imports have run. Kept as a public API shim.
 */
export function registerShipComponents(): void {
  /* Components self-register via @customElement. */
}
