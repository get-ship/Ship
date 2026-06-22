// Component catalog loader for the studio explorer.
//
// Source: the build-time `components.gen.ts` (emitted by `deck build` —
// `deck-bundler` aggregates the standard custom-elements-manifest of every
// component pack the app depends on, from the sui compiler's per-component
// CEM). No runtime endpoint — the old `/_deck/catalog/workspace` deck route was
// purged (deck-purge-root). We import the generated catalog and project the
// standard CEM `modules[].declarations[]` into the studio's flat CatalogEntry.
import type { ComponentExample } from '@ship/sui-runtime'
// eslint-disable-next-line @typescript-eslint/no-unused-vars -- generated at build
import { COMPONENTS_CATALOG } from '../../../components.gen'

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
   *  Pipelines). One of `@ship/sui-primitives`, `@ship/sui-charts`, `@ship/pipelines`. */
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
  'ship-code-editor': 'Data', 'ship-markdown-editor': 'Data', 'sui-code-block': 'Data',
  'ship-terminal': 'Data',
  'ship-git-diff': 'Data', 'ship-git-diff-file': 'Data', 'ship-git-diff-hunk': 'Data',
  'ship-git-diff-line': 'Data',
  'ship-viewer': 'Data', 'sui-media-image': 'Data', 'sui-media-csv': 'Data',
  'sui-media-av': 'Data',
  'sui-nav': 'Navigation',
}

// Order the index sections render in. Package-derived groups (Charts,
// Pipelines) sort after the semantic primitive groups; "Other" trails.
const CATEGORY_ORDER = [
  'Actions', 'Inputs', 'Layout', 'Feedback', 'Indicators',
  'Overlays', 'Data', 'Navigation', 'Charts', 'Pipelines', 'Other',
]

const PKG_CATEGORY: Record<string, string> = {
  '@ship/sui-charts': 'Charts',
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

// The SUI catalog (served at /_deck/catalog/workspace) is the single source
// of truth for installed-pack components: tag, props, examples, and routing.
// There is no Lit path and no per-tag hide list — the catalog already covers
// shell-local chrome and view-local components via the sui-out walk.
async function loadSuiComponents(): Promise<CatalogEntry[]> {
  const byTag = loadSuiCatalog()
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

// A standard CEM module + the class declaration we project. `package` is the
// deck extension (`components.gen.ts` annotates each module with its source
// pack); `examples` rides on the declaration when the compiler emits fixtures.
interface CemDeclaration {
  kind?: string
  tagName?: string
  customElement?: boolean
  description?: string
  attributes?: Array<{
    name: string
    type?: { text?: string }
    default?: string
    description?: string
    values?: unknown
  }>
  slots?: SlotEntry[]
  examples?: ComponentExample[]
}
interface CemModule {
  path?: string
  package?: string
  declarations?: CemDeclaration[]
}
interface SuiCatalogEntry {
  props: PropEntry[]
  examples?: ComponentExample[]
  description?: string
  slots?: SlotEntry[]
  pkg?: string
  sourcePath?: string
}

// Project the generated standard CEM into the studio's tag→entry map. Pure +
// synchronous — the catalog is bundled, not fetched. Each custom-element class
// declaration becomes one entry; `module.package`/`module.path` carry origin.
function loadSuiCatalog(): Map<string, SuiCatalogEntry> {
  const out = new Map<string, SuiCatalogEntry>()
  const modules = (COMPONENTS_CATALOG as { modules?: CemModule[] }).modules ?? []
  for (const m of modules) {
    for (const d of m.declarations ?? []) {
      if (d.kind !== 'class' || !d.customElement || !d.tagName) continue
      out.set(d.tagName, {
        props: (d.attributes ?? []).map((a): PropEntry => ({
          name: a.name,
          type: a.type?.text ?? 'string',
          // exactOptionalPropertyTypes: omit optional keys rather than
          // assigning `undefined`.
          ...(a.default === undefined ? {} : { default: a.default }),
          ...(a.description === undefined ? {} : { description: a.description }),
          ...(Array.isArray(a.values) ? { values: a.values as string[] } : {}),
        })),
        ...(Array.isArray(d.examples) ? { examples: d.examples } : {}),
        ...(d.description === undefined ? {} : { description: d.description }),
        ...(Array.isArray(d.slots) ? { slots: d.slots } : {}),
        ...(m.package === undefined ? {} : { pkg: m.package }),
        ...(m.path === undefined ? {} : { sourcePath: m.path }),
      })
    }
  }
  return out
}

// The components explorer shows installed-pack components — sourced entirely
// from the SUI catalog. Views are not components; they live in the view-picker.
export async function loadCatalog(): Promise<CatalogEntry[]> {
  return loadSuiComponents()
}
