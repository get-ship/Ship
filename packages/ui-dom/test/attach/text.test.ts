import { afterEach, describe, expect, it } from 'vitest'
import type { Signal } from '@ship/ui-runtime'
import { attach } from '../../src/attach'
import { clearContainers, makeModule, mountSsr } from './helpers'

afterEach(clearContainers)

describe('attach/text', () => {
  it('updates inlined SSR text on signal change', () => {
    const html =
      '<p>count: <span data-test="value"><!--sui:text:b0-->0<!--/sui--></span></p>'
    const root = mountSsr(html, { count: 0 })
    let countSig: Signal<number> | undefined
    const mod = makeModule(
      (ctx) => {
        countSig = ctx.scope!.count as unknown as Signal<number>
        return {}
      },
      [{ key: 'b0', kind: 'text', expr: 'count()', deps: ['count'] }],
    )
    const dispose = attach(root, { module: mod })
    expect(root.querySelector('[data-test="value"]')!.textContent).toBe('0')
    countSig!.set(7)
    expect(root.querySelector('[data-test="value"]')!.textContent).toBe('7')
    dispose()
    // After dispose, signal updates no longer push to the DOM.
    countSig!.set(99)
    expect(root.querySelector('[data-test="value"]')!.textContent).toBe('7')
  })
})
