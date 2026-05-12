// @ship/ui-components — SUI primitive registry.
//
// Each component lives under `src/<tag>/<tag>.sui` (template + `<prop>`
// directives + scoped `<style>`), optionally paired with `<tag>.ts`
// (controller(def)). The compiler emits `<tag>.compiled.js` into a
// sibling `dist/` directory — importing it triggers `customElements.define()`.
//
// The daemon serves the pre-built bundle of this module; the shell
// loads it before any view, so every `<sui-*>` tag is defined globally
// before any view template instantiates one.

import './sui-status-dot/dist/sui-status-dot.compiled.js'
import './sui-spinner/dist/sui-spinner.compiled.js'
import './sui-badge/dist/sui-badge.compiled.js'
import './sui-chip/dist/sui-chip.compiled.js'
import './sui-card/dist/sui-card.compiled.js'
import './sui-link/dist/sui-link.compiled.js'
import './sui-divider/dist/sui-divider.compiled.js'
import './sui-skeleton/dist/sui-skeleton.compiled.js'
import './sui-stat/dist/sui-stat.compiled.js'
import './sui-mark/dist/sui-mark.compiled.js'
import './sui-kbd/dist/sui-kbd.compiled.js'
import './sui-avatar/dist/sui-avatar.compiled.js'
import './sui-progress/dist/sui-progress.compiled.js'
import "./sui-progress/dist/sui-progress-ring.compiled.js"
import "./sui-area-chart/dist/sui-area-chart.compiled.js"
import "./sui-bar-chart/dist/sui-bar-chart.compiled.js"
import "./sui-line-chart/dist/sui-line-chart.compiled.js"
import "./sui-sparkline/dist/sui-sparkline.compiled.js"
import './sui-button/dist/sui-button.compiled.js'
import './sui-tabs/dist/sui-tabs.compiled.js'
import './sui-tabs/dist/sui-tab.compiled.js'
import './sui-empty-state/dist/sui-empty-state.compiled.js'
import './sui-input/dist/sui-input.compiled.js'
import './sui-icon/dist/sui-icon.compiled.js'
import "./sui-select/dist/sui-select.compiled.js"
import './sui-separator/dist/sui-separator.compiled.js'
import './sui-label/dist/sui-label.compiled.js'
import './sui-alert/dist/sui-alert.compiled.js'
import './sui-tooltip/dist/sui-tooltip.compiled.js'
import './sui-collapsible/dist/sui-collapsible.compiled.js'
import './sui-checkbox/dist/sui-checkbox.compiled.js'
import './sui-switch/dist/sui-switch.compiled.js'
import './sui-textarea/dist/sui-textarea.compiled.js'
import './sui-field/dist/sui-field.compiled.js'
import './sui-alert-dialog/dist/sui-alert-dialog.compiled.js'
import './sui-file-upload/dist/sui-file-upload.compiled.js'
import './sui-toast/dist/sui-toast.compiled.js'
import './sui-toast/dist/sui-toast-container.compiled.js'
import './sui-breadcrumb/dist/sui-breadcrumb.compiled.js'
import './sui-breadcrumb/dist/sui-breadcrumb-item.compiled.js'
import './sui-pagination/dist/sui-pagination.compiled.js'
import './sui-timestamp/dist/sui-timestamp.compiled.js'
import './sui-metachip/dist/sui-metachip.compiled.js'
import './sui-popover/dist/sui-popover.compiled.js'
import './sui-theme-picker/dist/sui-theme-picker.compiled.js'
import './sui-rail/dist/sui-rail.compiled.js'
import './sui-rail/dist/sui-rail-item.compiled.js'
import './sui-rail/dist/sui-rail-section.compiled.js'
import './sui-status-bar/dist/sui-status-bar.compiled.js'
import './sui-dialog/dist/sui-dialog.compiled.js'
import './sui-drawer/dist/sui-drawer.compiled.js'
import './sui-dropdown-menu/dist/sui-dropdown-menu.compiled.js'
import './sui-context-menu/dist/sui-context-menu.compiled.js'
import './sui-hover-card/dist/sui-hover-card.compiled.js'
import './sui-accordion/dist/sui-accordion.compiled.js'
import './sui-accordion/dist/sui-accordion-item.compiled.js'
import './sui-list-tile/dist/sui-list-tile.compiled.js'
import './sui-error-state/dist/sui-error-state.compiled.js'
import './sui-stepper/dist/sui-stepper.compiled.js'
import "./sui-step-segments/dist/sui-step-segments.compiled.js"
import './sui-timeline/dist/sui-timeline.compiled.js'
import './sui-timeline/dist/sui-timeline-item.compiled.js'
import './sui-loading/dist/sui-loading.compiled.js'
import './sui-edit-overlay/dist/sui-edit-overlay.compiled.js'
import './sui-annotation/dist/sui-annotation-layer.compiled.js'
import './sui-annotation/dist/sui-annotation-marker.compiled.js'
import './sui-annotation/dist/sui-annotation-list.compiled.js'
import './sui-radio-group/dist/sui-radio.compiled.js'
import './sui-radio-group/dist/sui-radio-group.compiled.js'
import './sui-slider/dist/sui-slider.compiled.js'
import './sui-segmented/dist/sui-segmented-option.compiled.js'
import './sui-segmented/dist/sui-segmented.compiled.js'
import './sui-toggle-group/dist/sui-toggle.compiled.js'
import './sui-toggle-group/dist/sui-toggle-group.compiled.js'
import './sui-code/dist/sui-code.compiled.js'
import './sui-copy-button/dist/sui-copy-button.compiled.js'
import './sui-json-view/dist/sui-json-view.compiled.js'
import './sui-diff/dist/sui-diff.compiled.js'
import './sui-markdown/dist/sui-markdown.compiled.js'
import "./sui-view-chrome/dist/sui-view-chrome.compiled.js"
// sui-nav-top + sui-launcher are island-bundled; see islands/.
import "./sui-navigation-menu/dist/sui-navigation-menu.compiled.js"
import "./sui-navigation-menu/dist/sui-navigation-item.compiled.js"
import "./sui-scroll-area/dist/sui-scroll-area.compiled.js"
import './sui-toolbar/dist/sui-toolbar.compiled.js'
import './sui-app-icon/dist/sui-app-icon.compiled.js'
import './sui-tree-view/dist/sui-tree-view.compiled.js'
import './sui-tree-view/dist/sui-tree-item.compiled.js'
import './sui-table/dist/sui-table.compiled.js'
import './sui-graph-canvas/dist/sui-graph-canvas.compiled.js'
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
