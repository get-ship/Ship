// <sui-home> — Ship's per-space desktop. A first-party CHROME surface (lives
// in apps/shell, not views/): the shell renders it at `/`. Owns its dock; the
// shell hides the status strip for it. Data comes over @ship/sdk-client live
// sources (chrome has no `ctx.ship` the way daemon-served views do).
import { controller, derived, live, unwrapOr, chrome } from '@ship/sui-runtime'
import type { LiveSignal, ViewScope } from '@ship/sui-runtime'
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
  { name: 'Collections', icon: 'database', href: '/studio/collections' },
  { name: 'Git', icon: 'git-branch', href: '/git' },
  { name: 'Studio', icon: 'sliders', href: '/studio' },
  { name: 'Library', icon: 'package', href: '/library' },
  { name: 'Activity', icon: 'activity', href: '/activity' },
]

const EMPTY_DASH: DashState = { stats: { active: 0, history: 0 }, agents: [], recent_runs: [], events: [], collections: [], total_events: 0 }
const EMPTY_GIT: GitOverview = { workspace_id: null, branch: '', head_short: '', clean: true, total_changed: 0, staged: [], modified: [], untracked: [], commits: [], branches: [], ahead: 0, behind: 0 }

const initialOf = (a: DashAgent): string => (a.letter ?? (a.name ?? a.id).slice(0, 1) ?? '?').toString().toUpperCase()

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    const dash = live(() => liveExpr<DashState>('ship.dashboard.state()'))
    const git = live(() => liveExpr<GitOverview>('ship.git.overview()'))
    const navigate = (path: string): void => chrome.navigate(path)
    const goNav = (event: Event, path: string): void => { event.preventDefault(); chrome.navigate(path) }

    const d = (): DashState => unwrapOr(dash as LiveSignal<DashState>, EMPTY_DASH)
    const g = (): GitOverview => unwrapOr(git as LiveSignal<GitOverview>, EMPTY_GIT)

    const greeting = derived<string>(() => { const h = new Date().getHours(); return h < 12 ? 'Good morning' : h < 18 ? 'Good afternoon' : 'Good evening' })
    const heroLine = derived<string>(() => {
      const dash_ = d()
      const running = dash_.stats.active > 0 ? `${dash_.stats.active} job${dash_.stats.active === 1 ? '' : 's'} running` : 'Nothing running'
      return `${running} · ${dash_.agents.length} agent${dash_.agents.length === 1 ? '' : 's'} ready in this space.`
    })
    const totalJobs = derived<number>(() => d().stats.active + d().stats.history)
    const agentCount = derived<number>(() => d().agents.length)
    const totalEvents = derived<number>(() => d().total_events)
    const collectionsCount = derived<number>(() => d().collections.length)
    const recentJobs = derived(() => d().recent_runs)
    const recentEvents = derived(() => d().events)
    const agentSlice = derived<AgentRow[]>(() => d().agents.slice(0, 6).map((a) => ({ ...a, initials: initialOf(a) })))
    const apps = derived(() => APPS)
    const isFirstRun = derived<boolean>(() => { const dash_ = d(); return dash_.stats.active + dash_.stats.history === 0 && dash_.agents.length === 0 && dash_.events.length === 0 && dash_.collections.length === 0 })
    const branchName = derived<string>(() => g().branch ?? '')
    const headShort = derived<string>(() => g().head_short ?? '')
    const isClean = derived<boolean>(() => g().clean)
    const changedSummary = derived<string>(() => `${g().total_changed} changed`)

    return {
      dash, git, navigate, goNav,
      greeting, heroLine, totalJobs, agentCount, totalEvents, collectionsCount,
      recentJobs, recentEvents, agentSlice, apps, isFirstRun,
      branchName, headShort, isClean, changedSummary,
    }
  },
})
