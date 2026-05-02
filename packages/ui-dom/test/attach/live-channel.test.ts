// JSDOM smoke test: openLiveChannel wires patch-signals SSE events to LiveSignals.
//
// We can't spawn a real SSE server in vitest, but we can exercise the
// EventSource event dispatch path directly using JSDOM's EventSource stub +
// manual event dispatch.

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { attach } from '../../src/attach'
import type { SetCompiledModule } from '../../src/attach'
import { clearContainers } from './helpers'

afterEach(clearContainers)

// ------------------------------------------------------------------
// Minimal EventSource stub — vitest/JSDOM doesn't ship one.
// ------------------------------------------------------------------

type EventListener = (event: MessageEvent) => void

interface StubEventSource {
  addEventListener(type: string, fn: EventListener): void
  close(): void
  dispatch(type: string, data: string): void
  closed: boolean
}

function makeEventSourceStub(): StubEventSource {
  const listeners: Map<string, EventListener[]> = new Map()
  let closed = false
  return {
    get closed() { return closed },
    addEventListener(type: string, fn: EventListener) {
      const arr = listeners.get(type) ?? []
      arr.push(fn)
      listeners.set(type, arr)
    },
    close() { closed = true },
    dispatch(type: string, data: string) {
      const ev = new MessageEvent(type, { data })
      for (const fn of listeners.get(type) ?? []) fn(ev)
    },
  }
}

describe('attach / live channel (patch-signals routing)', () => {
  let esStub: StubEventSource
  let esConstructorCalls: string[]

  beforeEach(() => {
    esConstructorCalls = []
    esStub = makeEventSourceStub()
    const stub = esStub
    const calls = esConstructorCalls
    // Returning an object from a constructor makes `new` use that object as
    // the result, so `new FakeEventSource(url)` evaluates to `stub`.
    function FakeEventSource(this: unknown, url: string): StubEventSource {
      calls.push(url)
      return stub
    }
    vi.stubGlobal('EventSource', FakeEventSource)
  })

  afterEach(() => {
    vi.unstubAllGlobals()
  })

  function mountWithLiveData(initJson: Record<string, unknown>): HTMLElement {
    const c = document.createElement('div')
    document.body.appendChild(c)
    const islandJson = JSON.stringify(initJson).replace(/<\//g, '<\\/')
    c.innerHTML = `<p>hi</p><script type="application/json" id="__set_init__">${islandJson}</script>`
    return c
  }

  function noopModule(): SetCompiledModule {
    return {
      default: { setup: () => ({}) },
      bindings: [],
    }
  }

  it('opens an EventSource on the subscribe URL when live data is present', () => {
    const root = mountWithLiveData({
      overview: { kind: 'loading' },
      __subscribe__: '/_sui/views/git/sse',
    })
    attach(root, { module: noopModule() })
    expect(esConstructorCalls).toHaveLength(1)
    expect(esConstructorCalls[0]).toBe('/_sui/views/git/sse')
  })

  it('does NOT open EventSource when no subscribe URL is present', () => {
    const root = mountWithLiveData({ n: 0 })
    attach(root, { module: noopModule() })
    expect(esConstructorCalls).toHaveLength(0)
  })

  it('opens EventSource even when liveKeys is empty so patch-elements can flow', () => {
    // Phase 0.5: the SSE channel is shared with `patch-elements` events
    // pushed by agents/SDK. Open it whenever the compiler emits a
    // subscribe URL, regardless of whether the view declares live data.
    const root = mountWithLiveData({
      count: 0,
      __subscribe__: '/_sui/views/git/sse',
    })
    attach(root, { module: noopModule() })
    expect(esConstructorCalls).toHaveLength(1)
    expect(esConstructorCalls[0]).toBe('/_sui/views/git/sse')
  })

  it('patch-signals event updates the matching LiveSignal', () => {
    const root = mountWithLiveData({
      overview: { kind: 'loading' },
      __subscribe__: '/_sui/views/git/sse',
    })

    // Capture the scope signal that attach wires into openLiveChannel.
    let overviewSig: (() => unknown) | undefined
    const captureModule: SetCompiledModule = {
      default: {
        setup: (ctx) => {
          overviewSig = ctx.scope?.['overview'] as (() => unknown) | undefined
          return {}
        },
      },
      bindings: [],
    }
    attach(root, { module: captureModule })

    expect(overviewSig).toBeDefined()
    expect(overviewSig!()).toEqual({ kind: 'loading' })

    // Fire a patch-signals event simulating the daemon pushing an ok value.
    esStub.dispatch('patch-signals', JSON.stringify({
      name: 'overview',
      value: { kind: 'ok', value: { branch: 'main', clean: true } },
    }))

    expect(overviewSig!()).toEqual({ kind: 'ok', value: { branch: 'main', clean: true } })
  })

  it('ignores patch-signals events for unknown data names', () => {
    const root = mountWithLiveData({
      overview: { kind: 'loading' },
      __subscribe__: '/_sui/views/git/sse',
    })
    attach(root, { module: noopModule() })
    // Should not throw for unknown names.
    expect(() => {
      esStub.dispatch('patch-signals', JSON.stringify({
        name: 'unknown-key',
        value: { kind: 'ok', value: {} },
      }))
    }).not.toThrow()
  })

  it('dispose closes the EventSource', () => {
    const root = mountWithLiveData({
      overview: { kind: 'loading' },
      __subscribe__: '/_sui/views/git/sse',
    })
    const dispose = attach(root, { module: noopModule() })
    expect(esStub.closed).toBe(false)
    dispose()
    expect(esStub.closed).toBe(true)
  })
})
