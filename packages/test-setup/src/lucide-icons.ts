// Lucide icon map loader for vitest setups. Reads the committed
// `crates/deck/deck-core/assets/lucide-icons.json` (the SAME embed the daemon
// serves at `/_deck/icons/{name}.svg`), wraps each fragment in the canonical
// `<svg>` envelope, and returns a `Map<name, full-svg-markup>` for the
// caller to push into `@ship/sui-primitives`'s per-icon cache.
//
// We do NOT depend on `@ship/sui-primitives` here — keeps test-setup free of
// component-package cycles. Callers pass `primeIconCache` in or iterate
// the map themselves.

import { existsSync, readFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

/** Repo-relative location of the committed lucide registry. */
const LUCIDE_REL = 'crates/deck/deck-core/assets/lucide-icons.json'

/**
 * Resolve the committed `lucide-icons.json` independent of `process.cwd()`.
 *
 * Every vitest setup used to hand-roll a `resolve(process.cwd(), '../../..')`
 * hop whose depth depended on the package dir — so the same file ENOENT'd the
 * moment a runner was invoked from a different cwd. Walk up from THIS module's
 * own location (stable regardless of which package's tests are running) until
 * the registry is found. Throws a clear error if the repo layout is missing it.
 */
export function defaultLucideIconsPath(): string {
  let dir = dirname(fileURLToPath(import.meta.url))
  for (let i = 0; i < 16; i++) {
    const candidate = join(dir, LUCIDE_REL)
    if (existsSync(candidate)) return candidate
    const parent = dirname(dir)
    if (parent === dir) break
    dir = parent
  }
  throw new Error(
    `lucide-icons.json not found in any ancestor of ${fileURLToPath(import.meta.url)} (looked for ${LUCIDE_REL})`,
  )
}

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

/** Convenience: read the JSON + push every entry through `primer`. When no
 *  path is given, resolves the committed registry via [`defaultLucideIconsPath`]
 *  (cwd-independent) — the form every setup should use. */
export function primeLucideIcons(
  primer: (name: string, svg: string) => void,
  jsonPath: string = defaultLucideIconsPath(),
): void {
  for (const [name, svg] of loadLucideIconMap(jsonPath)) primer(name, svg)
}
