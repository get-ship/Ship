import { describe, expect, it, vi } from 'vitest'
import { signal, derived, effect, batch, untrack } from '../src/signal'

describe('signal', () => {
  it('reads return the current value', () => {
    const $count = signal(42)
    expect($count()).toBe(42)
  })

  it('writes update the value', () => {
    const $count = signal(0)
    $count.set(5)
    expect($count()).toBe(5)
  })

  it('write with function form gets previous value', () => {
    const $count = signal(10)
    $count.set((prev) => prev * 2)
    expect($count()).toBe(20)
  })

  it('Object.is equal writes do not notify', () => {
    const $count = signal(7)
    const fn = vi.fn()
    effect(() => {
      $count()
      fn()
    })
    expect(fn).toHaveBeenCalledTimes(1)
    $count.set(7)
    expect(fn).toHaveBeenCalledTimes(1)
  })

  it('writes notify subscribers', () => {
    const $count = signal(0)
    const fn = vi.fn()
    effect(() => {
      $count()
      fn()
    })
    expect(fn).toHaveBeenCalledTimes(1)
    $count.set(1)
    expect(fn).toHaveBeenCalledTimes(2)
    $count.set(2)
    expect(fn).toHaveBeenCalledTimes(3)
  })

  it('peek does not subscribe', () => {
    const $count = signal(0)
    const fn = vi.fn()
    effect(() => {
      $count.__peek()
      fn()
    })
    expect(fn).toHaveBeenCalledTimes(1)
    $count.set(99)
    expect(fn).toHaveBeenCalledTimes(1)
  })
})

describe('effect', () => {
  it('runs once eagerly', () => {
    const fn = vi.fn()
    effect(fn)
    expect(fn).toHaveBeenCalledTimes(1)
  })

  it('cleanup runs before next re-run', () => {
    const $x = signal(0)
    const cleanup = vi.fn()
    effect(() => {
      $x()
      return cleanup
    })
    expect(cleanup).toHaveBeenCalledTimes(0)
    $x.set(1)
    expect(cleanup).toHaveBeenCalledTimes(1)
    $x.set(2)
    expect(cleanup).toHaveBeenCalledTimes(2)
  })

  it('disposer stops re-runs', () => {
    const $x = signal(0)
    const fn = vi.fn()
    const dispose = effect(() => {
      $x()
      fn()
    })
    expect(fn).toHaveBeenCalledTimes(1)
    dispose()
    $x.set(1)
    expect(fn).toHaveBeenCalledTimes(1) // unchanged
  })

  it('disposer runs final cleanup', () => {
    const $x = signal(0)
    const cleanup = vi.fn()
    const dispose = effect(() => {
      $x()
      return cleanup
    })
    dispose()
    expect(cleanup).toHaveBeenCalledTimes(1)
  })
})

describe('derived', () => {
  it('memoizes its result', () => {
    const $x = signal(2)
    const compute = vi.fn(() => $x() * 3)
    const $tripled = derived(compute)
    expect($tripled()).toBe(6)
    expect($tripled()).toBe(6) // re-read
    expect(compute).toHaveBeenCalledTimes(1)
  })

  it('recomputes when a dep changes', () => {
    const $x = signal(2)
    const $tripled = derived(() => $x() * 3)
    expect($tripled()).toBe(6)
    $x.set(5)
    expect($tripled()).toBe(15)
  })

  it('downstream effect re-runs when derived changes', () => {
    const $x = signal(1)
    const $double = derived(() => $x() * 2)
    const fn = vi.fn()
    effect(() => {
      $double()
      fn()
    })
    expect(fn).toHaveBeenCalledTimes(1)
    $x.set(2)
    expect(fn).toHaveBeenCalledTimes(2)
  })

  it('throws on .set()', () => {
    const $x = signal(0)
    const $d = derived(() => $x() + 1)
    expect(() => $d.set(99)).toThrow(/derived/)
  })

  it('skips notify when recomputed value is Object.is-equal', () => {
    const $count = signal(0)
    const $positive = derived(() => $count() > 0)
    const fn = vi.fn()
    effect(() => {
      $positive()
      fn()
    })
    expect(fn).toHaveBeenCalledTimes(1)
    // 0 → 5 flips false → true; subscriber should fire.
    $count.set(5)
    expect(fn).toHaveBeenCalledTimes(2)
    // 5 → 10 stays true → true; subscriber should NOT fire.
    $count.set(10)
    expect(fn).toHaveBeenCalledTimes(2)
    $count.set(20)
    expect(fn).toHaveBeenCalledTimes(2)
    // 20 → -1 flips true → false; subscriber fires again.
    $count.set(-1)
    expect(fn).toHaveBeenCalledTimes(3)
  })

  it('preserves identity-equal object refs (immutable-ref contract)', () => {
    // Holding the same object ref across a "write" looks unchanged to
    // the derived — callers must produce a fresh ref to signal a change.
    const obj = { a: 1 }
    const $s = signal(obj)
    const $d = derived(() => $s())
    const fn = vi.fn()
    effect(() => {
      $d()
      fn()
    })
    expect(fn).toHaveBeenCalledTimes(1)
    $s.set(obj)
    expect(fn).toHaveBeenCalledTimes(1)
    $s.set({ a: 1 }) // new ref → does fire
    expect(fn).toHaveBeenCalledTimes(2)
  })

  it('recomputes lazily when no subscribers exist', () => {
    // Without downstream subs, invalidation should stay lazy: the
    // derived recomputes on the next read, not eagerly on dep change.
    const $x = signal(1)
    const compute = vi.fn(() => $x() * 2)
    const $d = derived(compute)
    expect($d()).toBe(2)
    expect(compute).toHaveBeenCalledTimes(1)
    $x.set(2)
    $x.set(3)
    $x.set(4)
    expect(compute).toHaveBeenCalledTimes(1)
    expect($d()).toBe(8)
    expect(compute).toHaveBeenCalledTimes(2)
  })

  it('chained derived also memoizes downstream', () => {
    const $x = signal(0)
    const $isPositive = derived(() => $x() > 0)
    const $label = derived(() => ($isPositive() ? 'yes' : 'no'))
    const computeLabel = vi.fn(() => $label())
    const fn = vi.fn(computeLabel)
    effect(fn)
    expect(fn).toHaveBeenCalledTimes(1)
    $x.set(1)
    expect(fn).toHaveBeenCalledTimes(2)
    $x.set(2)
    // $isPositive stayed true → $label stayed 'yes' → no re-run.
    expect(fn).toHaveBeenCalledTimes(2)
  })
})

describe('batch', () => {
  it('coalesces multiple writes into one notification', () => {
    const $a = signal(0)
    const $b = signal(0)
    const fn = vi.fn()
    effect(() => {
      $a()
      $b()
      fn()
    })
    expect(fn).toHaveBeenCalledTimes(1)
    batch(() => {
      $a.set(1)
      $b.set(2)
    })
    expect(fn).toHaveBeenCalledTimes(2) // initial + 1 batched
  })

  it('nested batches still coalesce to outermost', () => {
    const $a = signal(0)
    const fn = vi.fn()
    effect(() => {
      $a()
      fn()
    })
    batch(() => {
      $a.set(1)
      batch(() => {
        $a.set(2)
        $a.set(3)
      })
      $a.set(4)
    })
    expect(fn).toHaveBeenCalledTimes(2) // initial + 1 batched
  })
})

describe('untrack', () => {
  it('reads inside untrack do not subscribe', () => {
    const $x = signal(0)
    const $y = signal(0)
    const fn = vi.fn()
    effect(() => {
      $x()
      untrack(() => $y())
      fn()
    })
    expect(fn).toHaveBeenCalledTimes(1)
    $y.set(1)
    expect(fn).toHaveBeenCalledTimes(1) // y did not trigger
    $x.set(1)
    expect(fn).toHaveBeenCalledTimes(2) // x did
  })
})
