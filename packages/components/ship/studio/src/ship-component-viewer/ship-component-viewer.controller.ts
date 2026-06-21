// <ship-component-viewer> — @ship/studio · the hub `/studio/components/:name` viewer.
//
// BACKEND-GAP controller (see the .sui for the data story): the
// component detail needs a `ship.ui.components.detail({ tag })` Connect
// op that does not exist today. The same gap closes the catalog index
// and the viewer — when the SDK ships the op, the controller's `entry`
// derived switches to `live(() => ship.ui.components.detail({ tag: tag() }))`
// and the rest of the surface (props, slots, source, snippet) reads
// the same shape.
//
// Until then the controller is a pure-derived presenter that
//   - resolves the category for the requested tag from a small static
//     map (the same map the shell's `categoryFor` exports, mirrored
//     inline here so the viewer doesn't import shell internals);
//   - reads the per-package list to drive the meta line;
//   - emits the documented empty-state for every tag (entry is null).
//
// The map is a stable public fact about the tag, not invented data —
// the gap is at the `entry` boundary, not inside the derived that
// consumes it.

import { controller, derived } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './ship-component-viewer.sui'

/** A single prop row, in the shape a future detail op would return. */
export interface ViewerProp {
  name: string
  type: string
  default?: string
  description?: string
}

/** A single slot row. */
export interface ViewerSlot {
  name: string
  description?: string
}

/** A single component-detail entry. Today this is always null; when
 *  the SDK ships, the controller's `entry` derived returns the wire
 *  shape cast to this interface. */
export interface ViewerEntry {
  tag: string
  pkg?: string
  description?: string
  props?: ViewerProp[]
  slots?: ViewerSlot[]
  /** Repo-relative source path (for the source link). */
  sourcePath?: string
}

/** State the template reads.
 *  - `'found'`   — `entry()` is non-null; the live branch renders.
 *  - `'missing'` — no entry; the empty-state renders. */
export type ViewerState = 'missing' | 'found'

/** Static tag → category map. Mirrors the shell's `CATEGORY_BY_TAG`
 *  (apps/shell/routes/studio/components/catalog.ts) — these are
 *  public facts about the tag, not invented data. */
const CATEGORY_BY_TAG: Record<string, string> = {
  // Actions
  'sui-button': 'Actions',
  'sui-toggle-group': 'Actions', 'sui-button-group': 'Actions', 'sui-icon-button': 'Actions',
  // Inputs
  'sui-input': 'Inputs', 'sui-textarea': 'Inputs', 'sui-select': 'Inputs',
  'sui-checkbox': 'Inputs', 'sui-radio-group': 'Inputs', 'sui-switch': 'Inputs',
  'sui-slider': 'Inputs', 'sui-file-upload': 'Inputs', 'sui-field': 'Inputs',
  'sui-form': 'Inputs', 'sui-combobox': 'Inputs', 'sui-color-picker': 'Inputs',
  'sui-date-picker': 'Inputs', 'sui-search': 'Inputs',
  // Layout
  'sui-card': 'Layout', 'sui-divider': 'Layout', 'sui-separator': 'Layout',
  'sui-tabs': 'Layout', 'sui-accordion': 'Layout', 'sui-collapsible': 'Layout',
  'sui-scroll-area': 'Layout', 'sui-grid': 'Layout', 'sui-stack': 'Layout',
  'sui-cluster': 'Layout',
  // Feedback
  'sui-alert': 'Feedback', 'sui-toast': 'Feedback',
  'sui-spinner': 'Feedback', 'sui-skeleton': 'Feedback', 'sui-empty-state': 'Feedback',
  'sui-error-state': 'Feedback',
  // Indicators
  'sui-badge': 'Indicators', 'sui-chip': 'Indicators', 'sui-status-dot': 'Indicators',
  'sui-stat': 'Indicators', 'sui-avatar': 'Indicators',
  'sui-icon': 'Indicators', 'sui-tag': 'Indicators',
  // Overlays
  'sui-dialog': 'Overlays', 'sui-alert-dialog': 'Overlays', 'sui-drawer': 'Overlays',
  'sui-popover': 'Overlays', 'sui-tooltip': 'Overlays', 'sui-hover-card': 'Overlays',
  'sui-dropdown-menu': 'Overlays', 'sui-context-menu': 'Overlays', 'sui-menu': 'Overlays',
  'sui-command-palette': 'Overlays', 'sui-sheet': 'Overlays',
  // Data
  'sui-table': 'Data', 'sui-tree-view': 'Data',
  'sui-json-view': 'Data', 'sui-diff': 'Data', 'sui-timeline': 'Data',
  'sui-history-row': 'Data', 'sui-list': 'Data', 'sui-data-grid': 'Data',
  'sui-code-block': 'Data', 'sui-markdown': 'Data',
  // Chrome (ship-* chrome pack + the chrome primitives it composes)
  'ship-nav-top': 'Chrome', 'ship-app-chrome': 'Chrome',
  'ship-rail': 'Chrome', 'ship-dock': 'Chrome', 'ship-dock-app': 'Chrome',
  'ship-command-bar': 'Chrome', 'ship-activity-bar': 'Chrome', 'ship-status-bar': 'Chrome',
  'ship-file-tree': 'Chrome', 'ship-tree-item': 'Chrome',
  'sui-project-switcher': 'Chrome', 'sui-account-menu': 'Chrome',
  // Domain composites
  'ship-home-greeting': 'Home', 'ship-spaces-hero': 'Home', 'ship-activity-runs': 'Home',
  'ship-component-catalog': 'Studio', 'ship-component-viewer': 'Studio',
  'ship-artifact-list': 'Artifacts', 'ship-artifact-filters': 'Artifacts', 'ship-artifact-preview': 'Artifacts',
  'ship-collection-table': 'Collections', 'ship-collection-picker': 'Collections',
  'ship-capability-bar': 'Authority', 'ship-approval-card': 'Authority',
  'ship-gate-log': 'Authority', 'ship-policy-matrix': 'Authority', 'ship-principal-list': 'Authority',
  'ship-code-editor': 'Code', 'ship-markdown-editor': 'Code', 'ship-terminal': 'Code',
  'ship-git-diff': 'Code', 'ship-git-diff-file': 'Code', 'ship-git-diff-hunk': 'Code', 'ship-git-diff-line': 'Code',
  'sui-nav': 'Navigation',
}

export function categoryFor(tag: string): string {
  return CATEGORY_BY_TAG[tag] ?? 'Other'
}

export default controller<ViewScope<Scope>>({
  scope: ({ tag }: ViewScope<Scope>, host?: Element) => {
    /** Single source of truth for "the catalog has a record for this
     *  tag". Today: always null. When the SDK ships the detail op,
     *  this becomes `derived<ViewerEntry | null>(() => ship.ui
     *  .components.detail({ tag: tag() }))`. The template's
     *  `state() === 'found'` branch flips on automatically. */
    const entry = derived<ViewerEntry | null>(() => null)

    const state = derived<ViewerState>(() => (entry() ? 'found' : 'missing'))

    const displayTag = derived<string>(() => {
      const t = (tag() ?? '').trim()
      return t || 'unnamed'
    })

    const category = derived<string>(() => categoryFor(displayTag()))

    /** "Props · slots" meta line. Stable public data (counts only,
     *  no invented entries) — the per-prop detail is the future op's
     *  job. */
    const metaLabel = derived<string>(() => {
      const e = entry()
      if (e) {
        const p = (e.props ?? []).length
        const s = (e.slots ?? []).length
        const parts: string[] = [`${p} prop${p === 1 ? '' : 's'}`]
        if (s > 0) parts.push(`${s} slot${s === 1 ? '' : 's'}`)
        return parts.join(' · ')
      }
      return 'BACKEND-GAP · no components op'
    })

    const propsList = derived<ViewerProp[]>(() => entry()?.props ?? [])
    const slotList = derived<ViewerSlot[]>(() => entry()?.slots ?? [])

    /** Empty-state copy. The "tag unknown" branch fires when the
     *  route's :name is empty; the BACKEND-GAP branch fires for
     *  every real tag today. */
    const emptyTitle = derived<string>(() => {
      if (!displayTag() || displayTag() === 'unnamed') return 'Pick a component'
      return 'Component details unavailable'
    })

    const emptyBody = derived<string>(() => {
      if (!displayTag() || displayTag() === 'unnamed') {
        return 'No tag was supplied in the route. Open the catalog and pick a component.'
      }
      return 'The hub cannot resolve component details right now — the Connect SDK has no ship.ui.components.detail op and the deck catalog endpoint was retired. The flag is recorded in .artifacts/hub-buildout-flags.md.'
    })

    /** Back-link click: emit the host-owned intent. The href
     *  still works for non-JS navigations / middle-click. */
    const onBack = (e: MouseEvent): void => {
      e.preventDefault()
      host?.dispatchEvent(
        new CustomEvent('ship-component-back', {
          bubbles: true,
          composed: true,
        }),
      )
    }

    return {
      entry,
      state,
      displayTag,
      category,
      metaLabel,
      propsList,
      slotList,
      emptyTitle,
      emptyBody,
      onBack,
    }
  },
})
