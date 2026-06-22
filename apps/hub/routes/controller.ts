// Hub home controller — the index route's view scope.
//
// The home composes the `@ship/home` pack:
//   • <ship-home-greeting> — pure-derived greeting + workspace chip + clock
//   • <ship-spaces-hero>   — spaces grid (empty-state until the list-all
//                            SDK method lands — see .artifacts/hub-buildout-
//                            flags.md BACKEND-GAP)
//   • <ship-activity-runs> — recent pipeline runs (LiveSource over
//                            `ship.pipelines.summaries()`)
//
// This controller only owns the two page-level props the @ship/home
// components bind to: `workspace` (the label the greeting shows next to the
// "Good evening" title) and `spaces` (the JSON-encoded list the spaces
// hero parses — `[]` today). The greeting's own controller owns the
// greeting/clock; the spaces hero owns its own empty-state; the activity
// runs controller owns the live read + tick. Each component is the
// authoritative owner of its slice; this controller stays a thin
// host-side wiring seam.
import { controller, derived } from '@ship/sui-runtime'

export default controller({
  scope: () => {
    // Workspace label — the hub's daemon is workspace-scoped, so the
    // label is the literal "ship" today. When a project-switcher
    // lands (per the hub-buildout-flags doc), the source becomes
    // `ship.pipelines.summaries().workspace_id` and the constant falls
    // out for free.
    const workspace = derived<string>(() => 'ship')
    // Spaces list — `[]` until the Connect SDK ships a list-all
    // method. The spaces hero renders the empty-state on the empty /
    // unparseable contract; when the op lands, the controller flips
    // to `JSON.stringify(rows)` and the hero lights up.
    const spaces = derived<string>(() => '[]')
    return { workspace, spaces }
  },
})
