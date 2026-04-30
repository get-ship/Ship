import { afterEach, describe, expect, it } from 'vitest'
import type { Signal } from '@ship-set/runtime'
import { attach } from '../../src/attach'
import { clearContainers, makeModule, mountSsr } from './helpers'

afterEach(clearContainers)

describe('attach/class', () => {
  it('toggles a class based on a truthy expression', () => {
    const html = '<button data-set-class:active="b0" class="btn active">x</button>'
    const root = mountSsr(html, { active: true })
    let a: Signal<boolean> | undefined
    const mod = makeModule(
      (ctx) => {
        a = ctx.scope!.active as unknown as Signal<boolean>
        return {}
      },
      [{ key: 'b0', kind: 'class:active', expr: 'active()', deps: ['active'] }],
    )
    const dispose = attach(root, { module: mod })
    const btn = root.querySelector('button')!
    expect(btn.classList.contains('active')).toBe(true)
    a!.set(false)
    expect(btn.classList.contains('active')).toBe(false)
    a!.set(true)
    expect(btn.classList.contains('active')).toBe(true)
    dispose()
  })
})
