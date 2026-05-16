// @ship/ui-components — SUI primitive registry.
//
// Each component lives under `src/<tag>/<tag>.sui` (template + `<prop>`
// directives + scoped `<style>`), optionally paired with `<tag>.ts`
// (controller(def)). The compiler emits `<tag>.compiled.js` into the
// project mirror tree at `<repo>/.ship/sui-out/<src-rel>/` — importing
// the source-sibling spec below routes through a build-time resolver
// that redirects to the mirror artifact, which then triggers
// `customElements.define()`.
//
// The daemon serves the pre-built bundle of this module; the shell
// loads it before any view, so every `<sui-*>` tag is defined globally
// before any view template instantiates one.

import './sui-status-dot/sui-status-dot.compiled.js'
import './sui-spinner/sui-spinner.compiled.js'
import './sui-badge/sui-badge.compiled.js'
import './sui-chip/sui-chip.compiled.js'
import './sui-card/sui-card.compiled.js'
import './sui-link/sui-link.compiled.js'
import './sui-divider/sui-divider.compiled.js'
import './sui-skeleton/sui-skeleton.compiled.js'
import './sui-stat/sui-stat.compiled.js'
import './sui-mark/sui-mark.compiled.js'
import './sui-kbd/sui-kbd.compiled.js'
import './sui-avatar/sui-avatar.compiled.js'
import './sui-progress/sui-progress.compiled.js'
import "./sui-progress/sui-progress-ring.compiled.js"
import "./sui-area-chart/sui-area-chart.compiled.js"
import "./sui-bar-chart/sui-bar-chart.compiled.js"
import "./sui-line-chart/sui-line-chart.compiled.js"
import "./sui-sparkline/sui-sparkline.compiled.js"
import "./sui-donut/sui-donut.compiled.js"
import "./sui-gauge/sui-gauge.compiled.js"
import "./sui-meter/sui-meter.compiled.js"
import "./sui-heatmap/sui-heatmap.compiled.js"
import './sui-button/sui-button.compiled.js'
import './sui-tabs/sui-tabs.compiled.js'
import './sui-tabs/sui-tab.compiled.js'
import './sui-tab-panel/sui-tab-panel.compiled.js'
import './sui-empty-state/sui-empty-state.compiled.js'
import './sui-input/sui-input.compiled.js'
import './sui-icon/sui-icon.compiled.js'
import "./sui-select/sui-select.compiled.js"
import './sui-separator/sui-separator.compiled.js'
import './sui-label/sui-label.compiled.js'
import './sui-alert/sui-alert.compiled.js'
import './sui-tooltip/sui-tooltip.compiled.js'
import './sui-collapsible/sui-collapsible.compiled.js'
import './sui-checkbox/sui-checkbox.compiled.js'
import './sui-switch/sui-switch.compiled.js'
import './sui-textarea/sui-textarea.compiled.js'
import './sui-field/sui-field.compiled.js'
import './sui-alert-dialog/sui-alert-dialog.compiled.js'
import './sui-file-upload/sui-file-upload.compiled.js'
import './sui-toast/sui-toast.compiled.js'
import './sui-toast/sui-toast-container.compiled.js'
import './sui-toast-host/sui-toast-host.compiled.js'
import './sui-breadcrumb/sui-breadcrumb.compiled.js'
import './sui-breadcrumb/sui-breadcrumb-item.compiled.js'
import './sui-pagination/sui-pagination.compiled.js'
import './sui-timestamp/sui-timestamp.compiled.js'
import './sui-metachip/sui-metachip.compiled.js'
import './sui-popover/sui-popover.compiled.js'
import './sui-theme-picker/sui-theme-picker.compiled.js'
// sui-rail is island-bundled; see islands/sui-rail/.
import './sui-status-bar/sui-status-bar.compiled.js'
import './sui-dialog/sui-dialog.compiled.js'
import './sui-drawer/sui-drawer.compiled.js'
import './sui-dropdown-menu/sui-dropdown-menu.compiled.js'
import './sui-context-menu/sui-context-menu.compiled.js'
import './sui-hover-card/sui-hover-card.compiled.js'
import './sui-accordion/sui-accordion.compiled.js'
import './sui-accordion/sui-accordion-item.compiled.js'
import './sui-list-tile/sui-list-tile.compiled.js'
import './sui-error-boundary/sui-error-boundary.compiled.js'
import './sui-portal/sui-portal.compiled.js'
import './sui-error-state/sui-error-state.compiled.js'
import './sui-stepper/sui-stepper.compiled.js'
import "./sui-step-segments/sui-step-segments.compiled.js"
import './sui-timeline/sui-timeline.compiled.js'
import './sui-timeline/sui-timeline-item.compiled.js'
import './sui-loading/sui-loading.compiled.js'
import './sui-edit-overlay/sui-edit-overlay.compiled.js'
import './sui-annotation/sui-annotation-layer.compiled.js'
import './sui-annotation/sui-annotation-marker.compiled.js'
import './sui-annotation/sui-annotation-list.compiled.js'
import './sui-radio-group/sui-radio.compiled.js'
import './sui-radio-group/sui-radio-group.compiled.js'
import './sui-slider/sui-slider.compiled.js'
import './sui-segmented/sui-segmented-option.compiled.js'
import './sui-segmented/sui-segmented.compiled.js'
import './sui-toggle-group/sui-toggle.compiled.js'
import './sui-toggle-group/sui-toggle-group.compiled.js'
import './sui-code/sui-code.compiled.js'
import './sui-copy-button/sui-copy-button.compiled.js'
import './sui-json-view/sui-json-view.compiled.js'
import './sui-diff/sui-diff.compiled.js'
import './sui-markdown/sui-markdown.compiled.js'
import "./sui-view-chrome/sui-view-chrome.compiled.js"
// sui-nav-top + sui-launcher are island-bundled; see islands/.
import "./sui-navigation-menu/sui-navigation-menu.compiled.js"
import "./sui-navigation-menu/sui-navigation-item.compiled.js"
import "./sui-scroll-area/sui-scroll-area.compiled.js"
import './sui-toolbar/sui-toolbar.compiled.js'
import './sui-app-icon/sui-app-icon.compiled.js'
import './sui-brand-mark/sui-brand-mark.compiled.js'
import './sui-tree-view/sui-tree-view.compiled.js'
import './sui-tree-view/sui-tree-item.compiled.js'
import './sui-table/sui-table.compiled.js'
import './sui-graph-canvas/sui-graph-canvas.compiled.js'
import './sui-pipeline-dag/sui-pipeline-dag.compiled.js'
import './sui-pipeline-step/sui-pipeline-step.compiled.js'
import './sui-pipeline-run/sui-pipeline-run.compiled.js'
import './sui-pipeline-step-def/sui-pipeline-step-def.compiled.js'
import './sui-pipeline-definition/sui-pipeline-definition.compiled.js'
import './sui-log-stream/sui-log-stream.compiled.js'
import './sui-provider-blob/sui-provider-blob.compiled.js'
import './sui-theme-toggle/sui-theme-toggle.compiled.js'
import './sui-history-row/sui-history-row.compiled.js'
import './sui-auth-dialog/sui-auth-dialog.compiled.js'
import './sui-git-diff/sui-git-diff.compiled.js'
import './sui-git-diff/sui-git-diff-file.compiled.js'
import './sui-git-diff/sui-git-diff-hunk.compiled.js'
import './sui-git-diff/sui-git-diff-line.compiled.js'
// sui-panel-dock is island-bundled; see islands/sui-panel-dock/.
// sui-devtools-* are island-bundled; see islands/sui-devtools/.
// Island bundles — loaded on demand by hosts that need them. See
// packages/ui-components/src/islands/<bundle>/:
//   - sui-terminal       — the terminal pane
//   - sui-command-bar    — command-bar + command-palette + command-chat
//   - sui-devtools       — devtools-source + -components + -lint + -schema + -signals
//   - sui-launcher       — global app launcher chrome
//   - sui-panel-dock     — bottom panel-dock chrome
//   - sui-nav-top        — top nav chrome
