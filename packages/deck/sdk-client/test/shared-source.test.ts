import { beforeEach, describe, expect, it } from 'vitest'
import { live, signal, type LiveSignal, type LiveSource } from '@ship/sui-runtime'
import { _resetSharedSourceCache, sharedSource } from '../src/shared-source'

// Synthetic LiveSource that records subscribe/unsubscribe calls and exposes
// a manual `emit` for driving the upstream.
function fakeSource<T>() {
  let subs = 0
  let captured: LiveSignal<T> | null = null
  const src: LiveSource<T> = {
    subscribe(sig) {
      subs++
      captured = sig
      return () => {
        subs--
        captured = null
      }
    },
  }
  return {
    src,
    emit: (state: { kind: 'ok'; value: T } | { kind: 'error'; error: Error }) =>
      captured?.set(state),
    subCount: () => subs,
  }
}

describe('sharedSource (#198 dedup)', () => {
  beforeEach(() => _resetSharedSourceCache())

  it('multiple subscribers share one upstream subscription', () => {
    const f = fakeSource<number>()
    const shared = sharedSource('k', () => f.src)
    const sig1 = signal<{ kind: string }>({ kind: 'loading' }) as LiveSignal<number>
    const sig2 = signal<{ kind: string }>({ kind: 'loading' }) as LiveSignal<number>
    shared.subscribe(sig1)
    shared.subscribe(sig2)
    expect(f.subCount()).toBe(1)
  })

  it('upstream values fan out to all subscribers', () => {
    const f = fakeSource<number>()
    const shared = sharedSource('k', () => f.src)
    const sig1 = signal({ kind: 'loading' }) as LiveSignal<number>
    const sig2 = signal({ kind: 'loading' }) as LiveSignal<number>
    shared.subscribe(sig1)
    shared.subscribe(sig2)
    f.emit({ kind: 'ok', value: 42 })
    const v1 = sig1()
    const v2 = sig2()
    expect(v1.kind).toBe('ok')
    expect(v2.kind).toBe('ok')
    if (v1.kind === 'ok' && v2.kind === 'ok') {
      expect(v1.value).toBe(42)
      expect(v2.value).toBe(42)
    }
  })

  it('late subscribers receive the last emitted value immediately', () => {
    const f = fakeSource<number>()
    const shared = sharedSource('k', () => f.src)
    const sig1 = signal({ kind: 'loading' }) as LiveSignal<number>
    shared.subscribe(sig1)
    f.emit({ kind: 'ok', value: 7 })
    const sig2 = signal({ kind: 'loading' }) as LiveSignal<number>
    shared.subscribe(sig2)
    const v2 = sig2()
    expect(v2.kind).toBe('ok')
    if (v2.kind === 'ok') expect(v2.value).toBe(7)
    expect(f.subCount()).toBe(1)
  })

  it('upstream tears down only when the last subscriber disposes', () => {
    const f = fakeSource<number>()
    const shared = sharedSource('k', () => f.src)
    const sig1 = signal({ kind: 'loading' }) as LiveSignal<number>
    const sig2 = signal({ kind: 'loading' }) as LiveSignal<number>
    const dispose1 = shared.subscribe(sig1)
    const dispose2 = shared.subscribe(sig2)
    expect(f.subCount()).toBe(1)
    dispose1()
    expect(f.subCount()).toBe(1) // sig2 still subscribed
    dispose2()
    expect(f.subCount()).toBe(0) // upstream closed
  })

  it('different keys do not share', () => {
    const f1 = fakeSource<number>()
    const f2 = fakeSource<number>()
    const a = sharedSource('a', () => f1.src)
    const b = sharedSource('b', () => f2.src)
    const sig1 = signal({ kind: 'loading' }) as LiveSignal<number>
    const sig2 = signal({ kind: 'loading' }) as LiveSignal<number>
    a.subscribe(sig1)
    b.subscribe(sig2)
    expect(f1.subCount()).toBe(1)
    expect(f2.subCount()).toBe(1)
  })

  it('subscribing again to a fully-disposed key opens a fresh upstream', () => {
    const f = fakeSource<number>()
    const shared = sharedSource('k', () => f.src)
    const sig1 = signal({ kind: 'loading' }) as LiveSignal<number>
    const dispose1 = shared.subscribe(sig1)
    dispose1()
    expect(f.subCount()).toBe(0)
    const sig2 = signal({ kind: 'loading' }) as LiveSignal<number>
    shared.subscribe(sig2)
    expect(f.subCount()).toBe(1)
  })

  it('integrates with live() — two live() calls reuse one upstream', () => {
    const f = fakeSource<{ items: string[] }>()
    const shared = sharedSource('k', () => f.src)
    live(() => shared)
    live(() => shared)
    expect(f.subCount()).toBe(1)
  })
})
