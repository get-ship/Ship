// scope: block + auto-provide flow
//
// The scope: factory contributes signals/derived/hooks to the merged
// scope; signals wrapped with shared() get auto-published on the host
// for descendants to consume via useShared.

import { describe, expect, it } from 'vitest'
import { signal, derived, shared, useShared } from '../src/index'
import { resolveController } from '../src/controller-resolver'
import type { ResolverContext } from '../src/controller-resolver-types'
import type { ShipSDK } from '../src/ship'

const fakeShip = {} as ShipSDK

function ctx(host?: Element): ResolverContext {
  return { ship: fakeShip, host }
}

describe('scope: block', () => {
  it('factory output merges into bindings', () => {
    const def = {
      scope: () => ({
        count: signal(7),
        doubled: derived(() => 14),
      }),
    }
    const bindings = resolveController(def, ctx())
    expect(typeof bindings.count).toBe('function')
    expect((bindings.count as () => number)()).toBe(7)
    expect((bindings.doubled as () => number)()).toBe(14)
  })

  it('on: handlers can mutate scope: signals via writable proxy', () => {
    const def = {
      scope: () => ({ count: signal(0) }),
      on: {
        inc: (s: { count: number }) => { s.count = s.count + 1 },
      },
    }
    const bindings = resolveController(def, ctx())
    const count = bindings.count as () => number
    const inc = bindings.inc as () => void
    expect(count()).toBe(0)
    inc()
    inc()
    expect(count()).toBe(2)
  })

  it('shared() signals are auto-provided on host under tag name', () => {
    const host = document.createElement('sui-cmd-bar')
    const child = document.createElement('span')
    host.appendChild(child)
    document.body.appendChild(host)

    const def = {
      scope: () => ({
        mode: shared(signal('find')),
        value: shared(signal('')),
        local: signal(0),  // not shared
      }),
    }
    resolveController(def, ctx(host))

    const ctxFound = useShared<{ mode: () => string; value: () => string; local?: never }>(
      child,
      'sui-cmd-bar',
    )
    expect(ctxFound).not.toBeNull()
    expect(ctxFound?.mode()).toBe('find')
    expect(ctxFound?.value()).toBe('')
    expect((ctxFound as Record<string, unknown> | null)?.local).toBeUndefined()

    document.body.removeChild(host)
  })

  it('without shared() entries, no provide call is made', () => {
    const host = document.createElement('sui-x')
    document.body.appendChild(host)
    const def = {
      scope: () => ({ count: signal(0) }),
    }
    resolveController(def, ctx(host))
    // Property is not set; useShared returns null on host or descendants.
    expect((host as Record<string, unknown>)['__sui_ctx_sui-x']).toBeUndefined()
    document.body.removeChild(host)
  })

  it('skipped when no scope: block declared', () => {
    const def = { on: { fn: () => {} } }
    const bindings = resolveController(def, ctx())
    expect(typeof bindings.fn).toBe('function')
  })
})
