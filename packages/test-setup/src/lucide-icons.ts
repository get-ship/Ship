// Lucide icon map loader for vitest setups. Reads the committed
// `crates/deck/deck-core/assets/lucide-icons.json` (the SAME embed the daemon
// serves at `/_sui/icons/{name}.svg`), wraps each fragment in the canonical
// `<svg>` envelope, and returns a `Map<name, full-svg-markup>` for the
// caller to push into `@ship/sui-primitives`'s per-icon cache.
//
// We do NOT depend on `@ship/sui-primitives` here — keeps test-setup free of
// component-package cycles. Callers pass `primeIconCache` in or iterate
// the map themselves.

import { readFileSync } from 'node:fs'

/** Aliases mirrored from `deck-core/src/assets.rs::ICON_ALIASES`. The daemon
 *  resolves these server-side; tests prime the cache identically so a
 *  `<sui-icon name="git">` test renders the same SVG it would in production. */
const ICON_ALIASES = [
  ['git', 'git-branch'],
  ['refresh', 'refresh-cw'],
  ['stop', 'stop-circle'],
  ['warning', 'triangle-alert'],
] as const

/** Wrap a lucide inner-svg fragment in the canonical envelope — kept in
 *  lockstep with `deck-core/src/assets.rs::serve_sui_icon`. */
function wrap(inner: string): string {
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" `
    + `stroke="currentColor" stroke-width="2" stroke-linecap="round" `
    + `stroke-linejoin="round">${inner}</svg>`
}

/** Read the committed lucide JSON + return a map of full-svg-markup per
 *  icon name (including alias names that point at their resolved svg). */
export function loadLucideIconMap(jsonPath: string): Map<string, string> {
  const icons: Record<string, string> = JSON.parse(readFileSync(jsonPath, 'utf8'))
  const out = new Map<string, string>()
  for (const [name, inner] of Object.entries(icons)) {
    out.set(name, wrap(inner))
  }
  for (const [alias, real] of ICON_ALIASES) {
    if (icons[real]) out.set(alias, wrap(icons[real]))
  }
  return out
}

/** Convenience: read the JSON + push every entry through `primer`. */
export function primeLucideIcons(
  jsonPath: string,
  primer: (name: string, svg: string) => void,
): void {
  for (const [name, svg] of loadLucideIconMap(jsonPath)) primer(name, svg)
}
