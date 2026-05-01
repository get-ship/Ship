import { afterEach, describe, expect, it } from 'vitest'
import type { Signal } from '@ship/ui-runtime'
import { attach } from '../../src/attach'
import { clearContainers, makeModule, mountSsr } from './helpers'

afterEach(clearContainers)

describe('attach/event', () => {
  it('on:click invokes the bound handler resolved against scope', () => {
    const html = '<button data-set-on:click="b0">+</button>'
    const root = mountSsr(html, { count: 0 })
    let countSig: Signal<number> | undefined
    const mod = makeModule(
      (ctx) => {
        countSig = ctx.scope!.count as unknown as Signal<number>
        return {
          incr: () => countSig!.set(countSig!() + 1),
        }
      },
      [{ key: 'b0', kind: 'on:click', expr: 'incr', deps: ['incr'] }],
    )
    const dispose = attach(root, { module: mod })
    const btn = root.querySelector('button')!
    btn.click()
    btn.click()
    expect(countSig!()).toBe(2)
    dispose()
    btn.click()
    expect(countSig!()).toBe(2)
  })

  it('does not crash when expr resolves to a non-function', () => {
    const html = '<button data-set-on:click="b0">x</button>'
    const root = mountSsr(html, {})
    const mod = makeModule(() => ({ notAFn: 7 }), [
      { key: 'b0', kind: 'on:click', expr: 'notAFn', deps: ['notAFn'] },
    ])
    const dispose = attach(root, { module: mod })
    expect(() => root.querySelector('button')!.click()).not.toThrow()
    dispose()
  })
})
