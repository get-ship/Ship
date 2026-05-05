import { describe, expect, it } from 'vitest'

import { effect, isRef, ref, signal } from '../src'

describe('ref()', () => {
  it('seeds to null', () => {
    const r = ref<HTMLDivElement>()
    expect(r()).toBeNull()
  })

  it('is writable like a signal', () => {
    const r = ref<HTMLButtonElement>()
    const btn = document.createElement('button')
    r.set(btn)
    expect(r()).toBe(btn)
    r.set(null)
    expect(r()).toBeNull()
  })

  it('subscribes effects on read', () => {
    const r = ref<HTMLDivElement>()
    const seen: (HTMLDivElement | null)[] = []
    effect(() => {
      seen.push(r())
    })
    const a = document.createElement('div')
    r.set(a)
    r.set(null)
    expect(seen).toEqual([null, a, null])
  })

  it('passes the isRef writable check', () => {
    expect(isRef(ref())).toBe(true)
    expect(isRef(signal(0))).toBe(true)
  })

  it('rejects non-writable values via isRef', () => {
    expect(isRef(null)).toBe(false)
    expect(isRef({})).toBe(false)
    expect(isRef(() => null)).toBe(false)
  })
})
