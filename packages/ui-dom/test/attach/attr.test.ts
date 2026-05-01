import { afterEach, describe, expect, it } from 'vitest'
import type { Signal } from '@ship/ui-runtime'
import { attach } from '../../src/attach'
import { clearContainers, makeModule, mountSsr } from './helpers'

afterEach(clearContainers)

describe('attach/attr', () => {
  it('updates attribute value when the expression signal changes', () => {
    const html = '<a attr:href="b0" href="/initial">go</a>'
    const root = mountSsr(html, { href: '/initial' })
    let h: Signal<string> | undefined
    const mod = makeModule(
      (ctx) => {
        h = ctx.scope!.href as unknown as Signal<string>
        return {}
      },
      [{ key: 'b0', kind: 'attr:href', expr: 'href()', deps: ['href'] }],
    )
    const dispose = attach(root, { module: mod })
    const a = root.querySelector('a')!
    expect(a.getAttribute('href')).toBe('/initial')
    h!.set('/about')
    expect(a.getAttribute('href')).toBe('/about')
    h!.set(null as unknown as string)
    expect(a.hasAttribute('href')).toBe(false)
    dispose()
  })
})

describe('attach/style', () => {
  it('sets and clears an inline style property', () => {
    const html = '<div style:color="b0">x</div>'
    const root = mountSsr(html, { c: 'red' })
    let c: Signal<string> | undefined
    const mod = makeModule(
      (ctx) => {
        c = ctx.scope!.c as unknown as Signal<string>
        return {}
      },
      [{ key: 'b0', kind: 'style:color', expr: 'c()', deps: ['c'] }],
    )
    const dispose = attach(root, { module: mod })
    const div = root.querySelector('div')!
    expect(div.style.color).toBe('red')
    c!.set('blue')
    expect(div.style.color).toBe('blue')
    c!.set('')
    expect(div.style.color).toBe('')
    dispose()
  })
})
