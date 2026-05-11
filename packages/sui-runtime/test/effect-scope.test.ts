import { describe, it, expect, vi } from 'vitest'

import {
  createScope,
  createChildScope,
  withScope,
  onCleanup,
  currentScope,
  type EffectScope,
} from '../src/effect-scope'

describe('effect-scope', () => {
  it('runs registered cleanups in reverse order on dispose', () => {
    const scope = createScope()
    const order: string[] = []
    withScope(scope, () => {
      onCleanup(() => order.push('a'))
      onCleanup(() => order.push('b'))
      onCleanup(() => order.push('c'))
    })
    scope.dispose()
    expect(order).toEqual(['c', 'b', 'a'])
  })

  it('treats onCleanup outside a scope as a no-op', () => {
    // No throws, no side effects — just doesn't register anywhere.
    expect(() => onCleanup(() => {})).not.toThrow()
    expect(currentScope()).toBeNull()
  })

  it('restores the prior scope on return AND on throw', () => {
    const outer = createScope()
    const inner = createScope()
    let observedInside: EffectScope | null = null
    withScope(outer, () => {
      withScope(inner, () => {
        observedInside = currentScope()
      })
      expect(currentScope()).toBe(outer)
    })
    expect(observedInside).toBe(inner)
    expect(currentScope()).toBeNull()

    // Throwing path
    expect(() => {
      withScope(outer, () => {
        throw new Error('boom')
      })
    }).toThrow('boom')
    expect(currentScope()).toBeNull()
  })

  it('dispose is idempotent', () => {
    const scope = createScope()
    const cleanup = vi.fn()
    withScope(scope, () => onCleanup(cleanup))
    scope.dispose()
    scope.dispose()
    scope.dispose()
    expect(cleanup).toHaveBeenCalledTimes(1)
  })

  it('logs and continues when a cleanup throws', () => {
    const scope = createScope()
    const after = vi.fn()
    const errSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
    withScope(scope, () => {
      onCleanup(after) // last registered → first to run
      onCleanup(() => {
        throw new Error('cleanup failed')
      })
    })
    scope.dispose()
    expect(after).toHaveBeenCalledOnce()
    expect(errSpy).toHaveBeenCalledWith(
      expect.stringContaining('[sui/scope] cleanup threw'),
      expect.any(Error),
    )
    errSpy.mockRestore()
  })

  it('disposes child when parent disposes', () => {
    const parent = createScope()
    const childFn = vi.fn()
    let child: EffectScope | undefined
    withScope(parent, () => {
      child = createChildScope()
      withScope(child, () => onCleanup(childFn))
    })
    expect(child!.disposed).toBe(false)
    parent.dispose()
    expect(child!.disposed).toBe(true)
    expect(childFn).toHaveBeenCalledOnce()
  })

  it('child stays alive when its scope finishes but parent is still active', () => {
    const parent = createScope()
    let child: EffectScope | undefined
    const childFn = vi.fn()
    withScope(parent, () => {
      child = createChildScope()
      withScope(child, () => onCleanup(childFn))
      // child WAS the current scope here, but withScope returned —
      // child itself is still alive until its parent disposes.
    })
    expect(child!.disposed).toBe(false)
    expect(childFn).not.toHaveBeenCalled()

    parent.dispose()
    expect(child!.disposed).toBe(true)
    expect(childFn).toHaveBeenCalledOnce()
  })

  it('runs late-registered cleanups immediately if scope already disposed', () => {
    const scope = createScope()
    scope.dispose()
    const fn = vi.fn()
    scope.cleanup(fn)
    expect(fn).toHaveBeenCalledOnce()
  })

  it('createChildScope without explicit parent picks up the current scope', () => {
    const root = createScope()
    let child: EffectScope | undefined
    withScope(root, () => {
      child = createChildScope()
    })
    expect(child!.disposed).toBe(false)
    root.dispose()
    expect(child!.disposed).toBe(true)
  })

  it('disposed flag flips on dispose', () => {
    const scope = createScope()
    expect(scope.disposed).toBe(false)
    scope.dispose()
    expect(scope.disposed).toBe(true)
  })
})
