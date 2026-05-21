import { controller, signal, derived, chrome } from '@ship/sui-runtime'
import type { Scope } from './sui-themes-list.sui'
import { THEME_PACKS, kindCounts, THEMES, type ThemeKind } from '../theme-registry'
import { snapshotTheme } from '../theme-snapshot'

// The four key tokens each card's 2×2 swatch shows (spec: background,
// foreground, primary, accent).
const SWATCH_TOKENS = ['--background', '--foreground', '--primary', '--accent']

export interface ThemeCard {
  id: string
  label: string
  kind: ThemeKind
  blurb: string
  meta: string
  bg: string
  fg: string
  pr: string
  ac: string
}

// Resolve each pack's swatch colors. Uses a hidden probe at mount; falls
// back to the shared catalog's curated swatches when DOM isn't available
// (SSR first paint), so cards are never blank.
function buildSwatches(): Record<string, string[]> {
  const out: Record<string, string[]> = {}
  for (const t of THEMES) {
    if (typeof document !== 'undefined') {
      const snap = snapshotTheme(t.id, SWATCH_TOKENS)
      out[t.id] = SWATCH_TOKENS.map((n) => snap[n] || '')
    }
    // Fallback: catalog swatches are [background, card, primary, accent].
    const sw = t.swatches
    out[t.id] = (out[t.id] && out[t.id].every(Boolean))
      ? out[t.id]
      : [sw[0], sw[1], sw[2], sw[3]]
  }
  return out
}

export default controller<Scope>({
  scope: () => {
    const query = signal('')
    const activeKind = signal<'all' | ThemeKind>('all')
    const swatches = buildSwatches()
    return { query, activeKind, swatches }
  },

  derived: {
    counts: () => kindCounts(),
    cards: ({ query, activeKind, swatches }): ThemeCard[] => {
      const q = query().trim().toLowerCase()
      const kind = activeKind()
      return THEME_PACKS.filter((p) => {
        if (kind !== 'all' && p.kind !== kind) return false
        if (!q) return true
        return (
          p.id.toLowerCase().includes(q) ||
          p.label.toLowerCase().includes(q) ||
          p.blurb.toLowerCase().includes(q)
        )
      }).map((p) => {
        const sw = swatches[p.id] ?? ['', '', '', '']
        const count = `${p.overrideCount} token overrides`
        return {
          id: p.id,
          label: p.id,
          kind: p.kind,
          blurb: p.blurb,
          meta: p.metaSuffix ? `${count} · ${p.metaSuffix}` : count,
          bg: sw[0] ?? '', fg: sw[1] ?? '', pr: sw[2] ?? '', ac: sw[3] ?? '',
        }
      })
    },
  },

  on: {
    open: (_s, id: string) => {
      chrome.navigate(`/studio/themes/${id}`)
    },
    setKind: (state, kind: 'all' | ThemeKind) => {
      state.activeKind = kind
    },
    onSearch: (state, e: Event) => {
      state.query = (e.target as HTMLInputElement).value
    },
    newTheme: () => {
      if (typeof window === 'undefined') return
      window.dispatchEvent(new CustomEvent('sui-studio-new', { detail: { section: 'themes' } }))
    },
  },
})
