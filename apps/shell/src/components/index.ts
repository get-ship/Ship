// First-paint chrome registration.
//
// These `<sui-*>` chrome elements live in the shell's static `index.html`,
// which is NOT a compiled `.sui` the per-tag resolver scans — so nothing in the
// app graph references them, and without this seed they silently never register
// (an unresolved tag is a no-op, so the shell renders a blank skeleton). Each
// `@sui-tag/<tag>` below is resolved by the SAME per-tag resolver the view build
// uses: it compiles the source `.sui` and registers the custom element. Chrome
// stamped by a view's own `page.sui` (rail, desktop, …) is pulled in by that
// view's tags when it mounts and does not belong here.
//
// Keep this list to the tags present in `apps/shell/index.html` at first paint.
import '@sui-tag/sui-nav-top'
import '@sui-tag/sui-command-bar'
import '@sui-tag/sui-launcher'
import '@sui-tag/sui-panel-dock'
import '@sui-tag/sui-shell-bottom'
import '@sui-tag/sui-edit-overlay'
import '@sui-tag/sui-blank-canvas'
// Stamped into index.html at first paint (`<sui-toast-container>`), and mounted
// imperatively by `initHelpOverlay()` (`<sui-help-overlay>`) — neither is reached
// by any view's tag graph, so register both here or they stay inert no-ops.
import '@sui-tag/sui-toast-container'
import '@sui-tag/sui-help-overlay'
