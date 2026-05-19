// Probe-based theme token snapshotting.
//
// SUI components render in shadow DOM, so the global `[data-theme-preview]`
// rules in themes.css don't cascade into the detail editor's preview frame.
// To preview a theme faithfully we resolve its tokens against a hidden
// light-DOM probe (where those global rules DO apply) and then apply the
// snapshot as inline custom properties on the preview frame — inherited
// custom properties cross shadow boundaries into the nested `<sui-*>`
// elements, so the preview reflects the theme exactly.
//
// The set of token names is read once from the live CSSOM (every custom
// property declared under a `:root` or `[data-theme*]` rule), so it stays
// in sync with tokens.css/themes.css without a hand-maintained list.

import { isDarkTheme, type ThemeId } from '../../../../../packages/components/ui-components/src/tokens/themes'

export type TokenMap = Record<string, string>

let cachedNames: string[] | null = null

/** Every theme-relevant custom-property name, harvested from the CSSOM. */
export function themeTokenNames(): string[] {
  if (cachedNames) return cachedNames
  const names = new Set<string>()
  if (typeof document === 'undefined') return []
  for (const sheet of Array.from(document.styleSheets)) {
    let rules: CSSRuleList
    try {
      rules = sheet.cssRules
    } catch {
      continue // cross-origin sheet — not readable, skip.
    }
    for (const rule of Array.from(rules)) {
      if (!(rule instanceof CSSStyleRule)) continue
      const sel = rule.selectorText
      if (!sel || !(sel.includes(':root') || sel.includes('data-theme'))) continue
      const style = rule.style
      for (let i = 0; i < style.length; i += 1) {
        const prop = style.item(i)
        if (prop.startsWith('--')) names.add(prop)
      }
    }
  }
  cachedNames = Array.from(names)
  return cachedNames
}

/**
 * Resolve `tokens` (defaults to the full CSSOM set) against a hidden probe
 * carrying the given theme. The probe inherits the canonical base from
 * `:root`, so any token the theme doesn't override resolves to the base —
 * exactly the theme's effective value.
 */
export function snapshotTheme(themeId: ThemeId, tokens?: ReadonlyArray<string>): TokenMap {
  if (typeof document === 'undefined') return {}
  const names = tokens ?? themeTokenNames()
  const probe = document.createElement('div')
  probe.setAttribute('data-theme-preview', themeId)
  if (isDarkTheme(themeId)) probe.classList.add('dark')
  probe.style.cssText = 'position:absolute;left:-9999px;top:-9999px;visibility:hidden;pointer-events:none;'
  document.body.appendChild(probe)
  const cs = getComputedStyle(probe)
  const out: TokenMap = {}
  for (const name of names) out[name] = cs.getPropertyValue(name).trim()
  probe.remove()
  return out
}

/** Apply a token map as inline custom properties on `el`. */
export function applyTokens(el: HTMLElement, tokens: TokenMap): void {
  for (const [name, value] of Object.entries(tokens)) {
    if (value) el.style.setProperty(name, value)
  }
}

/**
 * Tokens whose effective value (theme base + user overrides) deviates from
 * the ship-light baseline — the source of truth for "Ship defaults". Used
 * for the per-row override dots and the live override count.
 */
export function overriddenTokens(
  effective: TokenMap,
  baseline: TokenMap,
  names: ReadonlyArray<string>,
): Set<string> {
  const out = new Set<string>()
  for (const name of names) {
    const a = (effective[name] ?? '').trim()
    const b = (baseline[name] ?? '').trim()
    if (a && a !== b) out.add(name)
  }
  return out
}
