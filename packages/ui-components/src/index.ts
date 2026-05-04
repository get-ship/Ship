// @ship/ui-components — SUI primitive registry.
//
// Each component lives under `src/<tag>/<tag>.sui` (template + `<prop>`
// directives + scoped `<style>`), optionally paired with `<tag>.ts`
// (controller(def)). The compiler emits `<tag>.compiled.js` next to the
// source — importing it triggers `customElements.define()`.
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
import './sui-button/sui-button.compiled.js'
import './sui-tabs/sui-tabs.compiled.js'
import './sui-tab/sui-tab.compiled.js'
import './sui-empty-state/sui-empty-state.compiled.js'
import './sui-input/sui-input.compiled.js'
import './sui-icon/sui-icon.compiled.js'
import './sui-separator/sui-separator.compiled.js'
import './sui-label/sui-label.compiled.js'
import './sui-alert/sui-alert.compiled.js'
import './sui-tooltip/sui-tooltip.compiled.js'
import './sui-collapsible/sui-collapsible.compiled.js'
import './sui-checkbox/sui-checkbox.compiled.js'
import './sui-switch/sui-switch.compiled.js'
import './sui-textarea/sui-textarea.compiled.js'
import './sui-field/sui-field.compiled.js'
import './sui-toast/sui-toast.compiled.js'
import './sui-toast/sui-toast-container.compiled.js'
import './sui-breadcrumb/sui-breadcrumb.compiled.js'
import './sui-breadcrumb/sui-breadcrumb-item.compiled.js'
import './sui-pagination/sui-pagination.compiled.js'
import './sui-timestamp/sui-timestamp.compiled.js'
import './sui-metachip/sui-metachip.compiled.js'
import './sui-popover/sui-popover.compiled.js'
import './sui-theme-picker/sui-theme-picker.compiled.js'
import './sui-dialog/sui-dialog.compiled.js'
import './sui-drawer/sui-drawer.compiled.js'
import './sui-dropdown-menu/sui-dropdown-menu.compiled.js'
import './sui-context-menu/sui-context-menu.compiled.js'
import './sui-hover-card/sui-hover-card.compiled.js'
// sui-terminal is an island bundle — loaded only by views that use it.
// See packages/ui-components/src/islands/sui-terminal/.
