// Themes-section registry — the 7 built-in token packs.
//
// Identity (id, label, dark, swatch palette) is sourced from the shared
// theme catalog (packages/.../tokens/themes.ts) so there's one source of
// truth for what a theme *is*. This module adds the editorial metadata
// the Studio surface shows — kind chip, blurb, and the override-count
// meta line on cards.
//
// Override counts are curated to match the design (themes-list.html).
// TODO(themes): derive counts from each theme's [data-theme-preview]
// block vs the ship-light baseline once a build-time token differ lands;
// see theme-snapshot.ts for the runtime override detection used by the
// detail editor's per-token dots.

import { THEMES, isValidTheme, type ThemeId } from '../../../../../packages/components/ui-components/src/tokens/themes'

// Re-export the shared catalog symbols so section components (one
// directory deeper) import them from here rather than recomputing the
// path to the tokens package.
export { THEMES, isValidTheme, type ThemeId }

export type ThemeKind = 'light' | 'dark' | 'special'

export interface ThemePack {
  id: ThemeId
  label: string
  kind: ThemeKind
  blurb: string
  /** Curated override count shown on the card + detail meta line. */
  overrideCount: number
  /** Optional trailing note on the meta line (e.g. "base pack"). */
  metaSuffix?: string
}

// Editorial copy, keyed by id. Order + content track themes-list.html.
const META: Record<ThemeId, Omit<ThemePack, 'id' | 'label'>> = {
  'ship-light': { kind: 'light', overrideCount: 12, metaSuffix: 'base pack',
    blurb: 'Default. Warm off-white surfaces, amber primary, deliberate motion.' },
  'ship-dark': { kind: 'dark', overrideCount: 14,
    blurb: 'Warm-navy + amber sibling to ship-light. Same primary, inverted surfaces.' },
  'paper': { kind: 'light', overrideCount: 19,
    blurb: 'Cream + terracotta. Heavier body type, slower motion. Reading-first.' },
  'halo': { kind: 'light', overrideCount: 22,
    blurb: 'Pale-blue neumorph. Depth comes from soft shadows, not borders.' },
  'console': { kind: 'dark', overrideCount: 28, metaSuffix: 'most divergent',
    blurb: 'Phosphor-green on near-black. Mono everywhere. 2px corners.' },
  'contrast': { kind: 'dark', overrideCount: 17, metaSuffix: 'WCAG AAA',
    blurb: 'Near-black + amber fused. 1–2px ring shadows, bold weights.' },
  'glass': { kind: 'special', overrideCount: 24, metaSuffix: 'requires backdrop-filter support',
    blurb: 'Frosted aero. backdrop-filter on every elevated surface.' },
}

export const THEME_PACKS: ReadonlyArray<ThemePack> = THEMES.map((t) => ({
  id: t.id,
  label: t.label,
  ...META[t.id],
}))

const PACK_BY_ID: Record<string, ThemePack> = Object.fromEntries(
  THEME_PACKS.map((p) => [p.id, p]),
)

export const isThemePack = (id: string | undefined): id is ThemeId =>
  typeof id === 'string' && id in PACK_BY_ID

export const getPack = (id: string | undefined): ThemePack | undefined =>
  id ? PACK_BY_ID[id] : undefined

/** Group counts for the list-view filter pills. */
export function kindCounts(): Record<'all' | ThemeKind, number> {
  const out = { all: THEME_PACKS.length, light: 0, dark: 0, special: 0 }
  for (const p of THEME_PACKS) out[p.kind] += 1
  return out
}
