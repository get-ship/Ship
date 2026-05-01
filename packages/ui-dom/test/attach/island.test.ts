import { afterEach, describe, expect, it } from 'vitest'
import { readInitIsland } from '../../src/attach/island'
import { clearContainers, mountSsr } from './helpers'

afterEach(clearContainers)

describe('attach/island', () => {
  it('parses scalars into writable Signals', () => {
    const root = mountSsr('<p>hi</p>', { count: 0, name: 'ada' })
    const r = readInitIsland(root)
    expect(r.stateKeys.has('count')).toBe(true)
    expect(r.stateKeys.has('name')).toBe(true)
    expect(r.liveKeys.size).toBe(0)
    const count = r.scope.count!
    expect(count()).toBe(0)
    count.set(5)
    expect(count()).toBe(5)
  })

  it('wraps LiveState envelopes as Signals carrying the envelope', () => {
    const root = mountSsr('<p></p>', {
      agents: { kind: 'loading' },
      ready: true,
    })
    const r = readInitIsland(root)
    expect(r.liveKeys.has('agents')).toBe(true)
    expect(r.stateKeys.has('agents')).toBe(false)
    expect(r.stateKeys.has('ready')).toBe(true)
    expect(r.scope.agents!()).toEqual({ kind: 'loading' })
  })

  it('throws a clear error when the island is missing', () => {
    const c = document.createElement('div')
    document.body.appendChild(c)
    c.innerHTML = '<p>nothing here</p>'
    try {
      expect(() => readInitIsland(c)).toThrow(/__set_init__/)
    } finally {
      c.remove()
    }
  })

  it('handles object/array state values', () => {
    const root = mountSsr('<p></p>', {
      profile: { name: 'ada' },
      tags: ['a', 'b'],
    })
    const r = readInitIsland(root)
    expect(r.scope.profile!()).toEqual({ name: 'ada' })
    expect(r.scope.tags!()).toEqual(['a', 'b'])
  })
})
