// Studio surface — internal navigation state for the rail + breadcrumb
// + section content slot. The route shape is `/studio/:section/:sub?`,
// with `section` defaulting to `agents` (the most-used surface). SDK
// counts populate the rail badges where data exists; everything else
// displays a dash until a follow-up job lands the backing data.

import { controller, unwrap, chrome } from '@ship/sui-runtime'
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

const PROJECT_SUBS: ReadonlyArray<ProjectSub> = [
  { sub: 'overview',      label: 'Overview',      icon: 'circle' },
  { sub: 'isolation',     label: 'Isolation',     icon: 'box' },
  { sub: 'network',       label: 'Network',       icon: 'globe' },
  { sub: 'collaborators', label: 'Collaborators', icon: 'users' },
  { sub: 'lifecycle',     label: 'Lifecycle',     icon: 'activity' },
]

const LIBRARY_SUBS: ReadonlyArray<LibrarySub> = [
  { sub: 'installed',   label: 'Installed',   icon: 'package', staticCount: '22' },
  { sub: 'marketplace', label: 'Marketplace', icon: 'store',   staticCount: '—' },
  { sub: 'authored',    label: 'Authored',    icon: 'pen',     staticCount: '4' },
]

const CONFIGURE_ITEMS: ReadonlyArray<ConfigureItem> = [
  { id: 'agents',      label: 'Agents',      icon: 'bot',      section: 'agents' },
  { id: 'skills',      label: 'Skills',      icon: 'zap',      section: 'skills' },
  { id: 'mcp',         label: 'MCP servers', icon: 'terminal', section: 'mcp',         staticCount: '—' },
  { id: 'pipelines',   label: 'Pipelines',   icon: 'workflow', section: 'pipelines',   staticCount: '—' },
  { id: 'components',  label: 'Components',  icon: 'layers',   section: 'components',  staticCount: '94' },
  { id: 'themes',      label: 'Themes',      icon: 'palette',  section: 'themes',      staticCount: '7' },
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
  library:     { label: 'Library',      sub: 'Installed, marketplace, and authored extensions.' },
}

const DEFAULT_SECTION = 'agents'
const VALID_SECTIONS = new Set(Object.keys(SECTION_META))

function resolveSection(raw: string | undefined): string {
  if (!raw) return DEFAULT_SECTION
  return VALID_SECTIONS.has(raw) ? raw : DEFAULT_SECTION
}

function dispatchStudioEvent(name: string, detail?: Record<string, unknown>): void {
  if (typeof window === 'undefined') return
  window.dispatchEvent(new CustomEvent(name, { detail: detail ?? {} }))
}

export default controller<Scope>({
  scope: ({ ship }) => {
    const agentSummaries = ship.agents.summaries()
    const skillsList = ship.skills.list()
    return { agentSummaries, skillsList }
  },

  derived: {
    section: ({ route }) => resolveSection(route.params.section()),
    subitem: ({ route }) => route.params.sub() ?? '',
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
    sectionTitle: ({ route }) => {
      const s = resolveSection(route.params.section())
      return SECTION_META[s]?.label ?? 'Studio'
    },
    sectionSubtitle: ({ route }) => {
      const s = resolveSection(route.params.section())
      return SECTION_META[s]?.sub ?? ''
    },
    breadcrumbCurrent: ({ route }) => {
      const sub = route.params.sub() ?? ''
      if (sub) return sub
      const s = resolveSection(route.params.section())
      return SECTION_META[s]?.label ?? 'Studio'
    },
    // The CONFIGURE group header shows the static count of configurable
    // section types (7). Static — the rail entries are a fixed schema.
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
    openPalette: () => {
      dispatchStudioEvent('sui-open-studio-palette')
    },
    newInSection: ({ route }: ViewScope<Scope>) => {
      const section = resolveSection(route.params.section())
      dispatchStudioEvent('sui-studio-new', { section })
    },
  },
})
