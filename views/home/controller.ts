// Home view — workspace landing surface. Folds the OG j2 dashboard
// (stats / recent jobs / agents / activity / quick links) and the
// at-a-glance pieces of the j2 overview (live git status, project
// summary) into a single SUI view. The dashboard envelope flows in
// through `<data name="dash" source="ship.dashboard.state()" live />`;
// git overview through a second `<data>` block so the SSE channel
// shows live status without coupling the dashboard fetch.

import { controller, unwrapOr, chrome } from '@ship/sui-runtime'
import type { GitOverview } from '@ship/sdk-client'
import type { Scope } from './page.sui'

interface DashStats { active: number; history: number }
interface DashAgent {
  id: string
  name?: string
  description?: string
  letter?: string
}
interface DashJob {
  id: string
  slug?: string
  agent?: string
  tone?: string
  started_at?: string
  created_at?: string
}
interface DashEvent {
  id?: string
  timestamp: string
  event_type: string
  actor?: string
}
interface DashState {
  stats: DashStats
  agents: DashAgent[]
  recent_runs: DashJob[]
  events: DashEvent[]
  collections: unknown[]
  total_events: number
  settings?: { workspace_name?: string }
}

interface QuickLink { key: string; label: string; icon: string; href: string }

const EMPTY_DASH: DashState = {
  stats: { active: 0, history: 0 },
  agents: [],
  recent_runs: [],
  events: [],
  collections: [],
  total_events: 0,
}

const EMPTY_GIT: GitOverview = {
  workspace_id: null,
  branch: '',
  head_short: '',
  clean: true,
  total_changed: 0,
  staged: [],
  modified: [],
  untracked: [],
  commits: [],
  branches: [],
  ahead: 0,
  behind: 0,
}

const LINKS: readonly QuickLink[] = [
  { key: 'workflows', label: 'Workflows', icon: 'workflow', href: '/pipelines'   },
  { key: 'agents',    label: 'Agents',    icon: 'users',    href: '/studio/agents'      },
  { key: 'skills',    label: 'Skills',    icon: 'book',     href: '/studio/skills' },
  { key: 'data',      label: 'Data',      icon: 'database', href: '/collections' },
  { key: 'events',    label: 'Events',    icon: 'activity', href: '/events'      },
  { key: 'settings',  label: 'Settings',  icon: 'settings', href: '/settings'    },
]

interface AgentRow extends DashAgent { initials: string }

const initialOf = (agent: DashAgent): string =>
  (agent.letter ?? (agent.name ?? agent.id).slice(0, 1) ?? '?').toString().toUpperCase()

const readDash = ({ dash }: Scope): DashState =>
  unwrapOr(dash, EMPTY_DASH as unknown) as DashState

const readGit = ({ git }: Scope): GitOverview =>
  unwrapOr(git, EMPTY_GIT)

export default controller<Scope>({
  scope: () => ({
    navigate: (path: string): void => chrome.navigate(path),
    goNav: (event: Event, path: string): void => {
      event.preventDefault()
      chrome.navigate(path)
    },
  }),

  derived: {
    workspaceLabel:   (s) => readDash(s).settings?.workspace_name ?? 'workspace',
    totalJobs:        (s) => readDash(s).stats.active + readDash(s).stats.history,
    activeJobs:       (s) => readDash(s).stats.active,
    agentCount:       (s) => readDash(s).agents.length,
    totalEvents:      (s) => readDash(s).total_events,
    collectionsCount: (s) => readDash(s).collections.length,
    recentJobs:       (s) => readDash(s).recent_runs,
    recentEvents:     (s) => readDash(s).events,
    agentSlice:       (s): AgentRow[] => readDash(s).agents.slice(0, 6).map((a) => ({
      ...a,
      initials: initialOf(a),
    })),
    quickLinks:       () => LINKS,
    isFirstRun: (s) => {
      const d = readDash(s)
      return d.stats.active + d.stats.history === 0
        && d.agents.length === 0
        && d.events.length === 0
        && d.collections.length === 0
    },
    branchName:     (s) => readGit(s).branch ?? '',
    headShort:      (s) => readGit(s).head_short ?? '',
    isClean:        (s) => readGit(s).clean,
    changedSummary: (s) => `${readGit(s).total_changed} changed`,
  },
})
