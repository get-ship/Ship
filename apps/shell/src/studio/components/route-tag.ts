// Reactive "which component tag is the URL pointing at" signal for the
// Components section.
//
// Why not `route().params.sub()`? The studio surface is a same-shell view
// whose section bundle (sui-studio.js) links the *shared* `/_sui/runtime.js`
// runtime, while the host shell bundles its own runtime copy. Cross-bundle
// signal subscription doesn't fire reliably across that boundary, so a
// `derived` reading the shell-seeded route param can miss updates (and the
// param is never seeded at all on a cold deep-link). The retired
// ship-ui-studio hit the same wall and read `location.pathname` directly.
//
// We make that read reactive by owning a signal *in this bundle's runtime*
// and bumping it on the navigation events the shell already dispatches:
// `popstate` (synthetic, fired by the shell's same-shell route kick) and
// `sui:navigate` (emitted by `chrome.navigate`). Both list→detail clicks
// and the detail's back button flow through these, so the section root's
// list/detail switch and the detail's per-tag effects stay in sync.

import { signal } from '@ship/sui-runtime'

function readTag(): string {
  if (typeof location === 'undefined') return ''
  const segs = location.pathname.split('/').filter(Boolean)
  const i = segs.indexOf('components')
  return i >= 0 ? (segs[i + 1] ?? '') : ''
}

const _tag = signal<string>(readTag())
let _wired = false

function ensureWired(): void {
  if (_wired || typeof window === 'undefined') return
  _wired = true
  const update = (): void => _tag.set(readTag())
  window.addEventListener('popstate', update)
  document.addEventListener('sui:navigate', update)
}

/** Reactive read of the `<tag>` segment of `/studio/components/<tag>`,
 *  or `''` on the list route. Wires navigation listeners on first call. */
export function currentComponentTag(): string {
  ensureWired()
  return _tag()
}
