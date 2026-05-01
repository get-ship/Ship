import { afterEach, describe, expect, it } from 'vitest'
import type { Signal } from '@ship/ui-runtime'
import { attach } from '../../src/attach'
import { clearContainers, makeModule, mountSsr } from './helpers'

afterEach(clearContainers)

describe('attach/bind:value', () => {
  it('round-trips DOM ↔ signal both ways', () => {
    const html = '<input bind:value="b0" value="seed">'
    const root = mountSsr(html, { query: 'seed' })
    let q: Signal<string> | undefined
    const mod = makeModule(
      (ctx) => {
        q = ctx.scope!.query as unknown as Signal<string>
        return {}
      },
      [{ key: 'b0', kind: 'bind:value', expr: 'query', deps: ['query'] }],
    )
    const dispose = attach(root, { module: mod })
    const input = root.querySelector('input')!
    expect(input.value).toBe('seed')

    input.value = 'typed'
    input.dispatchEvent(new Event('input'))
    expect(q!()).toBe('typed')

    q!.set('programmatic')
    expect(input.value).toBe('programmatic')

    dispose()
    input.value = 'after-dispose'
    input.dispatchEvent(new Event('input'))
    expect(q!()).toBe('programmatic')
  })
})

describe('attach/bind:checked', () => {
  it('round-trips checkbox state', () => {
    const html = '<input type="checkbox" bind:checked="b0">'
    const root = mountSsr(html, { on: false })
    let on: Signal<boolean> | undefined
    const mod = makeModule(
      (ctx) => {
        on = ctx.scope!.on as unknown as Signal<boolean>
        return {}
      },
      [{ key: 'b0', kind: 'bind:checked', expr: 'on', deps: ['on'] }],
    )
    const dispose = attach(root, { module: mod })
    const cb = root.querySelector('input')!
    expect(cb.checked).toBe(false)
    cb.checked = true
    cb.dispatchEvent(new Event('change'))
    expect(on!()).toBe(true)
    on!.set(false)
    expect(cb.checked).toBe(false)
    dispose()
  })
})
