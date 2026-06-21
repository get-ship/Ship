// Hub home controller — the index route's view scope.
//
// The page composes the @ship/home pack (<ship-home-greeting>,
// <ship-spaces-hero>, <ship-activity-runs>). The pack controllers own
// their own SDK wiring (pipelines.summaries); this controller just
// feeds the page-level bindings (workspace name + spaces list).
//
// `name` is a no-op constant signal whose read on the template is
// what triggers the compiler to emit `hydrate` — see slice-2 note.
//
// The `spacesJson` binding is a JSON-encoded array of
// `{ id, name, path }`. Today the Connect SDK exposes no list-all-
// spaces method (a known BACKEND-GAP — see .artifacts/hub-buildout-
// flags.md), so this controller passes `[]`. The <ship-spaces-hero>
// component renders the empty-state; when the SDK lands, this becomes
// `live(() => ship.spaces.list())` and the JSON-serialized output
// flows into the attribute.
//
// `workspace` is a derived from the route's path; the home page has
// no per-space context yet, so we render the static "ship" label.
import { controller, derived, signal } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    const name = signal('Hub')

    // Today: empty list. The list-all-spaces SDK method is a
    // BACKEND-GAP (see .artifacts/hub-buildout-flags.md); the
    // <ship-spaces-hero> component degrades to its empty-state.
    // When ship.spaces.list() lands, switch this to:
    //   const spaces = live(() => ship.spaces.list() as LiveSource<SpaceRow[]>)
    //   const spacesJson = derived(() => JSON.stringify(spaces() ?? []))
    const spacesJson = derived<string>(() => '[]')

    // The workspace label is the host's project name; today the hub
    // is the single project context, so we surface a stable string.
    // When the host grows a project-switcher, this becomes a derived
    // over the active project's id.
    const workspace = derived<string>(() => 'ship')

    return { name, spacesJson, workspace }
  },
})
