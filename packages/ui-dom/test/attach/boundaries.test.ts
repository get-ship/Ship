import { afterEach, describe, expect, it, vi } from 'vitest'
import type { Signal } from '@ship/ui-runtime'
import { attach } from '../../src/attach'
import { clearContainers, flush, makeModule, mountSsr } from './helpers'

afterEach(() => {
  clearContainers()
  vi.restoreAllMocks()
})

describe('attach/boundaries', () => {
  it('does not crash on placeholder boundary; logs error on flip without viewName', () => {
    // Placeholder shape: server skipped this branch. Phase E-2 attempts
    // a swap on signal change; without `viewName` the runtime can't
    // build the fragment-render URL, so it logs an error instead.
    const html = '<sui-if key="b0" hidden></sui-if>'
    const root = mountSsr(html, { ready: false })
    let ready: Signal<boolean> | undefined
    const mod = makeModule(
      (ctx) => {
        ready = ctx.scope!.ready as unknown as Signal<boolean>
        return {}
      },
      [{ key: 'b0', kind: 'if', expr: 'ready()', deps: ['ready'] }],
    )
    const errSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
    const dispose = attach(root, { module: mod })
    expect(errSpy).not.toHaveBeenCalled()
    ready!.set(true)
    expect(
      errSpy.mock.calls.some((c) =>
        (c[0] as string).includes('cannot swap control-flow boundary'),
      ),
    ).toBe(true)
    dispose()
  })

  it('wires leaf bindings inside an active branch wrapper', () => {
    // Active branch: server rendered, wrapper is `<sui-if display:contents>`.
    const html =
      '<sui-if key="b0" style="display:contents">' +
      '<p><!--sui:text:b1-->ada<!--/sui--></p>' +
      '</sui-if>'
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

  it('boundary_swap_on_signal_change: POSTs scope + replaces children + wires new leaves', async () => {
    const html = '<sui-if key="b0" hidden></sui-if>'
    const root = mountSsr(html, { ready: false, name: 'ada' })
    let ready: Signal<boolean> | undefined
    let nameSig: Signal<string> | undefined
    const mod = makeModule(
      (ctx) => {
        ready = ctx.scope!.ready as unknown as Signal<boolean>
        nameSig = ctx.scope!.name as unknown as Signal<string>
        return {}
      },
      [
        { key: 'b0', kind: 'if', expr: 'ready()', deps: ['ready'] },
        { key: 'b1', kind: 'text', expr: 'name()', deps: ['name'] },
      ],
    )

    const fetchMock = vi.fn().mockResolvedValueOnce({
      ok: true,
      status: 200,
      json: async () => ({
        html: '<p data-test="ok"><!--sui:text:b1-->ada<!--/sui--></p>',
      }),
    })
    vi.stubGlobal('fetch', fetchMock)

    const dispose = attach(root, { module: mod, viewName: 'fixture' })
    expect(fetchMock).not.toHaveBeenCalled()

    ready!.set(true)
    await flush()
    await flush()

    expect(fetchMock).toHaveBeenCalledTimes(1)
    const [url, init] = fetchMock.mock.calls[0]!
    expect(url).toBe('/_ship-set/render-fragment/fixture')
    expect((init as RequestInit).method).toBe('POST')
    const body = JSON.parse((init as RequestInit).body as string) as {
      init: Record<string, unknown>
      key: string
    }
    expect(body.key).toBe('b0')
    expect(body.init.ready).toBe(true)
    expect(body.init.name).toBe('ada')

    // The placeholder element has been replaced by an active wrapper.
    const wrapper = root.querySelector('sui-if[key="b0"]')
    expect(wrapper).not.toBeNull()
    expect(wrapper!.tagName.toLowerCase()).toBe('sui-if')
    const p = wrapper!.querySelector('p[data-test="ok"]')
    expect(p).not.toBeNull()
    expect(p!.textContent).toBe('ada')

    // Leaf binding inside the freshly mounted subtree is wired.
    nameSig!.set('grace')
    expect(p!.textContent).toBe('grace')

    dispose()
  })

  it('boundary_swap_disposes_previous_listeners: second swap tears down first', async () => {
    const html = '<sui-if key="b0" hidden></sui-if>'
    const root = mountSsr(html, { ready: false, name: 'ada' })
    let ready: Signal<boolean> | undefined
    let nameSig: Signal<string> | undefined
    const mod = makeModule(
      (ctx) => {
        ready = ctx.scope!.ready as unknown as Signal<boolean>
        nameSig = ctx.scope!.name as unknown as Signal<string>
        return {}
      },
      [
        { key: 'b0', kind: 'if', expr: 'ready()', deps: ['ready'] },
        { key: 'b1', kind: 'text', expr: 'name()', deps: ['name'] },
      ],
    )

    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce({
        ok: true,
        status: 200,
        json: async () => ({
          html: '<p id="first"><!--sui:text:b1-->ada<!--/sui--></p>',
        }),
      })
      .mockResolvedValueOnce({
        ok: true,
        status: 200,
        json: async () => ({ html: '' }),
      })
    vi.stubGlobal('fetch', fetchMock)

    const dispose = attach(root, { module: mod, viewName: 'fixture' })

    ready!.set(true)
    await flush()
    await flush()

    const firstP = root.querySelector('#first') as HTMLElement | null
    expect(firstP).not.toBeNull()
    expect(firstP!.textContent).toBe('ada')

    // Second flip: empty body → wrapper becomes empty active div, the
    // text-binding effect for `name` from the first swap is disposed.
    ready!.set(false)
    await flush()
    await flush()

    expect(fetchMock).toHaveBeenCalledTimes(2)
    // Old <p> is gone after second swap.
    expect(root.querySelector('#first')).toBeNull()
    // First-swap's text effect must not throw / write to the detached
    // node when its dep changes after disposal.
    nameSig!.set('grace')
    // No assertion needed beyond "doesn't crash" — the detached node
    // was removed and its effect should have been torn down.

    dispose()
  })

  it('boundary_swap_failure_logs_and_keeps_existing: 500 response leaves DOM in place', async () => {
    const html =
      '<sui-if key="b0" style="display:contents">' +
      '<p data-test="initial">existing</p>' +
      '</sui-if>'
    const root = mountSsr(html, { ready: true })
    let ready: Signal<boolean> | undefined
    const mod = makeModule(
      (ctx) => {
        ready = ctx.scope!.ready as unknown as Signal<boolean>
        return {}
      },
      [{ key: 'b0', kind: 'if', expr: 'ready()', deps: ['ready'] }],
    )

    const fetchMock = vi.fn().mockResolvedValueOnce({
      ok: false,
      status: 500,
      text: async () => 'kaboom',
    })
    vi.stubGlobal('fetch', fetchMock)
    const errSpy = vi.spyOn(console, 'error').mockImplementation(() => {})

    const dispose = attach(root, { module: mod, viewName: 'fixture' })

    ready!.set(false)
    await flush()
    await flush()

    expect(fetchMock).toHaveBeenCalledTimes(1)
    expect(
      errSpy.mock.calls.some((c) =>
        (c[0] as string).includes('fragment render failed'),
      ),
    ).toBe(true)
    // Existing DOM unchanged.
    const p = root.querySelector('p[data-test="initial"]')
    expect(p).not.toBeNull()
    expect(p!.textContent).toBe('existing')

    dispose()
  })
})
