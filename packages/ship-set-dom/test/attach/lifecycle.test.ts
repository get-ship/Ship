import { afterEach, describe, expect, it } from 'vitest'
import { attach } from '../../src/attach'
import { clearContainers, makeModule, mountSsr } from './helpers'

afterEach(clearContainers)

describe('attach/lifecycle', () => {
  it('runs mount() after attach and unmount() on dispose', () => {
    const root = mountSsr('<p>hi</p>', {})
    const calls: string[] = []
    const lifecycle = {
      state: { x: 1 },
      mount: (state: Record<string, unknown>, el: HTMLElement) => {
        calls.push(`mount:${state.x}:${el.tagName}`)
      },
      unmount: (state: Record<string, unknown>) => {
        calls.push(`unmount:${state.x}`)
      },
    }
    const mod = makeModule(() => {
      const ret: Record<string, unknown> = {}
      Object.defineProperty(ret, '__shipSetLifecycle', {
        value: lifecycle,
        enumerable: false,
      })
      return ret
    }, [])
    const dispose = attach(root, { module: mod })
    expect(calls).toEqual([`mount:1:${root.tagName}`])
    dispose()
    expect(calls).toEqual([`mount:1:${root.tagName}`, 'unmount:1'])
  })

  it('disposer is idempotent', () => {
    const root = mountSsr('<p>hi</p>', {})
    let unmounts = 0
    const mod = makeModule(() => {
      const ret: Record<string, unknown> = {}
      Object.defineProperty(ret, '__shipSetLifecycle', {
        value: { state: {}, unmount: () => unmounts++ },
        enumerable: false,
      })
      return ret
    }, [])
    const dispose = attach(root, { module: mod })
    dispose()
    dispose()
    expect(unmounts).toBe(1)
  })

  it('catches mount() / unmount() errors and continues', () => {
    const errs: string[] = []
    const orig = console.error
    console.error = (...args) => {
      errs.push(args.map(String).join(' '))
    }
    try {
      const root = mountSsr('<p>hi</p>', {})
      const mod = makeModule(() => {
        const ret: Record<string, unknown> = {}
        Object.defineProperty(ret, '__shipSetLifecycle', {
          value: {
            state: {},
            mount: () => {
              throw new Error('boom-mount')
            },
            unmount: () => {
              throw new Error('boom-unmount')
            },
          },
          enumerable: false,
        })
        return ret
      }, [])
      const dispose = attach(root, { module: mod })
      expect(errs.some((e) => e.includes('boom-mount'))).toBe(true)
      dispose()
      expect(errs.some((e) => e.includes('boom-unmount'))).toBe(true)
    } finally {
      console.error = orig
    }
  })
})
