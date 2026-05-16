// picker-hidden-tags.ts — single source of truth for components that
// should NOT show in any user-facing picker.
//
// These are chrome / container tags that need their host or shared
// context to render (sui-rail without sui-panel-dock surroundings, a
// sui-toast without a sui-toast-host, sui-command-bar slotted by the
// shell, etc.). Trying to instantiate them standalone yields broken
// or invisible output.
//
// Used by:
//   - ship-ui-studio's catalog (filters the alphabetical browser)
//   - sui-component-picker (picker primitive used in cmdk + dev panel)
//   - any future "insert component" UX
//
// Add a tag here whenever a component should ONLY render inside
// chrome — never standalone.

export const PICKER_HIDDEN_TAGS: ReadonlySet<string> = new Set<string>([
  // Chrome surfaces
  'sui-command-bar',
  'sui-command-palette',
  'sui-command-chat',
  'sui-launcher',
  'sui-panel-dock',
  'sui-panel-dock-tab',
  'sui-nav-top',
  'sui-rail',
  'sui-rail-item',
  'sui-rail-section',
  'sui-status-bar',
  'sui-edit-mode-pill',
  'sui-edit-overlay',
  // Toast container + host: only meaningful when slotted into <sui-toast-container>
  'sui-toast',
  'sui-toast-container',
  'sui-toast-host',
  // Editor islands — large lazy bundles, render as bare textareas
  // without a host providing source / language / events
  'sui-terminal',
  'sui-code-editor',
  'sui-markdown-editor',
  // Devtools — only meaningful inside the dock
  'sui-devtools-source',
  'sui-devtools-signals',
  'sui-devtools-schema',
  'sui-devtools-components',
  // View-authoring slots — same: only meaningful inside dock VA tabs
  'sui-va-source',
  'sui-va-components',
  'sui-va-activity',
  'sui-va-compile',
  'sui-va-annotation-pins',
])

/** Return true if `tag` should be hidden from user-facing pickers. */
export function isPickerHidden(tag: string): boolean {
  return PICKER_HIDDEN_TAGS.has(tag)
}
