// Themes list — smoke tests over the section's pure data layer.
//
// Studio is shell-owned (apps/shell/src/studio); run under the shell's
// vitest config:  cd apps/shell && bunx vitest run src/studio/themes
//
// The list view's swatch snapshotting needs real CSS custom properties
// (a browser), so these target the deterministic, side-effect-free
// registry + filter logic the list renders from.

import { describe, it, expect } from 'vitest'
import { THEME_PACKS, kindCounts, isThemePack, getPack, type ThemePack } from './theme-registry'

// Mirror the controller's card filter so the contract is locked here.
function filterPacks(query: string, kind: 'all' | 'light' | 'dark' | 'special'): ThemePack[] {
  const q = query.trim().toLowerCase()
  return THEME_PACKS.filter((p) => {
    if (kind !== 'all' && p.kind !== kind) return false
    if (!q) return true
    return (
      p.id.toLowerCase().includes(q) ||
      p.label.toLowerCase().includes(q) ||
      p.blurb.toLowerCase().includes(q)
    )
  })
}

describe('themes: registry', () => {
  it('enumerates the 7 built-in packs', () => {
    expect(THEME_PACKS).toHaveLength(7)
    expect(THEME_PACKS.map((p) => p.id)).toEqual([
      'ship-light', 'ship-dark', 'paper', 'halo', 'console', 'contrast', 'glass',
    ])
  })

  it('every pack carries kind, blurb, and an override count', () => {
    for (const p of THEME_PACKS) {
      expect(['light', 'dark', 'special']).toContain(p.kind)
      expect(p.blurb.length).toBeGreaterThan(0)
      expect(p.overrideCount).toBeGreaterThan(0)
    }
  })

  it('splits 3 light / 3 dark / 1 special', () => {
    expect(kindCounts()).toEqual({ all: 7, light: 3, dark: 3, special: 1 })
  })

  it('marks glass as the special pack', () => {
    expect(getPack('glass')?.kind).toBe('special')
  })
})

describe('themes: pack id guard', () => {
  it('accepts every known pack id', () => {
    for (const p of THEME_PACKS) expect(isThemePack(p.id)).toBe(true)
  })
  it('rejects unknown / empty ids', () => {
    expect(isThemePack('nope')).toBe(false)
    expect(isThemePack(undefined)).toBe(false)
  })
})

describe('themes: list filter', () => {
  it('shows all packs by default', () => {
    expect(filterPacks('', 'all')).toHaveLength(7)
  })
  it('filters by kind pill', () => {
    expect(filterPacks('', 'light').every((p) => p.kind === 'light')).toBe(true)
    expect(filterPacks('', 'dark')).toHaveLength(3)
    expect(filterPacks('', 'special').map((p) => p.id)).toEqual(['glass'])
  })
  it('matches the search query against id, label, and blurb', () => {
    expect(filterPacks('console', 'all').map((p) => p.id)).toEqual(['console'])
    expect(filterPacks('phosphor', 'all').map((p) => p.id)).toEqual(['console'])
    expect(filterPacks('zzz', 'all')).toHaveLength(0)
  })
})
