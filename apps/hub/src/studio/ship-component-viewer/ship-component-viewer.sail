<controller>
// <ship-component-viewer> — @ship/studio · the hub `/studio/components/:name` viewer.
//
// BACKEND-GAP controller (see the .sail for the data story): the
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

import { controller, derived } from '@ship/sail-runtime'
import type { ViewScope } from '@ship/sail-runtime'
import type { Scope } from './ship-component-viewer.sail'

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
  'sail-toggle-group': 'Actions', 'sui-button-group': 'Actions', 'sui-icon-button': 'Actions',
  // Inputs
  'sail-input': 'Inputs', 'sui-textarea': 'Inputs', 'sail-select': 'Inputs',
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
  // Chrome (runtime-coupled @ship/chrome shell + the runtime-free @ship/shared
  // wrappers it pairs with; <sui-account-menu>/<sui-project-switcher> are now
  // the SDK-free @ship/sail-primitives versions, wired host-side by <ship-nav-top>)
  'ship-nav-top': 'Chrome', 'ship-app-chrome': 'Chrome',
  'ship-rail': 'Chrome', 'ship-dock': 'Chrome', 'ship-dock-app': 'Chrome',
  'ship-command-bar': 'Chrome', 'ship-activity-bar': 'Chrome', 'ship-status-bar': 'Chrome',
  'ship-file-tree': 'Chrome',
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
</controller>

<!--
  <ship-component-viewer> — @ship/studio · the hub `/studio/components/:name` viewer.

  The single-component viewer. Mirrors the home pack's
  `<ship-spaces-hero>` / `<ship-activity-runs>` pattern: a pure-derived
  presenter that reads a small set of inputs and renders. The data story
  is the same BACKEND-GAP the catalog index faces — no `ship.ui.components
  .detail({ tag })` op, the runtime catalog is gone, the file on disk is
  stale. The component renders a stable viewer surface for ANY tag the
  page routes to (the route's `name` param drives the viewer's `tag` prop),
  and the template shows the "tag not found in catalog" empty-state for
  every tag today. When the SDK ships the detail op, the controller's
  `entry` derived switches to `live(() => ship.ui.components.detail({tag}))`
  and the rest of the surface (props, slots, source, snippet) flips on
  without a template change.

  Until then: the viewer shows the requested tag, its category (the
  semantically-correct category the controller derives from the tag +
  the per-package list — these are stable public facts, not invented
  data), and the documented empty-state. No fabricated prop / slot /
  example rows.

  Prop / event
  ─────────────────────────────────────────────────────────────────────────
  tag    (string, '')   component tag from the route (the `:name` param)
  ─────────────────────────────────────────────────────────────────────────

  Emits:
    ship-component-back   {}   click on the back link
    ship-component-source { detail: { tag: string } }   click on the source link
-->
<!-- <prop name="tag" type="string" default="" /> -->

<!-- style block removed: use sx / external CSS -->

<a href="/studio/components" data-test="viewer-back" on:click={onBack}>
  <icon name="chevron-left" size="xs"></icon>
  Components
</a>

<header data-test="viewer-head">
  <span aria-hidden="true">&lt;/&gt;</span>
  <div>
    <h1 data-test="viewer-title">{displayTag()}</h1>
    <div data-test="viewer-meta">
      <code>{category()}</code>
      <span aria-hidden="true">·</span>
      <span>{metaLabel()}</span>
    </div>
    <div aria-hidden="true"></div>
  </div>
</header>

<if cond={state() === 'found'}>
  <!--
    The future-live branch. Kept structurally complete; the props / slots
    / source / snippet sections are hidden today because the BACKEND-GAP
    gate (`state() === 'found'`) is false. When the SDK ships the detail
    op, the controller's `entry` derived becomes non-null and the
    sections render without a template change.
  -->
  <if cond={propsList().length > 0}>
    <section data-test="viewer-props">
      <h2>Props · {propsList().length}</h2>
      <ul>
        <for as="p" of={propsList()} key={p.name}>
          <li><code>{p.name}</code>: <code>{p.type}</code></li>
        </for>
      </ul>
    </section>
  </if>
  <if cond={slotList().length > 0}>
    <section data-test="viewer-slots">
      <h2>Slots · {slotList().length}</h2>
      <ul>
        <for as="s" of={slotList()} key={s.name}>
          <li><code>slot="{s.name}"</code><if cond={s.desc}> — {s.desc}</if></li>
        </for>
      </ul>
    </section>
  </if>
<else>
  <div>
    <empty-state variant="default" size="md" tone="muted" data-test="viewer-empty">
      <icon slot="icon" name="package-search"></icon>
      <span slot="title">{emptyTitle()}</span>
      {emptyBody()}
    </empty-state>
  </div>
</else></if>
