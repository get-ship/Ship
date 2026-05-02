// `bind:this` — attach-time wirer tests.
//
// Mirrors the bind:value / bind:checked test pattern: hand-write SSR
// HTML carrying a `bind:this="bN"` marker, build a compiled-module
// stub with a matching binding entry, and verify the runtime resolves
// the element into a writable signal on the scope.
//
// We capture the ref by having the test setup return a pre-declared
// `signal(null)` under the same name the binding manifest names. The
// wirer reuses the existing signal instead of replacing it (the
// supported pattern when an author wants the ref reachable from
// `derived` declarations earlier in the setup body).

import { afterEach, describe, expect, it } from 'vitest'
import { signal, type Signal } from '@ship/ui-runtime'
import { attach } from '../../src/attach'
import { clearContainers, makeModule, mountSsr } from './helpers'

afterEach(clearContainers)

describe('attach/bind:this', () => {
  it('writes the SSR element into the ref signal on attach', () => {
    const html = '<button bind:this="b0" part="x">Save</button>'
    const root = mountSsr(html, {})
    const ref: Signal<Element | null> = signal<Element | null>(null)
    const mod = makeModule(() => ({ btnEl: ref }), [
      { key: 'b0', kind: 'bind:this', expr: 'btnEl', deps: ['btnEl'] },
    ])

    expect(ref()).toBeNull()
    const dispose = attach(root, { module: mod })

    const button = root.querySelector('button')!
    expect(ref()).toBe(button)
    expect((ref() as HTMLButtonElement).getAttribute('part')).toBe('x')

    dispose()
    expect(ref()).toBeNull()
  })

  it('dispose is idempotent', () => {
    const html = '<div bind:this="b0">x</div>'
    const root = mountSsr(html, {})
    const ref = signal<Element | null>(null)
    const mod = makeModule(() => ({ panel: ref }), [
      { key: 'b0', kind: 'bind:this', expr: 'panel', deps: ['panel'] },
    ])
    const dispose = attach(root, { module: mod })
    expect(ref()).toBe(root.querySelector('div'))
    dispose()
    dispose()
    expect(ref()).toBeNull()
  })
})
