// Shared catalog loader for ship-ui-studio.
// Fetches three sources: Lit CEM, SUI registry, daemon views.

import type { ComponentExample } from '@ship/sui-runtime'

export interface CatalogEntry {
  id: string
  tag: string
  kind: 'lit' | 'sui' | 'view'
  description?: string
  props?: PropEntry[]
  /** Co-located showcase scenarios, read from the component's CEM declaration. */
  examples?: ComponentExample[]
  /** Declared default/named slots, read from the component's CEM declaration. */
  slots?: SlotEntry[]
  /** Origin package — drives the top-level filter pills (Primitives / Charts /
   *  Pipelines). One of `@ship/primitives`, `@ship/charts`, `@ship/pipelines`. */
  pkg?: string
  /** Repo-relative authoring source, for the "Source" section link. */
  sourcePath?: string
  /** Semantic group within the index (Actions, Inputs, Layout, …). Derived
   *  from the tag via {@link categoryFor}; charts/pipelines packages map to
   *  their own group. */
  category: string
}

export interface SlotEntry {
  name?: string
  description?: string
}

export interface PropEntry {
  name: string
  type: string
  default?: string
  description?: string
  /// Closed set of allowed string values, when the prop is a union. Used
  /// by the studio prop panel to render a `<select>` instead of a free
  /// text input.
  values?: string[]
}

// Semantic index groups. Mirrors the components design index: a flat tag →
// group map for the primitives package, with charts/pipelines packages
// folded into their own group. A tag with no explicit mapping lands in
// "Other" so a newly-added primitive is still visible (never dropped).
const CATEGORY_BY_TAG: Record<string, string> = {
  // Actions
  'sui-button': 'Actions', 'sui-toolbar': 'Actions',
  'sui-toggle-group': 'Actions', 'sui-button-group': 'Actions', 'sui-icon-button': 'Actions',
  // Inputs
  'sui-input': 'Inputs', 'sui-textarea': 'Inputs', 'sui-select': 'Inputs',
  'sui-checkbox': 'Inputs', 'sui-radio-group': 'Inputs', 'sui-switch': 'Inputs',
  'sui-slider': 'Inputs', 'sui-file-upload': 'Inputs', 'sui-field': 'Inputs',
  'sui-form': 'Inputs', 'sui-combobox': 'Inputs', 'sui-color-picker': 'Inputs',
  'sui-date-picker': 'Inputs', 'sui-tag-input': 'Inputs', 'sui-search': 'Inputs',
  // Layout
  'sui-card': 'Layout', 'sui-divider': 'Layout', 'sui-separator': 'Layout',
  'sui-tabs': 'Layout', 'sui-accordion': 'Layout', 'sui-collapsible': 'Layout',
  'sui-scroll-area': 'Layout', 'sui-grid': 'Layout', 'sui-stack': 'Layout',
  'sui-resizable': 'Layout', 'sui-aspect-ratio': 'Layout', 'sui-split': 'Layout',
  // Feedback
  'sui-alert': 'Feedback', 'sui-toast': 'Feedback', 'sui-progress': 'Feedback',
  'sui-spinner': 'Feedback', 'sui-skeleton': 'Feedback', 'sui-empty-state': 'Feedback',
  'sui-error-state': 'Feedback', 'sui-banner': 'Feedback', 'sui-callout': 'Feedback',
  // Indicators
  'sui-badge': 'Indicators', 'sui-chip': 'Indicators', 'sui-status-dot': 'Indicators',
  'sui-stat': 'Indicators', 'sui-avatar': 'Indicators',
  'sui-icon': 'Indicators', 'sui-kbd': 'Indicators', 'sui-tag': 'Indicators',
  'sui-meter': 'Indicators',
  // Overlays
  'sui-dialog': 'Overlays', 'sui-alert-dialog': 'Overlays', 'sui-drawer': 'Overlays',
  'sui-popover': 'Overlays', 'sui-tooltip': 'Overlays', 'sui-hover-card': 'Overlays',
  'sui-dropdown-menu': 'Overlays', 'sui-context-menu': 'Overlays', 'sui-menu': 'Overlays',
  'sui-command-palette': 'Overlays', 'sui-sheet': 'Overlays',
  // Data
  'sui-table': 'Data', 'sui-list-tile': 'Data', 'sui-tree-view': 'Data',
  'sui-json-view': 'Data', 'sui-diff': 'Data', 'sui-timeline': 'Data',
  'sui-history-row': 'Data', 'sui-list': 'Data', 'sui-data-grid': 'Data',
  'sui-code-editor': 'Data', 'sui-markdown-editor': 'Data', 'sui-code-block': 'Data',
  'sui-breadcrumb': 'Navigation', 'sui-pagination': 'Navigation',
  'sui-nav': 'Navigation', 'sui-stepper': 'Navigation',
}

// Order the index sections render in. Package-derived groups (Charts,
// Pipelines) sort after the semantic primitive groups; "Other" trails.
const CATEGORY_ORDER = [
  'Actions', 'Inputs', 'Layout', 'Feedback', 'Indicators',
  'Overlays', 'Data', 'Navigation', 'Charts', 'Pipelines', 'Other',
]

const PKG_CATEGORY: Record<string, string> = {
  '@ship/charts': 'Charts',
  '@ship/pipelines': 'Pipelines',
}

export function categoryFor(tag: string, pkg: string | undefined): string {
  const byPkg = pkg ? PKG_CATEGORY[pkg] : undefined
  if (byPkg) return byPkg
  return CATEGORY_BY_TAG[tag] ?? 'Other'
}

export function categoryRank(category: string): number {
  const i = CATEGORY_ORDER.indexOf(category)
  return i === -1 ? CATEGORY_ORDER.length : i
}

// The SUI catalog (`@ship/sui-catalog`, served at /api/sui/catalog/published)
// is the single source of truth for installed-pack components: tag, props,
// examples, and routing. There is no Lit path and no per-tag hide list — the
// catalog already excludes shell-local chrome and view-local components.
async function loadSuiComponents(): Promise<CatalogEntry[]> {
  const byTag = await loadSuiCatalog()
  return [...byTag.keys()]
    .sort()
    .map(tag => {
      const e = byTag.get(tag)!
      return {
        id: tag,
        tag,
        kind: 'sui' as const,
        props: e.props,
        category: categoryFor(tag, e.pkg),
        ...(e.description ? { description: e.description } : {}),
        ...(e.examples?.length ? { examples: e.examples } : {}),
        ...(e.slots?.length ? { slots: e.slots } : {}),
        ...(e.pkg ? { pkg: e.pkg } : {}),
        ...(e.sourcePath ? { sourcePath: e.sourcePath } : {}),
      }
    })
}

// One wire entry from the published catalog. Carries the standard CEM fields
// the studio needs plus the co-located `examples` (fun fixtures).
interface CatalogEntryWire {
  tagName: string
  description?: string
  attributes?: Array<{
    name: string
    type?: { text?: string }
    default?: string
    description?: string
    values?: unknown
  }>
  examples?: ComponentExample[]
  slots?: SlotEntry[]
  package?: string
  sourcePath?: string
}
interface SuiCatalogEntry {
  props: PropEntry[]
  examples?: ComponentExample[]
  description?: string
  slots?: SlotEntry[]
  pkg?: string
  sourcePath?: string
}

async function loadSuiCatalog(): Promise<Map<string, SuiCatalogEntry>> {
  const out = new Map<string, SuiCatalogEntry>()
  try {
    const res = await fetch('/api/sui/catalog/published', { credentials: 'same-origin' })
    if (!res.ok) return out
    const doc = await res.json() as { entries?: CatalogEntryWire[] }
    for (const e of doc.entries ?? []) {
      if (!e.tagName) continue
      out.set(e.tagName, {
        props: (e.attributes ?? []).map((a): PropEntry => ({
          name: a.name,
          type: a.type?.text ?? 'string',
          // exactOptionalPropertyTypes: omit optional keys rather than
          // assigning `undefined`.
          ...(a.default === undefined ? {} : { default: a.default }),
          ...(a.description === undefined ? {} : { description: a.description }),
          ...(Array.isArray(a.values) ? { values: a.values as string[] } : {}),
        })),
        ...(e.examples === undefined ? {} : { examples: e.examples }),
        ...(e.description === undefined ? {} : { description: e.description }),
        ...(Array.isArray(e.slots) ? { slots: e.slots } : {}),
        ...(e.package === undefined ? {} : { pkg: e.package }),
        ...(e.sourcePath === undefined ? {} : { sourcePath: e.sourcePath }),
      })
    }
    return out
  } catch {
    return out
  }
}

// The components explorer shows installed-pack components — sourced entirely
// from the SUI catalog. Views are not components; they live in the view-picker.
export async function loadCatalog(): Promise<CatalogEntry[]> {
  return loadSuiComponents()
}
