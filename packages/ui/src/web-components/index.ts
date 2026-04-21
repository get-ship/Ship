/**
 * @ship/ui/elements — Lit-authored web components.
 *
 * Importing this module registers every component via its `@customElement`
 * decorator as a side-effect. Callers that want explicit control can
 * instead import per-component files.
 */

import './ship-accordion'
import './ship-alert'
import './ship-alert-dialog'
import './ship-annotation-layer'
import './ship-annotation-list'
import './ship-annotation-marker'
import './ship-badge'
import './ship-button'
import './ship-card'
import './ship-checkbox'
import './ship-collapsible'
import './ship-context-menu'
import './ship-dialog'
import './ship-dropdown-menu'
import './ship-empty-state'
import './ship-field'
import './ship-file-upload'
import './ship-hover-card'
import './ship-input'
import './ship-label'
import './ship-list-item'
import './ship-mark'
import './ship-navigation-menu'
import './ship-pagination'
import './ship-popover'
import './ship-progress'
import './ship-radio-group'
import './ship-scroll-area'
import './ship-select'
import './ship-separator'
import './ship-slider'
import './ship-spinner'
import './ship-stat'
import './ship-status-dot'
import './ship-stepper'
import './ship-switch'
import './ship-tabs'
import './ship-textarea'
import './ship-theme-toggle'
import './ship-toggle-group'
import './ship-toolbar'
import './ship-tooltip'

// Domain
import './domain/ship-pipeline-step'
import './domain/ship-pipeline-run'
import './domain/ship-pipeline-definition'
import './domain/ship-pipeline-dag'
import './domain/ship-history-row'
import './domain/ship-spec-row'
import './domain/ship-git-diff'
import './domain/ship-auth-dialog'

// Observability, charts, utility (each subdir index registers its own)
export * from './observability'
export * from './charts'
export * from './utility'

export { ShipAccordion, ShipAccordionItem } from './ship-accordion'
export { ShipAlert } from './ship-alert'
export { ShipAlertDialog } from './ship-alert-dialog'
export { ShipBadge } from './ship-badge'
export { ShipButton } from './ship-button'
export { ShipCard } from './ship-card'
export { ShipCheckbox } from './ship-checkbox'
export { ShipCollapsible } from './ship-collapsible'
export { ShipContextMenu } from './ship-context-menu'
export { ShipDialog } from './ship-dialog'
export { ShipDropdownMenu } from './ship-dropdown-menu'
export { ShipEmptyState } from './ship-empty-state'
export { ShipField } from './ship-field'
export { ShipFileUpload } from './ship-file-upload'
export { ShipHoverCard } from './ship-hover-card'
export { ShipInput } from './ship-input'
export { ShipLabel } from './ship-label'
export { ShipListItem } from './ship-list-item'
export { ShipMark } from './ship-mark'
export { ShipNavigationMenu, ShipNavigationItem } from './ship-navigation-menu'
export { ShipPagination } from './ship-pagination'
export { ShipPopover } from './ship-popover'
export { ShipProgress } from './ship-progress'
export { ShipRadioGroup, ShipRadio } from './ship-radio-group'
export { ShipScrollArea } from './ship-scroll-area'
export { ShipSelect } from './ship-select'
export { ShipSeparator } from './ship-separator'
export { ShipSlider } from './ship-slider'
export { ShipSpinner } from './ship-spinner'
export { ShipStat } from './ship-stat'
export { ShipStatusDot } from './ship-status-dot'
export { ShipStepper } from './ship-stepper'
export { ShipSwitch } from './ship-switch'
export { ShipTabs, ShipTab, ShipTabPanel } from './ship-tabs'
export { ShipTextarea } from './ship-textarea'
export { ShipThemeToggle } from './ship-theme-toggle'
export { ShipToggle, ShipToggleGroup } from './ship-toggle-group'
export { ShipToolbar } from './ship-toolbar'
export { ShipTooltip } from './ship-tooltip'

export { ShipPipelineStep } from './domain/ship-pipeline-step'
export { ShipPipelineRun } from './domain/ship-pipeline-run'
export { ShipPipelineDefinition } from './domain/ship-pipeline-definition'
export { ShipPipelineDag } from './domain/ship-pipeline-dag'
export { ShipHistoryRow } from './domain/ship-history-row'
export { ShipSpecRow } from './domain/ship-spec-row'
export { ShipGitDiff, ShipGitDiffFile, ShipGitDiffHunk, ShipGitDiffLine } from './domain/ship-git-diff'
export { ShipStepSegments } from './utility/ship-step-segments'

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
