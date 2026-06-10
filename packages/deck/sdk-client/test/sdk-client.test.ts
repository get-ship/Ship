import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { live, type LiveSignal, type LiveState } from '@ship/sui-runtime'
import { createClient } from '../src/index'
import { _resetSharedSourceCache } from '../src/shared-source'

class FakeEventSource {
  static instances: FakeEventSource[] = []
  url: string
  closed = false
  listeners: Record<string, Array<(ev: any) => void>> = {}
  constructor(url: string) {
    this.url = url
    FakeEventSource.instances.push(this)
  }
  addEventListener(name: string, cb: (ev: any) => void): void {
    ;(this.listeners[name] ||= []).push(cb)
  }
  removeEventListener(name: string, cb: (ev: any) => void): void {
    this.listeners[name] = (this.listeners[name] ?? []).filter((x) => x !== cb)
  }
  close(): void {
    this.closed = true
  }
  /** Emit a named SSE event with a raw `data` string. */
  emitEvent(eventName: string, data: string): void {
    for (const cb of this.listeners[eventName] ?? []) {
      cb({ data })
    }
  }
  emitPatch(payload: object): void {
    this.emitEvent('patch', JSON.stringify(payload))
  }
  emitError(): void {
    for (const cb of this.listeners['error'] ?? []) cb({})
  }
}

// ── Mux fetch mock helpers ────────────────────────────────────────────────────

interface MuxFetchCall {
  url: string
  body: unknown
}

interface MuxFetchMock {
  calls: MuxFetchCall[]
  mockFetch: typeof globalThis.fetch
}

function makeMuxFetch(opts?: {
  initialValue?: unknown
  subId?: string
  openStatus?: number
  subscribeStatus?: number
}): MuxFetchMock {
  const subId = opts?.subId ?? 'sub.test'
  const initialValue = opts?.initialValue ?? {}
  const calls: MuxFetchCall[] = []

  const mockFetch = async (url: any, init?: any): Promise<Response> => {
    const u = String(url)
    let body: unknown
    try {
      body = init?.body ? JSON.parse(init.body as string) : undefined
    } catch {
      body = init?.body
    }
    calls.push({ url: u, body })

    if (u.includes('/_deck/mux/open')) {
      const status = opts?.openStatus ?? 200
      if (status !== 200) {
        return new Response('error', { status })
      }
      return new Response(
        JSON.stringify({
          window_id: 'win.test',
          session_id: 'sess.test',
          heartbeat_ms: 15000,
        }),
        { status: 200, headers: { 'content-type': 'application/json' } },
      )
    }
    if (u.includes('/_deck/mux/subscribe')) {
      const status = opts?.subscribeStatus ?? 200
      if (status !== 200) {
        return new Response('error', { status })
      }
      return new Response(
        JSON.stringify({ subscription_id: subId, initial_value: initialValue }),
        { status: 200, headers: { 'content-type': 'application/json' } },
      )
    }
    if (u.includes('/_deck/mux/unsubscribe')) {
      return new Response(JSON.stringify({ removed: true }), {
        status: 200,
        headers: { 'content-type': 'application/json' },
      })
    }
    // Passthrough for non-mux fetch (mutations, artifacts, etc.)
    throw new Error(`Unexpected mux-mock fetch: ${u}`)
  }

  return { calls, mockFetch: mockFetch as typeof globalThis.fetch }
}

/** Flush the microtask queue + one macrotask tick so async fetch chains settle. */
const flush = (): Promise<void> => new Promise((r) => setTimeout(r, 0))

// ── Fake signal (out of a live() scope, for direct mux tests) ─────────────────

function makeSig<T>(): LiveSignal<T> {
  let v: LiveState<T> = { kind: 'loading' }
  const fn = (() => v) as LiveSignal<T>
  fn.set = (next: LiveState<T> | ((prev: LiveState<T>) => LiveState<T>)) => {
    v = typeof next === 'function' ? (next as (p: LiveState<T>) => LiveState<T>)(v) : next
  }
  fn.__subs = new Set()
  fn.__peek = () => v
  return fn
}

// ── Fake Connect client (used by mutation tests only) ─────────────────────────

class FakeConnectClient {
  private calls: Array<{ domain: string; op: string; req: unknown }> = []
  private responses: Map<string, unknown> = new Map()

  setResponse(key: string, response: unknown): void {
    this.responses.set(key, response)
  }

  getCalls(): Array<{ domain: string; op: string; req: unknown }> {
    return this.calls
  }

  build(): any {
    return new Proxy(
      {},
      {
        get: (_target, domain: string | symbol) => {
          if (typeof domain !== 'string') return undefined
          return new Proxy(
            {},
            {
              get: (_t, op: string | symbol) => {
                if (typeof op !== 'string') return undefined
                return (req: any) => {
                  this.calls.push({ domain, op, req })
                  const key = `${domain}.${op}`
                  const resp = this.responses.get(key) ?? { id: 'test-id' }
                  return Promise.resolve({
                    ...(resp as object),
                    json: (resp as any).json ?? JSON.stringify(resp),
                  })
                }
              },
            },
          )
        },
      },
    )
  }
}

// ── Mux live-path tests ───────────────────────────────────────────────────────

describe('mux live path', () => {
  let origFetch: typeof globalThis.fetch

  beforeEach(() => {
    FakeEventSource.instances = []
    _resetSharedSourceCache()
    origFetch = globalThis.fetch
  })

  afterEach(() => {
    globalThis.fetch = origFetch
    _resetSharedSourceCache()
  })

  it('agents.summaries opens mux window + subscribe and resolves on initial_value', async () => {
    const { mockFetch, calls } = makeMuxFetch({
      initialValue: { items: [{ id: 'a', name: 'A' }] },
      subId: 'sub.agents',
    })
    globalThis.fetch = mockFetch

    const ship = createClient({ EventSource: FakeEventSource as any })
    const sig = live(() => ship.agents.summaries())
    expect(sig().kind).toBe('loading')

    await flush()

    // Verify open → subscribe sequence
    const openCall = calls.find((c) => c.url.includes('/mux/open'))
    expect(openCall).toBeDefined()
    const subCall = calls.find((c) => c.url.includes('/mux/subscribe'))
    expect(subCall).toBeDefined()
    expect(subCall!.body).toMatchObject({ domain: 'agents', op: 'summaries' })

    const v = sig()
    expect(v.kind).toBe('ok')
    if (v.kind === 'ok') {
      expect((v.value as { items: unknown[] }).items).toHaveLength(1)
    }
  })

  it('mux-data event routes a full-value frame to the matching signal', async () => {
    const { mockFetch } = makeMuxFetch({
      initialValue: { items: [] },
      subId: 'sub.data',
    })
    globalThis.fetch = mockFetch

    const sig = makeSig<{ items: Array<{ id: string }> }>()
    const ship = createClient({ EventSource: FakeEventSource as any })
    const src = ship.agents.summaries()
    src.subscribe(sig)

    await flush()

    // Confirm EventSource was opened for the mux stream
    const es = FakeEventSource.instances.find((e) => e.url.includes('/mux/stream/'))
    expect(es).toBeDefined()

    // Emit a mux-data frame for this subscription
    es!.emitEvent('mux-data', JSON.stringify({ sub: 'sub.data', value: { items: [{ id: 'b' }] }, version: 2 }))

    const v = sig()
    expect(v.kind).toBe('ok')
    if (v.kind === 'ok') {
      expect(v.value.items.map((x) => x.id)).toEqual(['b'])
    }
  })

  it('teardown calls mux/unsubscribe and closes stream on last subscriber', async () => {
    const { mockFetch, calls } = makeMuxFetch({
      initialValue: {},
      subId: 'sub.unsub',
    })
    globalThis.fetch = mockFetch

    const sig = makeSig<unknown>()
    const ship = createClient({ EventSource: FakeEventSource as any })
    const teardown = ship.agents.summaries().subscribe(sig)

    await flush()

    teardown()
    await flush()

    const unsubCall = calls.find((c) => c.url.includes('/mux/unsubscribe'))
    expect(unsubCall).toBeDefined()
    expect(unsubCall!.body).toMatchObject({
      window_id: 'win.test',
      subscription_id: 'sub.unsub',
    })

    // EventSource should be closed when refCount hits 0
    const es = FakeEventSource.instances.find((e) => e.url.includes('/mux/stream/'))
    expect(es?.closed).toBe(true)
  })

  it('mux-close event surfaces an error to all live signals and resets', async () => {
    // Use a counter so each subscribe call gets a distinct sub_id in the mux subs map.
    let subscribeCount = 0
    globalThis.fetch = async (url: any, init?: any) => {
      const u = String(url)
      if (u.includes('/_deck/mux/open')) {
        return new Response(
          JSON.stringify({ window_id: 'win.test', session_id: 'sess.test', heartbeat_ms: 15000 }),
          { status: 200, headers: { 'content-type': 'application/json' } },
        )
      }
      if (u.includes('/_deck/mux/subscribe')) {
        subscribeCount++
        return new Response(
          JSON.stringify({ subscription_id: `sub.close${subscribeCount}`, initial_value: {} }),
          { status: 200, headers: { 'content-type': 'application/json' } },
        )
      }
      if (u.includes('/_deck/mux/unsubscribe')) {
        return new Response(JSON.stringify({ removed: true }), {
          status: 200,
          headers: { 'content-type': 'application/json' },
        })
      }
      throw new Error(`Unexpected fetch: ${u}`)
    }

    const sig1 = makeSig<unknown>()
    const sig2 = makeSig<unknown>()
    const ship = createClient({ EventSource: FakeEventSource as any })

    // Two different ops on the same client — same window, different sub_ids.
    ship.git.overview().subscribe(sig1)
    ship.settings.state().subscribe(sig2)
    await flush()

    const es = FakeEventSource.instances.find((e) => e.url.includes('/mux/stream/'))
    expect(es).toBeDefined()

    es!.emitEvent('mux-close', JSON.stringify({ reason: 'server_restart', message: 'daemon restarted' }))

    // Both signals should be in error state
    expect(sig1().kind).toBe('error')
    expect(sig2().kind).toBe('error')
  })

  it('mux/open failure surfaces as signal error', async () => {
    const { mockFetch } = makeMuxFetch({ openStatus: 503 })
    globalThis.fetch = mockFetch

    const sig = makeSig<unknown>()
    const ship = createClient({ EventSource: FakeEventSource as any })
    ship.agents.summaries().subscribe(sig)

    await flush()

    expect(sig().kind).toBe('error')
    if (sig().kind === 'error') {
      expect((sig() as { kind: 'error'; error: Error }).error.message).toMatch(/mux\/open failed/)
    }
  })

  it('mux/subscribe failure surfaces as signal error', async () => {
    const { mockFetch } = makeMuxFetch({ subscribeStatus: 404 })
    globalThis.fetch = mockFetch

    const sig = makeSig<unknown>()
    const ship = createClient({ EventSource: FakeEventSource as any })
    ship.agents.summaries().subscribe(sig)

    await flush()

    expect(sig().kind).toBe('error')
  })

  it('git.diff passes raw snake_case args to mux/subscribe', async () => {
    const { mockFetch, calls } = makeMuxFetch({ initialValue: {} })
    globalThis.fetch = mockFetch

    const ship = createClient({ EventSource: FakeEventSource as any })
    const sig = live(() => ship.git.diff({ commit: 'abc1234' }))
    expect(sig().kind).toBe('loading')

    await flush()

    const subCall = calls.find((c) => c.url.includes('/mux/subscribe'))
    expect(subCall).toBeDefined()
    expect(subCall!.body).toMatchObject({ domain: 'git', op: 'diff', args: { commit: 'abc1234' } })
  })

  it('git.diff includes workspace_id as-is (no camelCase conversion)', async () => {
    const { mockFetch, calls } = makeMuxFetch({ initialValue: {} })
    globalThis.fetch = mockFetch

    const ship = createClient({ EventSource: FakeEventSource as any })
    live(() => ship.git.diff({ commit: 'WORKING', workspace_id: 'ws-1' }))

    await flush()

    const subCall = calls.find((c) => c.url.includes('/mux/subscribe'))
    expect(subCall!.body).toMatchObject({
      domain: 'git',
      op: 'diff',
      args: { commit: 'WORKING', workspace_id: 'ws-1' },
    })
  })

  it('events.list passes raw filter args to mux/subscribe', async () => {
    const { mockFetch, calls } = makeMuxFetch({ initialValue: {} })
    globalThis.fetch = mockFetch

    const ship = createClient({ EventSource: FakeEventSource as any })
    live(() => ship.events.list({ q: 'foo', event_type: 'job.completed' }))

    await flush()

    const subCall = calls.find((c) => c.url.includes('/mux/subscribe'))
    expect(subCall!.body).toMatchObject({
      domain: 'events',
      op: 'list',
      args: { q: 'foo', event_type: 'job.completed' },
    })
  })

  it('canvas.doc passes slug args to mux/subscribe (not an SSE expr)', async () => {
    const { mockFetch, calls } = makeMuxFetch({ initialValue: {} })
    globalThis.fetch = mockFetch

    const ship = createClient({ EventSource: FakeEventSource as any })
    live(() => ship.canvas.doc({ slug: 'roadmap' }))

    await flush()

    const subCall = calls.find((c) => c.url.includes('/mux/subscribe'))
    expect(subCall).toBeDefined()
    expect(subCall!.body).toMatchObject({ domain: 'canvas', op: 'doc', args: { slug: 'roadmap' } })
    // No /_deck/data?expr= SSE should have been opened
    expect(FakeEventSource.instances.every((e) => !e.url.includes('/data?expr='))).toBe(true)
  })

  it('mesh.snapshot uses mux (not the legacy SSE expr fallback)', async () => {
    const { mockFetch, calls } = makeMuxFetch({ initialValue: {} })
    globalThis.fetch = mockFetch

    const ship = createClient({ EventSource: FakeEventSource as any })
    live(() => ship.mesh.snapshot())

    await flush()

    const subCall = calls.find((c) => c.url.includes('/mux/subscribe'))
    expect(subCall).toBeDefined()
    expect(subCall!.body).toMatchObject({ domain: 'mesh', op: 'snapshot' })
    // No /_deck/data?expr= SSE should have been opened
    expect(FakeEventSource.instances.every((e) => !e.url.includes('/data?expr='))).toBe(true)
  })

  it('no EventSource surfaces an error immediately', () => {
    const sig = makeSig<unknown>()
    // Provide undefined EventSource
    const ship = createClient({ EventSource: undefined })
    ship.agents.summaries().subscribe(sig)
    expect(sig().kind).toBe('error')
  })
})

// ── Mutation tests (Connect client — unaffected by mux) ───────────────────────

describe('mutations via Connect', () => {
  let origFetch: typeof globalThis.fetch

  beforeEach(() => {
    origFetch = globalThis.fetch
  })

  afterEach(() => {
    globalThis.fetch = origFetch
  })

  it('sessions.start POSTs JSON to /api/v1/sessions/spawn', async () => {
    const calls: Array<{ url: string; init?: RequestInit }> = []
    globalThis.fetch = async (url: any, init?: any) => {
      calls.push({ url: String(url), init })
      return new Response(JSON.stringify({ session_id: 'sess-1', attached: true }), {
        status: 200,
        headers: { 'content-type': 'application/json' },
      })
    }
    const ship = createClient({ EventSource: FakeEventSource as any })
    const resp = await ship.sessions.start({ workspace_id: 'ws-1', kind: 'shell' })
    expect(resp).toEqual({ session_id: 'sess-1', attached: true })
    expect(calls[0]!.url).toBe('/api/v1/sessions/spawn')
    expect(calls[0]!.init?.method).toBe('POST')
    expect(JSON.parse(calls[0]!.init?.body as string)).toEqual({
      workspace_id: 'ws-1',
      kind: 'shell',
    })
  })

  it('sessions.end DELETEs against the session id', async () => {
    const calls: Array<{ url: string; init?: RequestInit }> = []
    globalThis.fetch = async (url: any, init?: any) => {
      calls.push({ url: String(url), init })
      return new Response(null, { status: 204 })
    }
    const ship = createClient({ EventSource: FakeEventSource as any })
    await ship.sessions.end({ id: 'sess-2' })
    expect(calls[0]!.url).toBe('/api/v1/sessions/sess-2')
    expect(calls[0]!.init?.method).toBe('DELETE')
  })

  it('mutation errors surface the daemon response body', async () => {
    globalThis.fetch = async () =>
      new Response('{"error":"agent_id required"}', {
        status: 400,
        statusText: 'Bad Request',
      })
    const ship = createClient({ EventSource: FakeEventSource as any })
    await expect(
      ship.sessions.start({ workspace_id: 'ws', kind: 'agent' }),
    ).rejects.toThrow(/agent_id required/)
  })

  it('agents.create calls Connect and returns {ok, id}', async () => {
    const fakeConnect = new FakeConnectClient()
    fakeConnect.setResponse('agents.create', { id: 'foo' })
    const ship = createClient({ connectClient: fakeConnect.build() })
    const doc = { agent: { id: 'foo', name: 'Foo' } }
    const resp = await ship.agents.create({ id: 'foo', doc })
    expect(resp).toEqual({ ok: true, id: 'foo' })
    const calls = fakeConnect.getCalls()
    expect(calls[0]!.domain).toBe('agents')
    expect(calls[0]!.op).toBe('create')
    expect(calls[0]!.req).toEqual({ id: 'foo', docJson: JSON.stringify(doc), scope: '' })
  })

  it('agents.update calls Connect and returns {ok, id}', async () => {
    const fakeConnect = new FakeConnectClient()
    fakeConnect.setResponse('agents.put', { id: 'foo' })
    const ship = createClient({ connectClient: fakeConnect.build() })
    const resp = await ship.agents.update({ id: 'foo', doc: { agent: { id: 'foo' } } })
    expect(resp).toEqual({ ok: true, id: 'foo' })
    const calls = fakeConnect.getCalls()
    expect(calls[0]!.domain).toBe('agents')
    expect(calls[0]!.op).toBe('put')
  })

  it('collections.create_item calls Connect and returns {ok, id}', async () => {
    const fakeConnect = new FakeConnectClient()
    fakeConnect.setResponse('collections.createItem', { id: 'item-1' })
    const ship = createClient({ connectClient: fakeConnect.build() })
    const resp = await ship.collections.create_item({ collection: 'systems', data: { name: 'X' } })
    expect(resp).toEqual({ ok: true, id: 'item-1' })
    const calls = fakeConnect.getCalls()
    expect(calls[0]!.domain).toBe('collections')
    expect(calls[0]!.op).toBe('createItem')
    expect(calls[0]!.req).toEqual({
      collection: 'systems',
      dataJson: JSON.stringify({ name: 'X' }),
      id: '',
    })
  })

  it('collections.update_item and delete_item call Connect', async () => {
    const fakeConnect = new FakeConnectClient()
    fakeConnect.setResponse('collections.updateItem', { id: 'r-1' })
    fakeConnect.setResponse('collections.deleteItem', {})
    const ship = createClient({ connectClient: fakeConnect.build() })
    await ship.collections.update_item({ collection: 'systems', item_id: 'r-1', data: { a: 1 } })
    await ship.collections.delete_item({ collection: 'systems', item_id: 'r-1' })
    const calls = fakeConnect.getCalls()
    expect(calls[0]!.op).toBe('updateItem')
    expect(calls[1]!.op).toBe('deleteItem')
  })

  it('collections.create calls Connect and returns {ok, id}', async () => {
    const fakeConnect = new FakeConnectClient()
    fakeConnect.setResponse('collections.createSchema', { json: JSON.stringify({ ok: true, id: 'x' }) })
    const ship = createClient({ connectClient: fakeConnect.build() })
    const resp = await ship.collections.create({ id: 'x', name: 'X collection' })
    expect(resp).toEqual({ ok: true, id: 'x' })
  })

  it('artifacts.get returns text by default and json with as=json', async () => {
    globalThis.fetch = async (url: any) => {
      const u = String(url)
      if (u.endsWith('/a1')) {
        return new Response('hello world', {
          status: 200,
          headers: { 'content-type': 'text/plain' },
        })
      }
      return new Response(JSON.stringify({ ok: true, value: 42 }), {
        status: 200,
        headers: { 'content-type': 'application/json' },
      })
    }
    const ship = createClient({ EventSource: FakeEventSource as any })
    const text = await ship.artifacts.get({ id: 'a1' })
    expect(text).toBe('hello world')
    const json = await ship.artifacts.get({ id: 'a2', as: 'json' })
    expect(json).toEqual({ ok: true, value: 42 })
  })

  it('artifacts.upload posts multipart form to /api/v1/artifacts', async () => {
    const calls: Array<{ url: string; init?: RequestInit }> = []
    globalThis.fetch = async (url: any, init?: any) => {
      calls.push({ url: String(url), init })
      return new Response(
        JSON.stringify({ ok: true, data: { id: 'a1', name: 'foo.txt', mime: 'text/plain' } }),
        { status: 200, headers: { 'content-type': 'application/json' } },
      )
    }
    const ship = createClient({ EventSource: FakeEventSource as any })
    const file = new File(['hello'], 'foo.txt', { type: 'text/plain' })
    const resp = await ship.artifacts.upload({ file, collection_id: 'systems' })
    expect((resp as { ok: boolean }).ok).toBe(true)
    expect(calls[0]!.url).toBe('/api/v1/artifacts')
    expect(calls[0]!.init?.method).toBe('POST')
    expect(calls[0]!.init?.body).toBeInstanceOf(FormData)
    const fd = calls[0]!.init?.body as FormData
    expect(fd.get('file')).toBeInstanceOf(File)
    expect(fd.get('collection_id')).toBe('systems')
  })
})

// Multi-subscriber fanout coverage (#198) lives in shared-source.test.ts —
// exercises the dedup helper directly with a synthetic LiveSource so it
// doesn't depend on fetch/EventSource internals.
