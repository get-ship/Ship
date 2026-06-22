// @ship/home — Ship Home pack (ship-* tags for the hub `/` dashboard).
//
// Each <tag>.compiled.js calls customElements.define() on import, so a
// single `import '@ship/home'` registers every tag globally.
//
// Composites:
//   <ship-home-greeting>  greeting strip — hour-of-day greeting + workspace
//                         label + clock. Pure-derived presenter; no SDK.
//   <ship-spaces-hero>    the spaces grid; BACKEND-GAP today (no list-all
//                         SDK method) — degrades to the empty-state until
//                         the backend lands a list op. Pure presenter.
//   <ship-activity-runs>  recent pipeline runs card; reads
//                         ship.pipelines.summaries() (Connect SDK) and
//                         renders the active_runs list. Empty state on
//                         loading + zero runs.
//
// The pack composes <sui-card>, <sui-stack>, <sui-status-dot>,
// <sui-empty-state>, <sui-badge>, <sui-avatar>, <sui-icon> from
// @ship/sui-primitives (a declared dep the host must register before
// any of these tags mount).
import './ship-home-greeting/ship-home-greeting.compiled.js'
import './ship-spaces-hero/ship-spaces-hero.compiled.js'
import './ship-activity-runs/ship-activity-runs.compiled.js'
