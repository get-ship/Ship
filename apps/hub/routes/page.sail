<controller>
// Hub home controller — the `/` route's view scope.
//
// The home is a desktop: a greeting hero, a live git-glance, and a launcher
// dock. Two data sources:
//   • greeting — owned entirely by <ship-home-greeting> (pure-derived, no SDK).
//     This controller only feeds it the `workspace` label.
//   • git      — `ship.git.overview()` over the LIVE SSE reader. git.overview
//     has a registered mux resolver (apps/shipd/src/ship_runtime/
//     live_resolvers.rs), so `ctx.ship` streams it; ``
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
import { controller, derived, live, unwrapOr } from '@ship/sail-runtime'
import type { LiveSignal, ViewScope } from '@ship/sail-runtime'
import type { GitOverview } from '@ship/sdk-client'
import type { Scope } from './page.sail'

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

export default controller((ctx: ViewScope<Scope>) => {

    const git = live(() => ctx.ship.git.overview())
    const g = (): GitOverview => unwrapOr(git as LiveSignal<GitOverview>, EMPTY_GIT)

    const workspace = derived<string>(() => 'ship')
    const apps = derived<readonly DockApp[]>(() => APPS)

    const branchName = derived<string>(() => g().branch ?? '')
    const headShort = derived<string>(() => g().head_short ?? '')
    const isClean = derived<boolean>(() => g().clean)
    const statusLabel = derived<string>(() => (g().clean ? 'clean' : `${g().total_changed} changed`))

    
    const onLaunch = (e: CustomEvent<{ label?: string }>) => {
      const app = APPS.find((a) => a.label === e.detail?.label)
      if (app) ctx.navigate(app.href)
    }
    return { git, workspace, apps, branchName, headShort, isClean, statusLabel, onLaunch }
})
</controller>

<!--
  Hub home — the `/` route. A desktop, not a directory.

  Three parts, top to bottom:
    • <ship-home-greeting>  the hero strip (hour-of-day greeting + workspace
                            chip + clock). Pure-derived, client-side; the
                            controller only feeds it the `workspace` label.
    • git-glance            a live `ship.git.overview()` readout — branch,
                            short HEAD, clean/dirty dot. git.overview is a
                            registered live resolver in the hub today, so
                            `` streams loading → ok/offline.
    • <ship-home-dock>      the launcher dock — a glassmorphic tray of
                            <ship-dock-app> tiles. Each tile bubbles
                            `ship-launch { label }`; `onLaunch` maps it to a
                            real hub route and navigates.

  Page chrome:
    • the top nav is owned by the root layout (`<ship-app-chrome>`).
    • the desktop background (masked dot-grid + corner glow), the stage
      layout, and the dock row live in `routes/style.css` (FW-4: no inline
      route <style>); the components carry their own shadow-DOM styles.
    • tokens only — no hardcoded colors / spacing / radii.

  Degraded honestly: the spaces hero (`spaces.list`) and activity runs
  (`pipelines.summaries`) have no live resolver in the hub yet, so they are
  omitted from this take rather than rendered as broken / empty widgets.
-->

<main data-view-enter data-test="home">
  <div aria-hidden="true"></div>

  <div>
    <ship-home-greeting workspace={workspace()}></ship-home-greeting>

    <section aria-labelledby="home-git-h" data-test="home-git">
      <h2 id="home-git-h">Git</h2>
      
        <while-loading of="git">
        <span data-test="git-loading">Loading&hellip;</span>
        </while-loading><while-error as="err" of="git">
        <span data-test="git-error">offline</span>
        </while-error><if cond={git().kind === 'ok'}>
        <div data-test="git-summary">
          <span data-clean={isClean() ? '1' : '0'} aria-hidden="true"></span>
          <strong data-test="git-branch">{branchName()}</strong>
          <span data-test="git-head">{headShort()}</span>
          <span data-test="git-status">{statusLabel()}</span>
        </div>
      </if>
    </section>
  </div>

  <footer>
    <ship-home-dock apps={apps()} on:ship-launch={onLaunch} data-test="home-dock"></ship-home-dock>
  </footer>
</main>
