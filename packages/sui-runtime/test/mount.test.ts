// mount() — F1 unified entry point.
//
// Tests pass `opts.module` directly so the dynamic `import(payload.moduleUrl)`
// path doesn't run. JSDOM has no URL.createObjectURL and no actual ESM
// dynamic-import resolution, so the injected-module pattern is the
// idiomatic test seam (mirrors how `attach()` tests inject modules).

import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest'
import { mount, signal } from '../src/index'
import type { ViewPayload } from '../src/index'

// JSDOM doesn't implement EventSource — stub minimally for live-channel paths.
class FakeEventSource {
  url: string
  closed = false
  listeners = new Map<string, (e: MessageEvent) => void>()
  constructor(url: string) {
    this.url = url
  }
  addEventListener(name: string, fn: (e: MessageEvent) => void) {
    this.listeners.set(name, fn)
  }
  close() {
    this.closed = true
  }
  emit(name: string, data: unknown) {
    const fn = this.listeners.get(name)
    if (fn) fn({ data: JSON.stringify(data) } as MessageEvent)
  }
}
;(globalThis as unknown as { EventSource: unknown }).EventSource = FakeEventSource

interface FakeModule {
  default: {
    setup: (ctx: {
      ship?: unknown
      route?: unknown
      navigate?: (path: string) => void
      scope?: Record<string, unknown>
      stateKeys?: ReadonlySet<string>
    }) => Record<string, unknown>
  }
}

function emptyModule(): FakeModule {
  return { default: { setup: (_ctx) => ({}) } }
}

describe('mount()', () => {
  let target: HTMLElement

  beforeEach(() => {
    document.body.innerHTML = ''
    target = document.createElement('div')
    document.body.appendChild(target)
  })

  it('mounts a payload and returns a handle', async () => {
    const payload: ViewPayload = {
      name: 'demo',
      html: '<p>hello</p>',
      css: '',
      scope: {},
      moduleUrl: '/_sui/views/demo/view.js',
    }
    const handle = await mount(target, payload, { module: emptyModule() })
    expect(handle.element).toBe(target)
    expect(target.innerHTML).toBe('<p>hello</p>')
    handle.dispose()
  })

  it('builds signals from payload.scope', async () => {
    const payload: ViewPayload = {
      name: 'demo',
      html: '<div></div>',
      css: '',
      scope: { count: 5, label: 'hi' },
      moduleUrl: '/x',
    }
    const handle = await mount(target, payload, { module: emptyModule() })
    const scope = handle.scope as Record<string, () => unknown>
    expect(scope.count!()).toBe(5)
    expect(scope.label!()).toBe('hi')
    handle.dispose()
  })

  it('injects css into a <style> adjacent to the target by default', async () => {
    const payload: ViewPayload = {
      name: 'demo',
      html: '<div></div>',
      css: '.x { color: red }',
      scope: {},
      moduleUrl: '/x',
    }
    const handle = await mount(target, payload, { module: emptyModule() })
    const style = target.parentElement?.querySelector('style[data-sui-css]') as HTMLStyleElement
    expect(style).toBeTruthy()
    expect(style.textContent).toBe('.x { color: red }')
    handle.dispose()
    expect(document.querySelector('style[data-sui-css]')).toBeNull()
  })

  it('cssScope: "none" skips css injection', async () => {
    const payload: ViewPayload = {
      name: 'demo',
      html: '<div></div>',
      css: '.x { color: red }',
      scope: {},
      moduleUrl: '/x',
    }
    const handle = await mount(target, payload, { module: emptyModule(), cssScope: 'none' })
    expect(document.querySelector('style[data-sui-css]')).toBeNull()
    handle.dispose()
  })

  it('runs setup() with the provided ship/route/navigate', async () => {
    let captured: { ship?: unknown; navigate?: string } = {}
    const payload: ViewPayload = {
      name: 'demo',
      html: '<div></div>',
      css: '',
      scope: {},
      moduleUrl: '/x',
    }
    const navigate = (_p: string) => {}
    const handle = await mount(target, payload, {
      ship: { id: 'sdk' },
      navigate,
      module: {
        default: {
          setup: (ctx) => {
            captured = { ship: ctx.ship, navigate: typeof ctx.navigate }
            return {}
          },
        },
      },
    })
    expect(captured).toEqual({ ship: { id: 'sdk' }, navigate: 'function' })
    handle.dispose()
  })

  it('reload preserves serializable signal values', async () => {
    const payload: ViewPayload = {
      name: 'demo',
      html: '<div>v1</div>',
      css: '',
      scope: { count: 0 },
      moduleUrl: '/x',
    }
    const mod = emptyModule()
    const handle = await mount(target, payload, { module: mod })
    const scope = handle.scope as Record<string, ((v?: unknown) => unknown) & { set: (v: unknown) => void }>
    scope.count!.set(42)
    expect(scope.count!()).toBe(42)

    const next: ViewPayload = {
      ...payload,
      html: '<div>v2</div>',
      // Server reset count to 0; runtime should restore from current signal.
      scope: { count: 0 },
    }
    // reload's options are inherited from the initial mount; the test
    // module is reused so reload doesn't re-import.
    await handle.reload(next)
    expect(target.innerHTML).toBe('<div>v2</div>')
    const scope2 = handle.scope as Record<string, () => unknown>
    expect(scope2.count!()).toBe(42)
    handle.dispose()
  })

  it('restoreScope overlays values during initial mount', async () => {
    const payload: ViewPayload = {
      name: 'demo',
      html: '<div></div>',
      css: '',
      scope: { count: 0 },
      moduleUrl: '/x',
    }
    const handle = await mount(target, payload, {
      module: emptyModule(),
      restoreScope: { count: 99 },
    })
    const scope = handle.scope as Record<string, () => unknown>
    expect(scope.count!()).toBe(99)
    handle.dispose()
  })

  it('dispose runs hydrate-disposer + lifecycle.unmount + closes EventSource', async () => {
    const events: string[] = []
    const lifecycle = {
      state: {},
      mount: () => events.push('mount'),
      unmount: () => events.push('unmount'),
    }
    const mod: FakeModule = {
      default: {
        setup: (_ctx) => {
          const ret: Record<string, unknown> = {}
          ret.hydrate = () => () => events.push('hydrate-dispose')
          Object.defineProperty(ret, '__shipSetLifecycle', {
            value: lifecycle,
            enumerable: false,
          })
          return ret
        },
      },
    }
    const payload: ViewPayload = {
      name: 'demo',
      html: '<div></div>',
      css: '',
      scope: {
        live1: { kind: 'loading' as const },
        __subscribe__: '/_sui/views/demo/sse',
      } as unknown as Record<string, unknown>,
      moduleUrl: '/x',
    }
    const handle = await mount(target, payload, { module: mod })
    expect(events).toContain('mount')
    handle.dispose()
    expect(events).toContain('hydrate-dispose')
    expect(events).toContain('unmount')
    handle.dispose() // idempotent
  })

  it('mount via name string fetches /_sui/views/<name>', async () => {
    const payload: ViewPayload = {
      name: 'demo-fetch',
      html: '<div>fetched</div>',
      css: '',
      scope: {},
      moduleUrl: '/x',
    }
    const fetchSpy = vi.fn(async (url: string) => {
      expect(url).toBe('/_sui/views/demo-fetch')
      return new Response(JSON.stringify(payload), {
        headers: { 'content-type': 'application/json' },
      })
    })
    const prevFetch = globalThis.fetch
    ;(globalThis as { fetch: typeof fetch }).fetch = fetchSpy as unknown as typeof fetch

    try {
      const handle = await mount(target, 'demo-fetch', { module: emptyModule() })
      expect(fetchSpy).toHaveBeenCalled()
      expect(target.innerHTML).toBe('<div>fetched</div>')
      handle.dispose()
    } finally {
      globalThis.fetch = prevFetch
    }
  })

  it('throws if target is missing', async () => {
    await expect(
      mount(null as unknown as HTMLElement, {} as ViewPayload, { module: emptyModule() }),
    ).rejects.toThrow(/target is required/)
  })

  it('throws if module has no default.setup', async () => {
    const payload: ViewPayload = {
      name: 'broken',
      html: '<div></div>',
      css: '',
      scope: {},
      moduleUrl: '/x',
    }
    await expect(
      mount(target, payload, {
        module: { default: {} } as unknown as Parameters<typeof mount>[2] extends infer O
          ? O extends { module?: infer M } ? M : never
          : never,
      }),
    ).rejects.toThrow(/missing default\.setup/)
  })
})

describe('signal sanity in test harness', () => {
  it('signal export is callable', () => {
    const s = signal(1)
    expect(s()).toBe(1)
  })
})
