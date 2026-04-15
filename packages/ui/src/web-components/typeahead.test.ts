import { describe, expect, it, vi } from 'vitest'
import { Typeahead, isTypeaheadKey } from './typeahead'

describe('Typeahead', () => {
  it('single-letter buffer cycles through matches starting with that letter', () => {
    const t = new Typeahead()
    const labels = ['Apple', 'Avocado', 'Banana', 'Apricot']
    t.push('a')
    expect(t.match(labels, -1)).toBe(0)
    t.push('a')
    expect(t.match(labels, 0)).toBe(1)
    t.push('a')
    expect(t.match(labels, 1)).toBe(3)
    t.push('a')
    expect(t.match(labels, 3)).toBe(0)
  })

  it('multi-char buffer prefix-matches without cycling', () => {
    const t = new Typeahead()
    const labels = ['Apple', 'Apricot', 'Avocado']
    t.push('a')
    t.push('p')
    t.push('r')
    expect(t.match(labels, 0)).toBe(1)
  })

  it('buffer resets after the timeout', () => {
    vi.useFakeTimers()
    vi.setSystemTime(0)
    const t = new Typeahead(100)
    t.push('a')
    t.push('p')
    expect(t.match(['Apple'], -1)).toBe(0)
    vi.setSystemTime(500)
    t.push('b')
    expect(t.match(['Apple', 'Banana'], -1)).toBe(1)
    vi.useRealTimers()
  })

  it('isTypeaheadKey ignores modifiers and non-printables', () => {
    expect(isTypeaheadKey(new KeyboardEvent('keydown', { key: 'a' }))).toBe(true)
    expect(isTypeaheadKey(new KeyboardEvent('keydown', { key: 'a', ctrlKey: true }))).toBe(false)
    expect(isTypeaheadKey(new KeyboardEvent('keydown', { key: 'Enter' }))).toBe(false)
    expect(isTypeaheadKey(new KeyboardEvent('keydown', { key: ' ' }))).toBe(false)
  })
})
