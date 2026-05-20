import { controller, signal, live, chrome } from '@ship/sui-runtime'
import { liveExpr } from '@ship/sdk-client'
import type { Scope } from './sui-skills-list.sui'

export type ScopeKind = 'all' | 'project' | 'user'

interface SkillItem { id: string; name: string; description?: string }
interface SkillGroup { source: string; skills: SkillItem[] }
interface SummariesValue { groups: SkillGroup[]; kind_counts: Array<{ id: string; count: number }> }

export interface SkillRow {
  id: string
  name: string
  description: string
  color: string
  char: string
  scope: 'project' | 'user'
  scopeLabel: string
}

export interface RenderedGroup {
  source: string
  label: string
  skills: SkillRow[]
}

const TILE_COLORS = ['#d97757', '#c98a2c', '#b94a4a', '#7a5cd6', '#4376d6', '#2a8a8a', '#4f9e54', '#c25a8e', '#5b6878']

export function tileColor(id: string): string {
  return TILE_COLORS[(id.codePointAt(0) ?? 0) % TILE_COLORS.length]
}

export function tileChar(name: string): string {
  return (name[0] ?? '?').toUpperCase()
}

export function computeCounts(
  groups: SkillGroup[],
): { all: number; project: number; user: number } {
  const all = groups.reduce((n, g) => n + g.skills.length, 0)
  const project = groups.find((g) => g.source === 'PROJECT SKILLS')?.skills.length ?? 0
  const user = groups.find((g) => g.source === 'USER SKILLS')?.skills.length ?? 0
  return { all, project, user }
}

function renderGroups(raw: SkillGroup[]): RenderedGroup[] {
  return raw.map((g) => {
    const isProject = g.source === 'PROJECT SKILLS'
    return {
      source: g.source,
      label: isProject ? 'Project' : g.source === 'USER SKILLS' ? 'User' : g.source,
      skills: g.skills.map((s) => ({
        id: s.id,
        name: s.name,
        description: s.description ?? '',
        color: tileColor(s.id),
        char: tileChar(s.name),
        scope: isProject ? ('project' as const) : ('user' as const),
        scopeLabel: isProject ? 'project' : 'user',
      })),
    }
  })
}

export default controller<Scope>({
  scope: () => {
    const query = signal('')
    const activeKind = signal<ScopeKind>('all')
    const summaries = live(() =>
      liveExpr<SummariesValue>(
        `ship.skills.summaries(${JSON.stringify({
          q: query(),
          kind: activeKind() === 'all' ? '' : activeKind(),
        })})`,
      ),
    )
    return { query, activeKind, summaries }
  },

  derived: {
    groups: ({ summaries }) => {
      const st = summaries()
      if (st.kind !== 'ok' || !st.value) return [] as RenderedGroup[]
      return renderGroups((st.value as SummariesValue).groups ?? [])
    },
    counts: ({ summaries }) => {
      const st = summaries()
      if (st.kind !== 'ok' || !st.value) return { all: 0, project: 0, user: 0 }
      return computeCounts((st.value as SummariesValue).groups ?? [])
    },
  },

  on: {
    open: (_s, id: string) => chrome.navigate(`/studio/skills/${id}`),
    setKind: (state, kind: ScopeKind) => { state.activeKind = kind },
    onSearch: (state, e: Event) => {
      state.query = (e.target as HTMLInputElement).value
    },
    newSkill: () => {
      if (typeof window === 'undefined') return
      window.dispatchEvent(new CustomEvent('sui-studio-new', { detail: { section: 'skills' } }))
    },
  },
})
