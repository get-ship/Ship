import { describe, expect, it, vi } from 'vitest'
import { view } from '../src/view'
import type { ViewContext } from '../src/view'

describe('view', () => {
  it('returns { setup: fn }', () => {
    const fn = () => ({ count: 0 })
    const def = view(fn)
    expect(def.setup).toBe(fn)
  })

  it('setup receives the context object', () => {
    const setup = vi.fn((ctx: ViewContext) => ctx.route)
    const def = view(setup)
    const ctx: ViewContext = {
      route: { id: 'a-1' },
      ship: undefined,
      navigate: () => {},
    }
    const out = def.setup(ctx)
    expect(setup).toHaveBeenCalledWith(ctx)
    expect(out).toEqual({ id: 'a-1' })
  })

  it('setup return values are passed through', () => {
    const def = view(() => ({ hello: 'world', n: 42 }))
    const out = def.setup({
      route: {},
      ship: undefined,
      navigate: () => {},
    })
    expect(out).toEqual({ hello: 'world', n: 42 })
  })

  it('navigate callback is invoked from setup-returned bindings', () => {
    const navigate = vi.fn()
    const def = view((ctx) => ({ go: (p: string) => ctx.navigate(p) }))
    const bindings = def.setup({ route: {}, ship: undefined, navigate })
    bindings.go('/home')
    expect(navigate).toHaveBeenCalledWith('/home')
  })
})
