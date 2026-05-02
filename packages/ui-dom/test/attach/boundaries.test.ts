// Phase 2 — client-side control-flow boundaries.
//
// The compiler emits `__b<N>_attach: (target) => disposer` named
// handlers on the setup return for every `{#if}` / `{#for}` /
// `{#match}` boundary. `attach()` walks SSR'd `<sui-{kind}>` wrappers
// and dispatches to the matching handler, which swaps the wrapper for
// a Comment anchor and runs `bindIf` / `bindFor` against the live
// signal layer. No server round-trip; reactivity is single-source via
// signals. These tests build hand-written compiled-module shapes that
// mirror what the compiler would emit and verify the swap behavior.

import { afterEach, describe, expect, it, vi } from 'vitest'
import { bindFor, bindIf, signal, type Signal } from '@ship/ui-runtime'
import { attach } from '../../src/attach'
import type { SetupContext } from '../../src/attach'
import { clearContainers, makeModule, mountSsr } from './helpers'

afterEach(() => {
  clearContainers()
  vi.restoreAllMocks()
})

describe('attach/boundaries (client-side)', () => {
  it('replaces SSR\'d {#if} active wrapper with client-rendered branch', () => {
    const html =
      '<sui-if key="b0" style="display:contents">' +
      '<p data-test="ssr">stale-from-ssr</p>' +
      '</sui-if>'
    const root = mountSsr(html, { ready: true, name: 'ada' })

    let nameSig: Signal<string> | undefined
    const mod = makeModule(
      (ctx: SetupContext) => {
        const ready = ctx.scope!.ready as () => boolean
        nameSig = ctx.scope!.name as unknown as Signal<string>
        return {
          __b0_attach: (target: Element) => {
            const anchor = document.createComment('if')
            target.parentNode!.replaceChild(anchor, target)
            return bindIf({
              anchor,
              test: () => (ready() ? 0 : -1),
              branches: [
                () => {
                  const p = document.createElement('p')
                  p.setAttribute('data-test', 'csr')
                  const t = document.createTextNode('')
                  p.appendChild(t)
                  // Inline reactivity: signal subscription via effect
                  // would be in a real bindText; for the test, assign
                  // synchronously since we won't change the signal
                  // before checking.
                  t.nodeValue = nameSig!()
                  return { nodes: [p] }
                },
              ],
            })
          },
        }
      },
      [],
    )

    const dispose = attach(root, { module: mod })
    // SSR'd `<p data-test="ssr">` is gone; client-rendered tree is in.
    expect(root.querySelector('[data-test="ssr"]')).toBeNull()
    const csr = root.querySelector('[data-test="csr"]')
    expect(csr).not.toBeNull()
    expect(csr!.textContent).toBe('ada')
    dispose()
  })

  it('skips inner SSR\'d boundaries detached by outer handler', () => {
    // Outer if active in SSR, with inner if-branch nested. After the
    // outer attach, inner SSR'd `<sui-if>` is detached — its
    // `__b1_attach` should NOT be called.
    const html =
      '<sui-if key="b0" style="display:contents">' +
      '<sui-if key="b1" style="display:contents"><p>inner-ssr</p></sui-if>' +
      '</sui-if>'
    const root = mountSsr(html, { outer: true })

    const innerCalls: number[] = []
    const mod = makeModule(
      () => ({
        __b0_attach: (target: Element) => {
          const anchor = document.createComment('if')
          target.parentNode!.replaceChild(anchor, target)
          return bindIf({
            anchor,
            test: () => 0,
            branches: [
              () => {
                const p = document.createElement('p')
                p.textContent = 'outer-csr'
                return { nodes: [p] }
              },
            ],
          })
        },
        __b1_attach: () => {
          innerCalls.push(1)
          return () => {}
        },
      }),
      [],
    )

    attach(root, { module: mod })
    expect(innerCalls.length).toBe(0)
    expect(root.querySelector('p')?.textContent).toBe('outer-csr')
  })

  it('bindFor with per-iteration scope renders correct text per item', () => {
    // The studio empty-tile bug: per-iteration `item` scope must
    // propagate to inner `bindText`. Build a compiled-module shape
    // mirroring what the compiler emits for {#for item of items}.
    const html =
      '<div data-test="grid">' +
      '<sui-for key="b0" style="display:contents">' +
      '<div class="tile"><code>stale</code></div>' +
      '</sui-for>' +
      '</div>'
    const root = mountSsr(html, {
      items: [
        { id: 'one', kind: 'lit' },
        { id: 'two', kind: 'sui' },
        { id: 'three', kind: 'view' },
      ],
    })

    const mod = makeModule(
      (ctx: SetupContext) => {
        const items = ctx.scope!.items as () => readonly { id: string; kind: string }[]
        return {
          __b0_attach: (target: Element) => {
            const anchor = document.createComment('for')
            target.parentNode!.replaceChild(anchor, target)
            return bindFor<{ id: string; kind: string }>({
              anchor,
              items: () => items(),
              key: (it) => it.id,
              render: (item) => {
                // Mirror what the compiler emits per iteration:
                // childScope captures `item`; bindings inside resolve
                // against that childScope.
                const tile = document.createElement('div')
                tile.className = 'tile'
                tile.setAttribute('data-test', 'tile')
                const code = document.createElement('code')
                code.textContent = item.id
                tile.appendChild(code)
                return { nodes: [tile] }
              },
            })
          },
        }
      },
      [],
    )

    attach(root, { module: mod })

    const tiles = root.querySelectorAll('[data-test="tile"]')
    expect(tiles.length).toBe(3)
    expect(tiles[0]!.textContent).toBe('one')
    expect(tiles[1]!.textContent).toBe('two')
    expect(tiles[2]!.textContent).toBe('three')
  })

  it('disposes boundaries on attach disposer', () => {
    const html = '<sui-if key="b0" style="display:contents"><p>x</p></sui-if>'
    const root = mountSsr(html, {})
    const disposed = vi.fn()
    const mod = makeModule(
      () => ({
        __b0_attach: (target: Element) => {
          const anchor = document.createComment('if')
          target.parentNode!.replaceChild(anchor, target)
          // Synchronous bindIf-shaped disposer (no real effect).
          return disposed
        },
      }),
      [],
    )

    const dispose = attach(root, { module: mod })
    expect(disposed).not.toHaveBeenCalled()
    dispose()
    expect(disposed).toHaveBeenCalledTimes(1)
  })

  it('logs error if SSR\'d boundary has no matching handler', () => {
    const html = '<sui-if key="b9" style="display:contents"></sui-if>'
    const root = mountSsr(html, {})
    const errSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
    const mod = makeModule(() => ({}), [])
    attach(root, { module: mod })
    expect(
      errSpy.mock.calls.some((c) =>
        (c[0] as string).includes('no boundary handler for __b9_attach'),
      ),
    ).toBe(true)
  })

  it('flips {#if} branch when signal changes (bindIf reactivity)', async () => {
    const html = '<sui-if key="b0" style="display:contents"><p>ssr</p></sui-if>'
    const root = mountSsr(html, { tab: 'a' })

    let tabSig: Signal<string> | undefined
    const mod = makeModule(
      (ctx: SetupContext) => {
        tabSig = ctx.scope!.tab as unknown as Signal<string>
        const tab = tabSig
        return {
          __b0_attach: (target: Element) => {
            const anchor = document.createComment('if')
            target.parentNode!.replaceChild(anchor, target)
            return bindIf({
              anchor,
              test: () => (tab!() === 'a' ? 0 : 1),
              branches: [
                () => {
                  const p = document.createElement('p')
                  p.textContent = 'A'
                  return { nodes: [p] }
                },
                () => {
                  const p = document.createElement('p')
                  p.textContent = 'B'
                  return { nodes: [p] }
                },
              ],
            })
          },
        }
      },
      [],
    )

    attach(root, { module: mod })
    expect(root.querySelector('p')?.textContent).toBe('A')
    tabSig!.set('b')
    expect(root.querySelector('p')?.textContent).toBe('B')
  })
})

describe('attach/boundaries — manifest decoupling', () => {
  it('does not require {#if|for|match|await} entries in the manifest', () => {
    // The compiler drops these kinds from the manifest. attach() must
    // dispatch via setup-return handlers, not via manifest lookup.
    const html = '<sui-if key="b0" style="display:contents"><p>x</p></sui-if>'
    const root = mountSsr(html, {})
    let calls = 0
    const mod = makeModule(
      () => ({
        __b0_attach: (target: Element) => {
          calls++
          const anchor = document.createComment('if')
          target.parentNode!.replaceChild(anchor, target)
          return () => {}
        },
      }),
      [], // empty manifest!
    )
    attach(root, { module: mod })
    expect(calls).toBe(1)
  })
})

void signal
