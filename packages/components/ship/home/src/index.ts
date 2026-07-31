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
//
// NOTE: <ship-activity-runs> is SDK-coupled (reads ship.pipelines.summaries
// via Connect SDK) and is NOT included in this ship-kit copy. It remains
// in the Ship hub repo.
//
// The pack composes <sui-card>, <sui-stack>, <sui-status-dot>,
// <sui-empty-state>, <sui-badge>, <sui-avatar>, <sui-icon> from
// @ship/sail-primitives (a declared dep the host must register before
// any of these tags mount).
import './ship-home-greeting/ship-home-greeting.compiled.js'
import './ship-spaces-hero/ship-spaces-hero.compiled.js'
