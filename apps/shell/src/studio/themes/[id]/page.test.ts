// Theme detail — editor + preview behavior tests.
//
// Run under the shell's vitest config:
//   cd apps/shell && bunx vitest run src/studio/themes
//
// Live preview rendering needs a browser (resolved custom properties on a
// probe), so these target the editor's deterministic core: the token
// catalog, override detection (criterion 4), and import/export round-trips
// (criterion 5).

import { describe, it, expect } from 'vitest'
import { CATEGORIES, ALL_TOKENS, TOKEN_NAMES, TAB_IDS, tokenType } from '../token-catalog'
import { overriddenTokens } from '../theme-snapshot'
import { toCss, toJson, parseImport } from '../theme-io'

describe('themes/[id]: token catalog', () => {
  it('groups tokens into the six editor categories', () => {
    expect(CATEGORIES.map((c) => c.id)).toEqual([
      'foundation', 'tone', 'typography', 'motion', 'radii', 'effects',
    ])
  })
  it('exposes an "all" tab plus one per category', () => {
    expect(TAB_IDS).toEqual(['all', 'foundation', 'tone', 'typography', 'motion', 'radii', 'effects'])
  })
  it('every token name is a CSS custom property with a known type', () => {
    expect(ALL_TOKENS.length).toBe(TOKEN_NAMES.length)
    for (const name of TOKEN_NAMES) {
      expect(name.startsWith('--')).toBe(true)
      expect(tokenType(name)).toBeTruthy()
    }
  })
})

describe('themes/[id]: override detection', () => {
  const baseline = { '--primary': '#aaa', '--background': '#fff', '--radius-md': '8px' }

  it('flags tokens whose effective value deviates from the baseline', () => {
    const effective = { '--primary': '#f00', '--background': '#fff', '--radius-md': '8px' }
    const dotted = overriddenTokens(effective, baseline, Object.keys(baseline))
    expect([...dotted]).toEqual(['--primary'])
  })
  it('treats a value matching the baseline as not overridden', () => {
    const dotted = overriddenTokens({ ...baseline }, baseline, Object.keys(baseline))
    expect(dotted.size).toBe(0)
  })
  it('ignores empty effective values', () => {
    const dotted = overriddenTokens({ '--primary': '' }, baseline, ['--primary'])
    expect(dotted.size).toBe(0)
  })
})

describe('themes/[id]: import / export', () => {
  const values = { '--primary': 'oklch(0.6 0.2 30)', '--radius-md': '10px' }

  it('exports a parseable CSS block that round-trips', () => {
    const css = toCss('paper', values)
    expect(css).toContain('[data-theme-preview="paper"]')
    expect(parseImport(css)).toEqual(values)
  })
  it('exports JSON that round-trips through parseImport', () => {
    const json = toJson('paper', values)
    expect(JSON.parse(json)).toEqual({ pack: 'paper', values })
    expect(parseImport(json)).toEqual(values)
  })
  it('parses a bare JSON token map', () => {
    expect(parseImport('{"--primary":"#123456"}')).toEqual({ '--primary': '#123456' })
  })
  it('parses a loose CSS variables block', () => {
    expect(parseImport('--primary: #fff;\n--accent: #000;')).toEqual({
      '--primary': '#fff', '--accent': '#000',
    })
  })
  it('returns null when nothing parseable is found', () => {
    expect(parseImport('not css or json')).toBeNull()
    expect(parseImport('')).toBeNull()
  })
})
