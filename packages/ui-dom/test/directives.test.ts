import { describe, expect, it } from 'vitest'
import { mountState } from '../src/directives/state'
import { mountData } from '../src/directives/data'
import type { Signal } from '@ship/ui-runtime'

describe('mountState', () => {
  it('wraps the compiled value expression in a signal', () => {
    const sig = mountState(
      { name: 'count', value: () => 5 },
      {},
    ) as Signal<number>
    expect(sig()).toBe(5)
    sig.set(7)
    expect(sig()).toBe(7)
  })

  it('reads other scope members in the value expression', () => {
    const sig = mountState(
      { name: 'next', value: (scope) => (scope.first as number) + 1 },
      { first: 10 },
    ) as Signal<number>
    expect(sig()).toBe(11)
  })
})

describe('mountData', () => {
  it('returns a plain value for non-live data', () => {
    const result = mountData(
      { name: 'thing', source: () => ({ a: 1 }), live: false },
      {},
      {},
    )
    expect(result).toEqual({ a: 1 })
  })

  it('returns a LiveSignal for live data, transitioning to ok', async () => {
    const ship = { thing: () => Promise.resolve({ count: 7 }) }
    const sig = mountData(
      { name: 'thing', source: (s: any) => s.thing(), live: true },
      ship,
      {},
    ) as Signal<{ kind: string; value?: { count: number } }>
    expect(sig().kind).toBe('loading')
    await new Promise((r) => setTimeout(r, 0))
    const v = sig()
    expect(v.kind).toBe('ok')
    expect(v.value).toEqual({ count: 7 })
  })
})
