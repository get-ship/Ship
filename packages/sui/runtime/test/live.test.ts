import { describe, expect, it, vi } from 'vitest'
import { signal } from '../src/signal'
import { live, type LiveSignal, type LiveSource } from '../src/live'

const tick = () => new Promise((r) => setTimeout(r, 0))

describe('live', () => {
  it('starts in loading', () => {
    const s = live(() => new Promise<number>(() => {}))
    expect(s().kind).toBe('loading')
  })

  it('transitions to ok on resolve', async () => {
    const s = live(() => Promise.resolve(42))
    expect(s().kind).toBe('loading')
    await tick()
    await tick()
    const v = s()
    expect(v.kind).toBe('ok')
    if (v.kind === 'ok') expect(v.value).toBe(42)
  })

  it('transitions to error on reject', async () => {
    const s = live(() => Promise.reject(new Error('boom')))
    await tick()
    await tick()
    const v = s()
    expect(v.kind).toBe('error')
    if (v.kind === 'error') expect(v.error.message).toBe('boom')
  })

  it('wraps non-Error rejections in Error', async () => {
    const s = live(() => Promise.reject('plain'))
    await tick()
    await tick()
    const v = s()
    expect(v.kind).toBe('error')
    if (v.kind === 'error') expect(v.error.message).toBe('plain')
  })

  it('catches synchronous throws from source', async () => {
    const s = live<number>(() => {
      throw new Error('sync')
    })
    await tick()
    const v = s()
    expect(v.kind).toBe('error')
    if (v.kind === 'error') expect(v.error.message).toBe('sync')
  })

  it('accepts sync return values', async () => {
    const s = live(() => 7 as number | Promise<number>)
    await tick()
    const v = s()
    expect(v.kind).toBe('ok')
    if (v.kind === 'ok') expect(v.value).toBe(7)
  })

  it('re-fires when a tracked dep changes — flips to loading then ok', async () => {
    const id = signal(1)
    const s = live(() => Promise.resolve(`v${id()}`))
    await tick()
    await tick()
    expect(s().kind).toBe('ok')
    id.set(2)
    expect(s().kind).toBe('loading')
    await tick()
    await tick()
    const v = s()
    expect(v.kind).toBe('ok')
    if (v.kind === 'ok') expect(v.value).toBe('v2')
  })

  it('invalidates old in-flight resolutions (token)', async () => {
    const id = signal(1)
    let resolveFirst: (v: string) => void = () => {}
    let resolveSecond: (v: string) => void = () => {}
    const s = live(() => {
      const cur = id()
      if (cur === 1) return new Promise<string>((r) => { resolveFirst = r })
      return new Promise<string>((r) => { resolveSecond = r })
    })
    expect(s().kind).toBe('loading')
    id.set(2)
    expect(s().kind).toBe('loading')
    // Stale resolution arrives — must be ignored.
    resolveFirst('STALE')
    await tick()
    expect(s().kind).toBe('loading')
    resolveSecond('FRESH')
    await tick()
    const v = s()
    expect(v.kind).toBe('ok')
    if (v.kind === 'ok') expect(v.value).toBe('FRESH')
  })

  it('LiveSource: subscribe owns lifecycle, transitions to ok', () => {
    const source: LiveSource<number> = {
      subscribe: (sig: LiveSignal<number>) => {
        sig.set({ kind: 'ok', value: 11 })
        return () => {}
      },
    }
    const s = live(() => source)
    const v = s()
    expect(v.kind).toBe('ok')
    if (v.kind === 'ok') expect(v.value).toBe(11)
  })

  it('LiveSource: dispose runs on tracked-dep re-fire', () => {
    const id = signal(1)
    const dispose = vi.fn()
    const s = live<number>(() => ({
      subscribe: (sig: LiveSignal<number>) => {
        const cur = id()
        sig.set({ kind: 'ok', value: cur * 10 })
        return dispose
      },
    }))
    expect(s()).toEqual({ kind: 'ok', value: 10 })
    id.set(2)
    expect(dispose).toHaveBeenCalledOnce()
    expect(s()).toEqual({ kind: 'ok', value: 20 })
  })

  it('calls onError on rejection', async () => {
    const onError = vi.fn()
    live(() => Promise.reject(new Error('oops')), { onError })
    await tick()
    await tick()
    expect(onError).toHaveBeenCalledOnce()
    expect((onError.mock.calls[0]![0] as Error).message).toBe('oops')
  })
})
