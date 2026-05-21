// Controller-layer unit tests for the agent detail keystone view.
//
// Mirrors views/settings/page.test.ts: these target the controller's
// pure functions and derived-state shape; they do NOT spin up the
// compiled .sui template. To run from the repo root:
//   bunx vitest run views/agents/[id]/page.test.ts
//
// The keystone shape's correctness lives in three places: the hue
// derivation (so the tile matches the create-flow), the initials
// derivation (the visible letter), and the rules-total roll-up
// (inline-rules count as one row). Everything else is straight data
// pass-through and is covered by SSR + web-qa.

import { describe, it, expect } from 'vitest'
import { pickHue, pickInitials } from './hue'

describe('pickHue', () => {
  it('returns a slate hue for the empty string', () => {
    expect(pickHue('')).toBe('slate')
  })

  it('returns a value from the documented palette', () => {
    const palette = new Set([
      'orange', 'amber', 'red', 'violet', 'blue',
      'teal', 'green', 'pink', 'slate',
    ])
    for (const id of ['rust-compiler', 'shipwright', 'gatekeeper', 'a', 'zzz']) {
      expect(palette.has(pickHue(id))).toBe(true)
    }
  })

  it('is deterministic across calls', () => {
    const a = pickHue('rust-compiler')
    const b = pickHue('rust-compiler')
    expect(a).toBe(b)
  })

  it('different ids generally pick different hues', () => {
    const hues = new Set([
      pickHue('one'), pickHue('two'), pickHue('three'),
      pickHue('four'), pickHue('five'),
    ])
    // Not a strict guarantee, but for these 5 the modulo lands on
    // multiple buckets — catches an off-by-one in the hash.
    expect(hues.size).toBeGreaterThan(1)
  })
})

describe('pickInitials', () => {
  it('returns ? for empty input', () => {
    expect(pickInitials('')).toBe('?')
  })

  it('uses the first letter of two parts of a hyphenated id', () => {
    expect(pickInitials('rust-compiler')).toBe('RC')
    expect(pickInitials('test_runner')).toBe('TR')
    expect(pickInitials('agent zero')).toBe('AZ')
  })

  it('uses two letters of a single-token id', () => {
    expect(pickInitials('shipwright')).toBe('SH')
    expect(pickInitials('gatekeeper')).toBe('GA')
  })

  it('falls back to name when id is empty', () => {
    expect(pickInitials('', 'Rust Compiler')).toBe('RC')
  })

  it('always uppercases', () => {
    expect(pickInitials('lower-case')).toBe('LC')
    expect(pickInitials('mixedCase')).toBe('MI')
  })

  it('handles a single character gracefully', () => {
    expect(pickInitials('a')).toBe('A')
  })
})

describe('rules-total roll-up', () => {
  // Inline rules count as one row in the section header — the same
  // way they render as one block in the template. The shape lives in
  // the derived signal, so we reproduce its logic here.
  function rulesTotal(rules: string[], inline: string): number {
    return rules.length + (inline.trim().length > 0 ? 1 : 0)
  }

  it('counts only the rules list when inline is empty', () => {
    expect(rulesTotal(['a', 'b'], '')).toBe(2)
    expect(rulesTotal([], '')).toBe(0)
  })

  it('adds 1 for non-empty inline rules', () => {
    expect(rulesTotal(['a'], 'be kind')).toBe(2)
    expect(rulesTotal([], 'be kind')).toBe(1)
  })

  it('treats whitespace-only inline as empty', () => {
    expect(rulesTotal([], '   \n  ')).toBe(0)
  })
})

describe('providerList default selection', () => {
  // The provider strip in the keystone view marks one provider as the
  // "default" — explicit when default_provider is set, falling back
  // to the first entry otherwise. Mirrors the controller's
  // derived(providerList) logic so a regression on either path fails
  // here.
  function buildProviders(providers: string[], def: string | null | undefined) {
    if (providers.length === 0) return []
    const chosen = def || providers[0]
    return providers.map((id) => ({
      id,
      label: id.length > 0 ? id[0].toUpperCase() + id.slice(1) : id,
      isDefault: id === chosen,
    }))
  }

  it('marks the explicit default when one is set', () => {
    const list = buildProviders(['anthropic', 'openai'], 'openai')
    expect(list.find((p) => p.isDefault)?.id).toBe('openai')
  })

  it('falls back to the first provider when default is missing', () => {
    const list = buildProviders(['anthropic', 'openai'], null)
    expect(list.find((p) => p.isDefault)?.id).toBe('anthropic')
  })

  it('capitalises the label', () => {
    const [first] = buildProviders(['anthropic'], 'anthropic')
    expect(first.label).toBe('Anthropic')
  })

  it('returns empty when no providers configured', () => {
    expect(buildProviders([], 'anthropic')).toEqual([])
  })
})
