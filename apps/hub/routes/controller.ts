// Hub home controller — the `/` route's view scope.
//
// The home is a desktop: a greeting hero, a live git-glance, and a launcher
// dock. Two data sources:
//   • greeting — owned entirely by <ship-home-greeting> (pure-derived, no SDK).
//     This controller only feeds it the `workspace` label.
//   • git      — `ship.git.overview()` over the LIVE SSE reader. git.overview
//     has a registered mux resolver (apps/shipd/src/ship_runtime/
//     live_resolvers.rs), so `ctx.clients.ship` streams it; `{#async git}`
//     consumes the LiveSignal. The derived helpers read it defensively
//     through `unwrapOr` so the template binds plain expressions.
//
// Launch: <ship-home-dock> renders one <ship-dock-app> per `apps` entry and
// bubbles the tile's `ship-launch { label }` up here. `onLaunch` maps the
// label to its route and navigates — the dock stays route-agnostic, this
// host owns the hub's real routes. Targets are hub routes that exist today
// (Studio + the five /studio/* sections, Settings, Account); no links to
// surfaces the hub lacks.
//
// Degraded honestly: the spaces hero (spaces.list) and activity runs
// (pipelines.summaries) have no live resolver in the hub yet, so they are
// omitted from this take rather than shown as broken/empty widgets.
import { controller, derived, live, unwrapOr } from '@ship/sui-runtime'
import type { LiveSignal, ViewScope } from '@ship/sui-runtime'
import type { GitOverview } from '@ship/sdk-client'
import type { Scope } from './page.sui'

interface DockApp { label: string; icon: string; href: string }

const APPS: readonly DockApp[] = [
  { label: 'Studio', icon: 'sliders', href: '/studio' },
  { label: 'Agents', icon: 'bot', href: '/studio/agents' },
  { label: 'Skills', icon: 'sparkles', href: '/studio/skills' },
  { label: 'Components', icon: 'box', href: '/studio/components' },
  { label: 'Collections', icon: 'database', href: '/studio/collections' },
  { label: 'Themes', icon: 'palette', href: '/studio/themes' },
  { label: 'Settings', icon: 'settings', href: '/settings' },
  { label: 'Account', icon: 'user', href: '/account' },
]

const EMPTY_GIT: GitOverview = {
  workspace_id: null, branch: '', head_short: '', clean: true, total_changed: 0,
  staged: [], modified: [], untracked: [], commits: [], branches: [], ahead: 0, behind: 0,
}

export default controller({
  scope: (ctx: ViewScope<Scope>) => {
    const git = live(() => ctx.clients.ship.git.overview())
    const g = (): GitOverview => unwrapOr(git as LiveSignal<GitOverview>, EMPTY_GIT)

    const workspace = derived<string>(() => 'ship')
    const apps = derived<readonly DockApp[]>(() => APPS)

    const branchName = derived<string>(() => g().branch ?? '')
    const headShort = derived<string>(() => g().head_short ?? '')
    const isClean = derived<boolean>(() => g().clean)
    const statusLabel = derived<string>(() => (g().clean ? 'clean' : `${g().total_changed} changed`))

    return { git, workspace, apps, branchName, headShort, isClean, statusLabel }
  },

  on: {
    onLaunch: (s, e: CustomEvent<{ label?: string }>) => {
      const app = APPS.find((a) => a.label === e.detail?.label)
      if (app) s.navigate(app.href)
    },
  },
})
