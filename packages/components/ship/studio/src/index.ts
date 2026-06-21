// @ship/studio — Ship Studio pack (ship-* tags for the hub /studio/* surfaces).
//
// Each <tag>.compiled.js calls customElements.define() on import, so a
// single `import '@ship/studio'` registers every tag globally.
//
// Composites:
//   <ship-component-catalog>  the hub `/studio/components` index. Renders
//                             the documented BACKEND-GAP empty-state
//                             today; the controller's pure-derived
//                             `entries` / `groups` / `state` shape is the
//                             stable contract — a future
//                             `ship.ui.components.list()` Connect op
//                             flips the `state() === 'ready'` branch on
//                             without a template change.
//
//   <ship-component-viewer>   the hub `/studio/components/:name` viewer.
//                             Same BACKEND-GAP story; same stable
//                             `entry` / `state` / `displayTag` /
//                             `category` shape. The route passes the
//                             `[name]` param as the `tag` prop; the
//                             category is a stable public fact about
//                             the tag, so the meta line works today.
//
// The pack composes <sui-empty-state>, <sui-stack>, <sui-icon>,
// <sui-card> from @ship/sui-primitives. No new primitives, no forked
// layout — per the placement hard-rule (charter §Placement).
import './ship-component-catalog/ship-component-catalog.compiled.js'
import './ship-component-viewer/ship-component-viewer.compiled.js'
