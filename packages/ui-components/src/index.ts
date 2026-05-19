// @ship/ui-components — Ship domain `<sui-*>` components.
//
// Chrome (rail, nav-top, launcher, panel-dock, command-bar, devtools,
// view chrome, status-bar, theme-picker, brand-mark, annotations,
// component-picker, view-authoring, edit-overlay, edit-mode-pill,
// view-picker, code-editor, markdown-editor, terminal) was relocated
// to `apps/shell/src/components/`. This package now holds the
// feature/domain components only: git-diff, pipeline-*, log-stream,
// harness-blob, auth-dialog, graph-canvas. Generic primitives live
// in `@ship/primitives` (packages/components/primitives/).
//
// Every domain component uses primitive `<sui-*>` tags in its
// templates — they resolve at runtime because we side-effect-import
// the primitives pack at the top of this file, which triggers every
// primitive `customElements.define()`.

import '@ship/primitives'
import './sui-graph-canvas/sui-graph-canvas.compiled.js'
import './sui-pipeline-dag/sui-pipeline-dag.compiled.js'
import './sui-pipeline-step/sui-pipeline-step.compiled.js'
import './sui-pipeline-run/sui-pipeline-run.compiled.js'
import './sui-pipeline-step-def/sui-pipeline-step-def.compiled.js'
import './sui-pipeline-definition/sui-pipeline-definition.compiled.js'
import './sui-log-stream/sui-log-stream.compiled.js'
import './sui-harness-blob/sui-harness-blob.compiled.js'
import './sui-auth-dialog/sui-auth-dialog.compiled.js'
import './sui-git-diff/sui-git-diff.compiled.js'
import './sui-git-diff/sui-git-diff-file.compiled.js'
import './sui-git-diff/sui-git-diff-hunk.compiled.js'
import './sui-git-diff/sui-git-diff-line.compiled.js'
import './sui-isolation-badge/sui-isolation-badge.compiled.js'
