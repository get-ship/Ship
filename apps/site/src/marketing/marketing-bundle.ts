// Marketing bundle membership — which `<sail-*>` tags ship in `sail-marketing.js`.
// Deck traces this module (TargetEntry::Real); side-effect imports ARE the tag set.
// Runtime/dom stay external per bundler config.
//
// Import per-tag package subpaths (`@ship/sail-primitives/sail-diff`, …).
// SailPlugin maps `*.compiled.js` targets to sibling `.sail` and compiles in-process.
// Dissolved L1 (button, badge, card, theme-picker, …) is host HTML + sail-base /
// ThemePicker.astro — not CE tags.

import '@ship/sail-primitives/sail-diff'
import '@ship/sail-charts/sail-bar-chart'
import '@ship/sail-charts/sail-sparkline'
