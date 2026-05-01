import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { signal, derived, effect } from '../src/signal'
import { state, debounce } from '../src/state'

describe('state', () => {
  it('is the same engine as signal', () => {
    const s = state(0)
    expect(typeof s).toBe('function')
    expect(s()).toBe(0)
    s.set(1)
    expect(s()).toBe(1)
  })

  it('participates in reactive scopes', () => {
    const s = state('a')
    const seen: string[] = []
    effect(() => {
      seen.push(s())
    })
    s.set('b')
    expect(seen).toEqual(['a', 'b'])
  })
})

describe('debounce', () => {
  beforeEach(() => {
    vi.useFakeTimers()
  })
  afterEach(() => {
    vi.useRealTimers()
  })

  it('initial value matches input', () => {
    const q = signal('hello')
    const d = debounce(q, 100)
    expect(d()).toBe('hello')
  })

  it('output remains old value until ms elapses', () => {
    const q = signal('a')
    const d = debounce(q, 100)
    q.set('b')
    expect(d()).toBe('a')
    vi.advanceTimersByTime(99)
    expect(d()).toBe('a')
    vi.advanceTimersByTime(1)
    expect(d()).toBe('b')
  })

  it('rapid writes only produce one output update', () => {
    const q = signal('')
    const d = debounce(q, 100)
    const seen: string[] = []
    effect(() => {
      seen.push(d())
    })
    q.set('h')
    vi.advanceTimersByTime(50)
    q.set('he')
    vi.advanceTimersByTime(50)
    q.set('hel')
    vi.advanceTimersByTime(99)
    expect(seen).toEqual([''])
    vi.advanceTimersByTime(1)
    expect(seen).toEqual(['', 'hel'])
  })

  it('output is reactive — derived can subscribe', () => {
    const q = signal(1)
    const d = debounce(q, 50)
    const doubled = derived(() => d() * 2)
    expect(doubled()).toBe(2)
    q.set(5)
    vi.advanceTimersByTime(50)
    expect(doubled()).toBe(10)
  })
})
