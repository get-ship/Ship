// Counter view e2e — Phase E v1 demo target.
//
// Mirrors the SSR walker's output for `crates/ship-set-compiler/tests/
// fixtures/counter.html.set` plus a hand-written compiled-module
// shape. Pulling the real Rust compiler into vitest is too heavy;
// the integration test in shipd already exercises the genuine path.

import { afterEach, describe, expect, it } from 'vitest'
import type { Signal } from '@ship-set/runtime'
import { attach } from '../../src/attach'
import { clearContainers, makeModule, mountSsr } from './helpers'

afterEach(clearContainers)

const COUNTER_HTML = `
<div class="counter">
  <p>count: <span data-test="value"><!--set:text:b0-->0<!--/set--></span></p>
  <button data-test="inc" data-set-on:click="b1">increment</button>
  <button data-test="reset" data-set-on:click="b2">reset</button>
</div>`.trim()

describe('attach — counter e2e', () => {
  it('renders, increments, resets, and stops after dispose', () => {
    const root = mountSsr(COUNTER_HTML, { count: 0 })
    let countSig: Signal<number> | undefined
    const mod = makeModule(
      (ctx) => {
        countSig = ctx.scope!.count as unknown as Signal<number>
        return {
          incr: () => countSig!.set(countSig!() + 1),
          reset: () => countSig!.set(0),
        }
      },
      [
        { key: 'b0', kind: 'text', expr: 'count()', deps: ['count'] },
        {
          key: 'b1',
          kind: 'on:click',
          expr: 'incr',
          deps: ['incr'],
        },
        {
          key: 'b2',
          kind: 'on:click',
          expr: 'reset',
          deps: ['reset'],
        },
      ],
    )

    const dispose = attach(root, { module: mod })

    const value = root.querySelector('[data-test="value"]')!
    const inc = root.querySelector('[data-test="inc"]') as HTMLButtonElement
    const reset = root.querySelector('[data-test="reset"]') as HTMLButtonElement

    expect(value.textContent).toBe('0')

    inc.click()
    expect(value.textContent).toBe('1')

    inc.click()
    inc.click()
    expect(value.textContent).toBe('3')
    expect(countSig!()).toBe(3)

    reset.click()
    expect(value.textContent).toBe('0')

    dispose()

    inc.click()
    expect(value.textContent).toBe('0')
    expect(countSig!()).toBe(0)
  })
})
