// Marketing bundle membership — the SINGLE source of truth for which `<sui-*>`
// tags ship in `sui-marketing.js`. The deck bundler points its `sui-marketing`
// target at this module (TargetEntry::Real) and rolldown TRACES the import
// graph below: the set of side-effect imports here IS the bundle's tag set.
//
// To add or remove a marketing tag, edit THIS file — nothing in Rust. Each
// import pulls in one component's `.compiled.js` sibling (the deck SuiPlugin
// compiles the adjacent `.sui` on load); the runtime singletons
// (`@ship/sui-runtime`, `@ship/sui-dom`) stay external per the bundler config.
//
// Import specific component modules, NEVER a package index/umbrella — importing
// `@ship/sui-primitives` would balloon the bundle with every primitive tag.
//
// `sui-icon` is registered explicitly: the bundler defines only the elements
// imported here, and a `<sui-icon>` nested inside another template (e.g. the
// theme-picker trigger) is not transitively pulled in. Without this import
// `<sui-icon>` stays an inert HTMLUnknownElement — no glyph fetch, no render.
//
// NOTE (#221 Slice 2): the retired `MARKETING_SOURCES` Rust const also listed
// a few components that were never committed (no `.sui` source anywhere in
// the repo), so the virtual-aggregate build errored on the missing paths
// and produced stale output; those tags were never actually defined. They
// are intentionally NOT traced here — a non-existent module cannot be a
// bundle member. If/when those components land, add their imports below.

// Primitives — packages/components/sui/primitives
import '../../../../packages/components/sui/primitives/src/sui-icon/sui-icon.compiled.js'
import '../../../../packages/components/sui/primitives/src/sui-badge/sui-badge.compiled.js'
import '../../../../packages/components/sui/primitives/src/sui-button/sui-button.compiled.js'
import '../../../../packages/components/sui/primitives/src/sui-separator/sui-separator.compiled.js'
import '../../../../packages/components/sui/primitives/src/sui-status-dot/sui-status-dot.compiled.js'
import '../../../../packages/components/sui/primitives/src/sui-table/sui-table.compiled.js'
import '../../../../packages/components/sui/primitives/src/sui-card/sui-card.compiled.js'
import '../../../../packages/components/sui/primitives/src/sui-diff/sui-diff.compiled.js'
import '../../../../packages/components/sui/primitives/src/sui-tabs/sui-tabs.compiled.js'
import '../../../../packages/components/sui/primitives/src/sui-tab-panel/sui-tab-panel.compiled.js'

// Charts — packages/components/sui/charts
import '../../../../packages/components/sui/charts/src/sui-bar-chart/sui-bar-chart.compiled.js'
import '../../../../packages/components/sui/charts/src/sui-sparkline/sui-sparkline.compiled.js'

// Shell-owned chrome — apps/shell/src/components (brand mark + theme picker).
import '../../../shell/src/components/sui-brand-mark/sui-brand-mark.compiled.js'
import '../../../shell/src/components/sui-theme-picker/sui-theme-picker.compiled.js'
