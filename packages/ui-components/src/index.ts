// @ship/ui-components — Ship chrome + domain `<sui-*>` components.
//
// Primitives (button, input, dialog, popover, tabs, …) moved to
// `@ship/primitives` (packages/components/primitives/). This pack now
// holds only the components that know about Ship's chrome (rail,
// nav-top, launcher, panel-dock, command-bar, devtools, view chrome,
// status-bar, theme-picker, …) or are tied to a feature (git-diff,
// pipeline-*, auth-dialog, …). Every chrome / domain component uses
// primitive `<sui-*>` tags in its templates — they resolve at runtime
// because we side-effect-import the primitives pack at the top of this
// file, which triggers every primitive `customElements.define()`.
//
// The daemon serves the pre-built bundle of this module; the shell
// loads it after `@ship/primitives` so every `<sui-*>` tag is defined
// globally before any view template instantiates one.

import '@ship/primitives'
import './sui-view-picker/sui-view-picker.compiled.js'
import './sui-theme-picker/sui-theme-picker.compiled.js'
// sui-rail opts into its own separate bundle via `<bundle separate />`; see sui-rail/.
import './sui-status-bar/sui-status-bar.compiled.js'
import './sui-edit-overlay/sui-edit-overlay.compiled.js'
import './sui-edit-mode-pill/sui-edit-mode-pill.compiled.js'
import './sui-annotation/sui-annotation-layer.compiled.js'
import './sui-annotation/sui-annotation-marker.compiled.js'
import './sui-annotation/sui-annotation-list.compiled.js'
import './sui-view-chrome/sui-view-chrome.compiled.js'
// sui-nav-top + sui-launcher opt into separate bundles via `<bundle separate />`.
import './sui-brand-mark/sui-brand-mark.compiled.js'
import './sui-graph-canvas/sui-graph-canvas.compiled.js'
import './sui-pipeline-dag/sui-pipeline-dag.compiled.js'
import './sui-pipeline-step/sui-pipeline-step.compiled.js'
import './sui-pipeline-run/sui-pipeline-run.compiled.js'
import './sui-pipeline-step-def/sui-pipeline-step-def.compiled.js'
import './sui-pipeline-definition/sui-pipeline-definition.compiled.js'
import './sui-log-stream/sui-log-stream.compiled.js'
import './sui-provider-blob/sui-provider-blob.compiled.js'
import './sui-auth-dialog/sui-auth-dialog.compiled.js'
import './sui-git-diff/sui-git-diff.compiled.js'
import './sui-git-diff/sui-git-diff-file.compiled.js'
import './sui-git-diff/sui-git-diff-hunk.compiled.js'
import './sui-git-diff/sui-git-diff-line.compiled.js'
// sui-panel-dock opts into its own separate bundle via `<bundle separate />`.
// sui-devtools-* opt into a shared separate bundle via `<bundle separate />`.
// Separate bundles — loaded on demand by hosts that need them. Each lives at
// packages/ui-components/src/<tag>/ with a top-of-file `<bundle separate />`
// directive on its `.sui` source:
//   - sui-terminal       — the terminal pane
//   - sui-command-bar    — command-bar + command-palette + command-chat
//   - sui-devtools       — devtools-source + -components + -lint + -schema + -signals
//   - sui-launcher       — global app launcher chrome
//   - sui-panel-dock     — bottom panel-dock chrome
//   - sui-nav-top        — top nav chrome

// Cross-surface registries — single sources of truth used by
// pickers (sui-component-picker, ship-ui-studio catalog, cmdk insert).
export { SLOT_COMPOSITIONS } from './picker-slot-compositions'
