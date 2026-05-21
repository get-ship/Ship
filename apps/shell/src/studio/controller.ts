// Studio surface — internal navigation state for the rail + breadcrumb
// + section content slot. The route shape is `/studio/:section/:sub?`,
// with `section` defaulting to `agents` (the most-used surface). SDK
// counts populate the rail badges where data exists; everything else
// displays a dash until a follow-up job lands the backing data.

import { controller, unwrap, chrome, signal, live } from '@ship/sui-runtime'
import type { LiveSignal, ViewScope, MutableState } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

// ── Studio section elements ──────────────────────────────────────────
// Each Studio section renders as a registered `<sui-studio-<name>>`
// custom element. Side-importing the element here bundles it into
// sui-studio.js and triggers its `customElements.define()`, so
// `page.sui`'s content slot can dispatch on `section()` to the matching
// tag (one `{#if}` arm per section). Sections without an element fall
// back to the shared "Coming soon" placeholder.
//
// To add a section: author the element in its own tag-named directory
// `apps/shell/src/studio/<name>/sui-studio-<name>/sui-studio-<name>.sui`
// (the dir name MUST match the tag so the compiler builds it in
// component mode), add its import below, and add a
// `{:else if section() === '<name>'}` arm in page.sui's content slot.
// Nothing else in the shell needs to change.
import './themes/sui-studio-themes/sui-studio-themes.sui'
import './components/sui-studio-components/sui-studio-components.sui'
import './skills/sui-studio-skills/sui-studio-skills.sui'

interface ListEnvelope { items?: unknown[] }

// Route patterns for `/studio/:section/:sub?` are registered by the shell
// (apps/shell/src/main.ts) so the shared routeSignals resolve `section`
// and `sub` from the URL. This controller reads them via `route.params`
// and navigates through `chrome.navigate`, which the shell routes through
// its same-shell handling (no remount between sections).

export type RailGroup = 'project' | 'configure' | 'library'

export interface ProjectSub { sub: string; label: string; icon: string }
export interface LibrarySub { sub: string; label: string; icon: string; staticCount: string }
export interface ConfigureItem {
  id: string
  label: string
  icon: string
  section: string
  staticCount?: string
}

// Project sub-nav. Only surfaces backed by a real daemon capability or an
// acknowledged-future one. Overview reads workspace/settings; Isolation is
// coming-soon (no API yet). Network/Collaborators/Lifecycle were designer
// mockup feels with no backing — cut.
const PROJECT_SUBS: ReadonlyArray<ProjectSub> = [
  { sub: 'overview',  label: 'Overview',  icon: 'circle' },
  { sub: 'isolation', label: 'Isolation', icon: 'box' },
]

// Library — a real concept, but no backend yet, so every sub is coming-soon
// (no fabricated counts). "Authored" → "Publishing" (the eventual flow).
const LIBRARY_SUBS: ReadonlyArray<LibrarySub> = [
  { sub: 'installed',   label: 'Installed',   icon: 'package', staticCount: '—' },
  { sub: 'marketplace', label: 'Marketplace', icon: 'store',   staticCount: '—' },
  { sub: 'publishing',  label: 'Publishing',  icon: 'upload',  staticCount: '—' },
]

// Configuration sections. Real + built first (no count rather than a stale
// fabricated one); deferred-but-real APIs (pipelines/mcp/permissions) follow
// and render the honest "Coming soon" stub until their UI lands.
const CONFIGURE_ITEMS: ReadonlyArray<ConfigureItem> = [
  { id: 'agents',      label: 'Agents',      icon: 'bot',      section: 'agents' },
  { id: 'skills',      label: 'Skills',      icon: 'zap',      section: 'skills' },
  { id: 'components',  label: 'Components',  icon: 'layers',   section: 'components' },
  { id: 'themes',      label: 'Themes',      icon: 'palette',  section: 'themes' },
  { id: 'pipelines',   label: 'Pipelines',   icon: 'workflow', section: 'pipelines',   staticCount: '—' },
  { id: 'mcp',         label: 'MCP servers', icon: 'terminal', section: 'mcp',         staticCount: '—' },
  { id: 'permissions', label: 'Permissions', icon: 'shield',   section: 'permissions', staticCount: '—' },
]

const SECTION_META: Record<string, { label: string; sub: string }> = {
  project:     { label: 'Project',      sub: 'Configuration for this workspace.' },
  agents:      { label: 'Agents',       sub: 'Agents you have configured for this project.' },
  skills:      { label: 'Skills',       sub: 'Skills available to agents in this project.' },
  mcp:         { label: 'MCP servers',  sub: 'External tools wired through the MCP boundary.' },
  pipelines:   { label: 'Pipelines',    sub: 'Reusable multi-step workflows.' },
  components:  { label: 'Components',   sub: 'SUI primitives available to views.' },
  themes:      { label: 'Themes',       sub: 'Visual themes for the workspace.' },
  permissions: { label: 'Permissions',  sub: 'Tool, network, and filesystem permissions.' },
  library:     { label: 'Library',      sub: 'Installed, marketplace, and published extensions.' },
}

const DEFAULT_SECTION = 'agents'
const VALID_SECTIONS = new Set(Object.keys(SECTION_META))

function resolveSection(raw: string | undefined): string {
  if (!raw) return DEFAULT_SECTION
  return VALID_SECTIONS.has(raw) ? raw : DEFAULT_SECTION
}

// Parse `/studio/<section>/<sub>` directly off a pathname. The studio
// bundle owns its own signal universe, so reading the shell-injected
// `route` signal can't drive reactivity across the bundle boundary (the
// global route updater iterates only the installing bundle's instances).
// Deriving from a local path signal — fed by the shell's `sui:navigate`
// event + popstate in mount() — keeps section reactivity inside this
// bundle, so rail clicks actually swap the content.
function sectionFromPath(pathname: string): string {
  const segs = pathname.split('/').filter(Boolean)
  // ['studio', '<section>', '<sub>']
  return resolveSection(segs[1])
}
function subFromPath(pathname: string): string {
  const segs = pathname.split('/').filter(Boolean)
  return segs[2] ?? ''
}

function dispatchStudioEvent(name: string, detail?: Record<string, unknown>): void {
  if (typeof window === 'undefined') return
  window.dispatchEvent(new CustomEvent(name, { detail: detail ?? {} }))
}

export default controller<Scope>({
  scope: ({ ship }) => {
    // `ship.*.()` returns a LiveSource (an SSE channel), not a LiveSignal.
    // `live()` subscribes it into a readable signal so `unwrap()` works —
    // calling unwrap on the raw source threw `sig is not a function`.
    const agentSummaries = live(() => ship.agents.summaries())
    const skillsList = live(() => ship.skills.list())
    const studioPath = signal(
      typeof location !== 'undefined' ? location.pathname : '/studio',
    )
    return { agentSummaries, skillsList, studioPath }
  },

  derived: {
    section: ({ studioPath }) => sectionFromPath(studioPath()),
    subitem: ({ studioPath }) => subFromPath(studioPath()),
    agentCount: ({ agentSummaries }) => {
      const v = unwrap(agentSummaries as LiveSignal<ListEnvelope>)
      return v?.items?.length ?? 0
    },
    skillCount: ({ skillsList }) => {
      const v = unwrap(skillsList as LiveSignal<ListEnvelope>)
      return v?.items?.length ?? 0
    },
    projectSubs: () => PROJECT_SUBS,
    librarySubs: () => LIBRARY_SUBS,
    configureItems: () => CONFIGURE_ITEMS,
    sectionTitle: ({ studioPath }) => {
      const s = sectionFromPath(studioPath())
      return SECTION_META[s]?.label ?? 'Studio'
    },
    sectionSubtitle: ({ studioPath }) => {
      const s = sectionFromPath(studioPath())
      return SECTION_META[s]?.sub ?? ''
    },
    breadcrumbCurrent: ({ studioPath }) => {
      const sub = subFromPath(studioPath())
      if (sub) return sub
      const s = sectionFromPath(studioPath())
      return SECTION_META[s]?.label ?? 'Studio'
    },
    // The CONFIGURE group header shows the static count of configurable
    // section types. Static — the rail entries are a fixed schema.
    configureCount: () => CONFIGURE_ITEMS.length,
  },

  on: {
    goSection: (_state: MutableState<Scope>, section: string, sub?: string) => {
      const target = sub ? `/studio/${section}/${sub}` : `/studio/${section}`
      chrome.navigate(target)
    },
    closeStudio: () => {
      dispatchStudioEvent('sui-close-studio')
    },
    newInSection: (state: ViewScope<Scope>) => {
      const studioPath = (state as unknown as { studioPath: () => string }).studioPath
      dispatchStudioEvent('sui-studio-new', { section: sectionFromPath(studioPath()) })
    },
  },
})
