/**
 * @ship/ui/web-components/observability — observability primitive bundle.
 *
 * Importing this module registers each custom element as a side effect.
 * The top-level `web-components/index.ts` re-exports from here.
 */

import './ship-log-line'
import './ship-log-stream'
import './ship-event-row'
import './ship-timestamp'
import './ship-json-view'
import './ship-diff'
import './ship-code'
import './ship-copy-button'
import './ship-markdown'
import './ship-table'
import './ship-timeline'
import './ship-tree-view'

export { ShipLogLine } from './ship-log-line'
export { ShipLogStream } from './ship-log-stream'
export { ShipEventRow } from './ship-event-row'
export { ShipTimestamp, formatRelative } from './ship-timestamp'
export { ShipJsonView } from './ship-json-view'
export { ShipDiff } from './ship-diff'
export { ShipCode } from './ship-code'
export { ShipCopyButton } from './ship-copy-button'
export { ShipMarkdown, renderMarkdown } from './ship-markdown'
export { ShipTable } from './ship-table'
export { ShipTimeline, ShipTimelineItem } from './ship-timeline'
export { ShipTreeView, ShipTreeNode } from './ship-tree-view'
