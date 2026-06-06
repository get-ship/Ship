// `sui-chrome` aggregate — non-separate chrome `<sui-*>` components for
// the shell. Loaded eagerly via `/_sui/sui-chrome.js` so the surfaces
// painted on first paint (rail, nav-top, dock, status-bar, brand-mark,
// theme/view pickers, view-chrome, annotations, edit-overlay/pill) are
// registered before mount runs.
//
// Primitives register globally via the shell's `@ship/sui-primitives/bundle`
// import; chrome components instantiate primitive `<sui-*>` tags freely
// without explicit per-component imports.

import './sui-view-picker/sui-view-picker.compiled.js'
import './sui-theme-picker/sui-theme-picker.compiled.js'
import './sui-status-bar/sui-status-bar.compiled.js'
import './sui-edit-overlay/sui-edit-overlay.compiled.js'
import './sui-edit-mode-pill/sui-edit-mode-pill.compiled.js'
import './sui-annotation/sui-annotation-layer.compiled.js'
import './sui-annotation/sui-annotation-marker.compiled.js'
import './sui-annotation/sui-annotation-list.compiled.js'
import './sui-view-chrome/sui-view-chrome.compiled.js'
import './sui-brand-mark/sui-brand-mark.compiled.js'
import './sui-shell-bottom/sui-shell-bottom.compiled.js'
import './sui-blank-canvas/sui-blank-canvas.compiled.js'
// Chrome surfaces (home, activity, account, library, setup, settings,
// canvas) are now fs-routed pages at apps/shell/src/<name>/page.sui,
// mounted by the shell via the same runtime.mount() path as daemon
// views. They are NOT custom elements anymore — no .compiled.js,
// no registration, no per-chrome .sui in components/. The studio
// rail below stays as a custom element because the studio deck
// drops <sui-studio-rail> as a tag inside its page.sui files.
import './sui-help-overlay/sui-help-overlay.compiled.js'
// Studio rail — reused by every studio deck route (views/studio/**). Eager
// so on-demand-compiled nested leaves can drop the tag without importing it.
import './sui-studio-rail/sui-studio-rail.compiled.js'

// Single source of truth for slot compositions used by the component
// picker. Exported so future shell-internal pickers (cmd-bar insert,
// etc.) can share it without forking.
export { SLOT_COMPOSITIONS } from './picker-slot-compositions'
