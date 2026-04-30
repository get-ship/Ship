import { afterEach, describe, expect, it, vi } from 'vitest'
import type { Signal } from '@ship-set/runtime'
import { attach } from '../../src/attach'
import { clearContainers, makeModule, mountSsr } from './helpers'

afterEach(clearContainers)

describe('attach/boundaries', () => {
  it('does not crash on placeholder boundary; warns on flip', () => {
    // Placeholder shape: server skipped this branch.
    const html = '<template data-set-if="b0"></template>'
    const root = mountSsr(html, { ready: false })
    let ready: Signal<boolean> | undefined
    const mod = makeModule(
      (ctx) => {
        ready = ctx.scope!.ready as unknown as Signal<boolean>
        return {}
      },
      [{ key: 'b0', kind: 'if', expr: 'ready()', deps: ['ready'] }],
    )
    const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {})
    try {
      const dispose = attach(root, { module: mod })
      expect(warnSpy).not.toHaveBeenCalled()
      ready!.set(true)
      expect(
        warnSpy.mock.calls.some((c) =>
          (c[0] as string).includes('control-flow signal changed'),
        ),
      ).toBe(true)
      dispose()
    } finally {
      warnSpy.mockRestore()
    }
  })

  it('wires leaf bindings inside an active branch wrapper', () => {
    // Active branch: server rendered, wrapper is `<div display:contents>`.
    const html =
      '<div data-set-if="b0" style="display:contents">' +
      '<p><!--set:text:b1-->ada<!--/set--></p>' +
      '</div>'
    const root = mountSsr(html, { ready: true, name: 'ada' })
    let nameSig: Signal<string> | undefined
    const mod = makeModule(
      (ctx) => {
        nameSig = ctx.scope!.name as unknown as Signal<string>
        return {}
      },
      [
        { key: 'b0', kind: 'if', expr: 'ready()', deps: ['ready'] },
        { key: 'b1', kind: 'text', expr: 'name()', deps: ['name'] },
      ],
    )
    const dispose = attach(root, { module: mod })
    expect(root.querySelector('p')!.textContent).toBe('ada')
    nameSig!.set('grace')
    expect(root.querySelector('p')!.textContent).toBe('grace')
    dispose()
  })
})
