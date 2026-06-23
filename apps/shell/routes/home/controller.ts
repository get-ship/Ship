// <sui-home> — Ship's per-space desktop. A first-party CHROME surface (lives
// in apps/shell, not views/): the shell renders it at `/`. Owns its dock; the
// shell hides the status strip for it. Data comes over @ship/sdk-client live
// sources (chrome has no `ctx.ship` the way daemon-served views do).
//
// The dashboard aggregate (`dashboard.state`) is a ghost op with no backend
// builder (see apps/shipd/src/live_resolvers.rs) — DashboardService was culled.
// The home surface renders from the live sources that actually exist: git is
// read-only via the mux resolver (`git.overview`). Stats/jobs/agents/activity
// widgets that hung off the dead aggregate are gone, not stubbed.
import { controller, derived, live, unwrapOr, chrome } from '@ship/sui-runtime'
import type { LiveSignal, ViewScope } from '@ship/sui-runtime'
import type { GitOverview } from '@ship/sdk-client'
import { ship } from '../../ops'
import type { Scope } from './page.sui'

interface AppLauncher { name: string; icon: string; href: string }

const APPS: readonly AppLauncher[] = [
  { name: 'Canvas', icon: 'image', href: '/canvas' },
  { name: 'Pipelines', icon: 'workflow', href: '/studio/pipelines' },
  { name: 'Git', icon: 'git-branch', href: '/git' },
  { name: 'Studio', icon: 'sliders', href: '/studio' },
  { name: 'Library', icon: 'package', href: '/library' },
  { name: 'Activity', icon: 'activity', href: '/activity' },
]

const EMPTY_GIT: GitOverview = { workspace_id: null, branch: '', head_short: '', clean: true, total_changed: 0, staged: [], modified: [], untracked: [], commits: [], branches: [], ahead: 0, behind: 0 }

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    const git = live(() => ship.git.overview())
    const navigate = (path: string): void => chrome.navigate(path)
    const goNav = (event: Event, path: string): void => { event.preventDefault(); chrome.navigate(path) }

    const g = (): GitOverview => unwrapOr(git as LiveSignal<GitOverview>, EMPTY_GIT)

    const greeting = derived<string>(() => { const h = new Date().getHours(); return h < 12 ? 'Good morning' : h < 18 ? 'Good afternoon' : 'Good evening' })
    const apps = derived(() => APPS)
    const branchName = derived<string>(() => g().branch ?? '')
    const headShort = derived<string>(() => g().head_short ?? '')
    const isClean = derived<boolean>(() => g().clean)
    const changedSummary = derived<string>(() => `${g().total_changed} changed`)

    return {
      git, navigate, goNav,
      greeting, apps,
      branchName, headShort, isClean, changedSummary,
    }
  },
})
