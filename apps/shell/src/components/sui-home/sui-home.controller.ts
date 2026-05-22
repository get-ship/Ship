// <sui-home> — Ship's per-space desktop. A first-party CHROME surface (lives
// in apps/shell, not views/): the shell renders it at `/`. Owns its dock; the
// shell hides the status strip for it. Data comes over @ship/sdk-client live
// sources (chrome has no `ctx.ship` the way daemon-served views do).
import { controller, derived, live, unwrapOr, chrome } from '@ship/sui-runtime'
import { liveExpr } from '@ship/sdk-client'
import type { GitOverview } from '@ship/sdk-client'
import type { Scope } from './sui-home.sui'

interface DashStats { active: number; history: number }
interface DashAgent { id: string; name?: string; description?: string; letter?: string }
interface DashJob { id: string; slug?: string; agent?: string; tone?: string; started_at?: string; created_at?: string }
interface DashEvent { id?: string; timestamp: string; event_type: string; actor?: string }
interface DashState {
  stats: DashStats; agents: DashAgent[]; recent_runs: DashJob[]; events: DashEvent[]
  collections: unknown[]; total_events: number; settings?: { workspace_name?: string }
}
interface AppLauncher { name: string; icon: string; href: string }
interface AgentRow extends DashAgent { initials: string }

const APPS: readonly AppLauncher[] = [
  { name: 'Canvas', icon: 'image', href: '/canvas' },
  { name: 'Pipelines', icon: 'workflow', href: '/studio/pipelines' },
  { name: 'Collections', icon: 'database', href: '/collections' },
  { name: 'Git', icon: 'git-branch', href: '/git' },
  { name: 'Studio', icon: 'sliders', href: '/studio' },
  { name: 'Library', icon: 'package', href: '/library' },
  { name: 'Activity', icon: 'activity', href: '/activity' },
]

const EMPTY_DASH: DashState = { stats: { active: 0, history: 0 }, agents: [], recent_runs: [], events: [], collections: [], total_events: 0 }
const EMPTY_GIT: GitOverview = { workspace_id: null, branch: '', head_short: '', clean: true, total_changed: 0, staged: [], modified: [], untracked: [], commits: [], branches: [], ahead: 0, behind: 0 }

const initialOf = (a: DashAgent): string => (a.letter ?? (a.name ?? a.id).slice(0, 1) ?? '?').toString().toUpperCase()
const readDash = ({ dash }: Scope): DashState => unwrapOr(dash, EMPTY_DASH as unknown) as DashState
const readGit = ({ git }: Scope): GitOverview => unwrapOr(git, EMPTY_GIT)

export default controller<Scope>({
  scope: () => ({
    dash: live(() => liveExpr<DashState>('ship.dashboard.state()')),
    git: live(() => liveExpr<GitOverview>('ship.git.overview()')),
    navigate: (path: string): void => chrome.navigate(path),
    goNav: (event: Event, path: string): void => { event.preventDefault(); chrome.navigate(path) },
  }),
  derived: {
    greeting: () => { const h = new Date().getHours(); return h < 12 ? 'Good morning' : h < 18 ? 'Good afternoon' : 'Good evening' },
    heroLine: (s) => {
      const d = readDash(s)
      const running = d.stats.active > 0 ? `${d.stats.active} job${d.stats.active === 1 ? '' : 's'} running` : 'Nothing running'
      return `${running} · ${d.agents.length} agent${d.agents.length === 1 ? '' : 's'} ready in this space.`
    },
    totalJobs: (s) => readDash(s).stats.active + readDash(s).stats.history,
    agentCount: (s) => readDash(s).agents.length,
    totalEvents: (s) => readDash(s).total_events,
    collectionsCount: (s) => readDash(s).collections.length,
    recentJobs: (s) => readDash(s).recent_runs,
    recentEvents: (s) => readDash(s).events,
    agentSlice: (s): AgentRow[] => readDash(s).agents.slice(0, 6).map((a) => ({ ...a, initials: initialOf(a) })),
    apps: () => APPS,
    isFirstRun: (s) => { const d = readDash(s); return d.stats.active + d.stats.history === 0 && d.agents.length === 0 && d.events.length === 0 && d.collections.length === 0 },
    branchName: (s) => readGit(s).branch ?? '',
    headShort: (s) => readGit(s).head_short ?? '',
    isClean: (s) => readGit(s).clean,
    changedSummary: (s) => `${readGit(s).total_changed} changed`,
  },
})
