// Shared catalog loader for ship-ui-studio.
// Fetches three sources: Lit CEM, SUI registry, daemon views.

export interface CatalogEntry {
  id: string
  tag: string
  kind: 'lit' | 'sui' | 'view'
  description?: string
  props?: PropEntry[]
}

export interface PropEntry {
  name: string
  type: string
  default?: string
  description?: string
}

// All 18 ported set-* components (kept as a static list since
// CustomElementRegistry has no standard enumeration API).
const SET_TAGS: string[] = [
  'sui-status-dot', 'sui-spinner', 'sui-badge', 'sui-chip', 'sui-card',
  'sui-link', 'sui-divider', 'sui-skeleton', 'sui-stat', 'sui-mark',
  'sui-kbd', 'sui-avatar', 'sui-progress', 'sui-button', 'sui-tabs',
  'sui-tab', 'sui-empty-state', 'sui-input',
]

// Baked SUI views not surfaced by /api/v1/views.
const BAKED_SUI_VIEWS: CatalogEntry[] = [
  { id: 'counter', tag: 'counter', kind: 'view', description: 'Simple counter demo' },
  { id: 'agents',  tag: 'agents',  kind: 'view', description: 'Agent list' },
  { id: 'git',     tag: 'git',     kind: 'view', description: 'Git overview' },
]

interface CemAttribute {
  name: string
  type?: { text: string }
  default?: string
  description?: string
}

interface CemDeclaration {
  kind: string
  tagName?: string
  description?: string
  attributes?: CemAttribute[]
  members?: Array<{
    kind: string
    name: string
    type?: { text: string }
    default?: string
    description?: string
    privacy?: string
  }>
}

interface CemModule {
  declarations?: CemDeclaration[]
  exports?: Array<{ kind: string; name: string; declaration?: { name: string } }>
}

interface CemRoot {
  modules?: CemModule[]
}

async function loadLitComponents(): Promise<CatalogEntry[]> {
  try {
    const res = await fetch('/api/assets/custom-elements.json', { credentials: 'same-origin' })
    if (!res.ok) return []
    const cem = await res.json() as CemRoot
    const entries: CatalogEntry[] = []
    for (const mod of cem.modules ?? []) {
      for (const decl of mod.declarations ?? []) {
        if (decl.kind !== 'class' || !decl.tagName) continue
        const props: PropEntry[] = (decl.members ?? [])
          .filter(m => m.kind === 'field' && !m.name.startsWith('_') && m.privacy !== 'private')
          .map(m => ({
            name: m.name,
            type: m.type?.text ?? 'string',
            default: m.default,
            description: m.description,
          }))
        entries.push({
          id: decl.tagName,
          tag: decl.tagName,
          kind: 'lit',
          description: decl.description,
          props,
        })
      }
    }
    return entries
  } catch {
    return []
  }
}

async function loadSuiComponents(): Promise<CatalogEntry[]> {
  const propsByTag = await loadSuiPropMap()
  return SET_TAGS.map(tag => ({
    id: tag,
    tag,
    kind: 'sui' as const,
    props: propsByTag[tag] ?? [],
  }))
}

// Fetch the daemon's aggregated SUI CEM and project it down to the prop
// shape the studio's prop panel renders. Empty map on any error so the
// catalog still renders even if the endpoint is unreachable.
async function loadSuiPropMap(): Promise<Record<string, PropEntry[]>> {
  try {
    const res = await fetch('/api/assets/sui-custom-elements.json', { credentials: 'same-origin' })
    if (!res.ok) return {}
    const cem = await res.json() as CemRoot
    const out: Record<string, PropEntry[]> = {}
    for (const mod of cem.modules ?? []) {
      for (const decl of mod.declarations ?? []) {
        if (decl.kind !== 'class' || !decl.tagName) continue
        const attrs = decl.attributes ?? []
        out[decl.tagName] = attrs.map(a => ({
          name: a.name,
          type: a.type?.text ?? 'string',
          default: a.default,
          description: a.description,
        }))
      }
    }
    return out
  } catch {
    return {}
  }
}

interface ApiView {
  name: string
  type?: string
  rail?: { label?: string; icon?: string }
}

async function loadDaemonViews(): Promise<CatalogEntry[]> {
  try {
    const res = await fetch('/api/v1/views', { credentials: 'same-origin' })
    if (!res.ok) return []
    const body = await res.json() as { data?: { views?: ApiView[] } }
    return (body.data?.views ?? [])
      .filter(v => v.type === 'sui')
      .map(v => ({
        id: v.name,
        tag: v.name,
        kind: 'view' as const,
        description: v.rail?.label,
      }))
  } catch {
    return []
  }
}

export interface CatalogResult {
  lit: CatalogEntry[]
  sui: CatalogEntry[]
  views: CatalogEntry[]
}

export async function loadCatalog(): Promise<CatalogResult> {
  // Storybook documents SUI components only. Lit primitives are
  // legacy production stack; documenting them in the SUI studio is
  // wasted effort unless we're doing intentional side-by-side
  // comparison work, which we're not.
  const [daemonViews, sui] = await Promise.all([
    loadDaemonViews(),
    loadSuiComponents(),
  ])

  // Merge baked views; skip any already returned by the daemon endpoint.
  const daemonIds = new Set(daemonViews.map(v => v.id))
  const extraViews = BAKED_SUI_VIEWS.filter(v => !daemonIds.has(v.id))
  const views = [...daemonViews, ...extraViews]

  return { lit: [], sui, views }
}
