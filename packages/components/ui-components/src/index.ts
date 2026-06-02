// @ship/ui-components — Ship domain `<sui-*>` components.
//
// Chrome (rail, nav-top, launcher, panel-dock, command-bar, devtools,
// view chrome, status-bar, theme-picker, brand-mark, annotations,
// component-picker, view-authoring, edit-overlay, edit-mode-pill,
// view-picker) was relocated to `apps/shell/src/components/`. The
// code-editor + markdown-editor + terminal + git-diff tree moved to
// dedicated `ship-*` packs (`@ship/code`, `@ship/markdown`). This
// package now holds the feature/domain components only: pipeline-*,
// log-stream, harness-blob, auth-dialog, graph-canvas. Generic
// primitives live in `@ship/sui-primitives`
// (packages/components/sui/primitives/).
//
// Every domain component uses primitive `<sui-*>` tags in its
// templates — they resolve at runtime because we side-effect-import
// the primitives pack at the top of this file, which triggers every
// primitive `customElements.define()`.

import '@ship/sui-primitives'
import './sui-graph-canvas/sui-graph-canvas.compiled.js'
// sui-pipeline-* live in the dedicated @ship/pipelines pack (loaded via
// '@ship/pipelines/bundle'); the umbrella must NOT also define them or they
// register twice. The copies under src/ were diverged duplicates (#151).
import './sui-log-stream/sui-log-stream.compiled.js'
import './sui-harness-blob/sui-harness-blob.compiled.js'
import './sui-auth-dialog/sui-auth-dialog.compiled.js'
