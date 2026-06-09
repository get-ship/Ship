import { describe, expect, it } from 'vitest'
import { live } from '@ship/sui-runtime'
import { createClient, makeSseSource } from '../src/index'

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
  emitPatch(payload: object): void {
    for (const cb of this.listeners.patch ?? []) {
      cb({ data: JSON.stringify(payload) })
    }
  }
  emitError(): void {
    for (const cb of this.listeners.error ?? []) cb({})
  }
}

// Fake Connect client for testing. Captures RPC calls and allows tests
// to emit frames and responses.
class FakeConnectClient {
  private calls: Array<{ domain: string; op: string; req: unknown }> = []
  private frameQueues: Map<string, { json: string }[]> = new Map()
  private resolvers: Map<string, () => void> = new Map()
  private responses: Map<string, unknown> = new Map()

  setResponse(key: string, response: unknown): void {
    this.responses.set(key, response)
  }

  emitFrame(key: string, frame: object): void {
    if (!this.frameQueues.has(key)) {
      this.frameQueues.set(key, [])
    }
    this.frameQueues.get(key)!.push({ json: JSON.stringify(frame) })
    // Wake up any waiting iterators
    const resolver = this.resolvers.get(key)
    if (resolver) {
      resolver()
    }
  }

  getCalls(): Array<{ domain: string; op: string; req: unknown }> {
    return this.calls
  }

  clearCalls(): void {
    this.calls = []
  }

  // Build a Proxy-based fake that intercepts domain.op calls
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
                // Return a function that handles both live reads and mutations
                return (req: any, opts?: { signal?: AbortSignal }) => {
                  this.calls.push({ domain, op, req })
                  const key = `${domain}.${op}`

                  // If it's a Live op (async iterable), return a generator
                  if (op.endsWith('Live')) {
                    const self = this
                    return {
                      async *[Symbol.asyncIterator]() {
                        if (!self.frameQueues.has(key)) {
                          self.frameQueues.set(key, [])
                        }
                        const queue = self.frameQueues.get(key)!
                        let yieldedCount = 0

                        while (!opts?.signal?.aborted) {
                          if (yieldedCount < queue.length) {
                            yield queue[yieldedCount]!
                            yieldedCount++
                          } else {
                            // Wait for more frames or signal abort
                            const signalAborted = new Promise<void>((resolve) => {
                              opts?.signal?.addEventListener('abort', () => resolve())
                            })
                            const frameAdded = new Promise<void>((resolve) => {
                              self.resolvers.set(key, () => resolve())
                            })
                            await Promise.race([signalAborted, frameAdded])
                            self.resolvers.delete(key)
                            // After waiting, loop will check again for new frames
                          }
                        }
                      },
                    }
                  }

                  // Otherwise it's a mutation, return a promise
                  const resp = this.responses.get(key) || { id: 'test-id' }
                  // Ensure json field exists for okJson calls
                  return Promise.resolve({
                    ...resp,
                    json: resp.json || JSON.stringify(resp),
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

function createFakeConnectClient() {
  return new FakeConnectClient()
}

describe('createClient', () => {
  it('agents.summaries opens a Connect stream and resolves on first frame', async () => {
    const fakeConnect = createFakeConnectClient()
    const ship = createClient({ connectClient: fakeConnect.build() })
    const sig = live(() => ship.agents.summaries())
    expect(sig().kind).toBe('loading')
    const calls = fakeConnect.getCalls()
    expect(calls).toHaveLength(1)
    expect(calls[0]!.domain).toBe('agents')
    expect(calls[0]!.op).toBe('summariesLive')
    fakeConnect.emitFrame('agents.summariesLive', {
      items: [{ id: 'a', name: 'A' }],
    })
    // Wait for async operations to complete
    await new Promise((resolve) => setTimeout(resolve, 0))
    const v = sig()
    expect(v.kind).toBe('ok')
    if (v.kind === 'ok') {
      expect(v.value.items).toHaveLength(1)
      expect(v.value.items[0]!.id).toBe('a')
    }
  })

  it('subsequent patches mutate via applyPatch', () => {
    FakeEventSource.instances = []
    const source = makeSseSource<{ items: { id: string }[] }>(
      'http://test/_deck/data?expr=ship.agents.summaries%28%29',
      FakeEventSource as any,
    )
    const sig = live(() => source)
    const es = FakeEventSource.instances[0]!
    es.emitPatch({ op: 'set', value: { items: [{ id: 'a' }] } })
    es.emitPatch({ op: 'append', path: 'items', value: { id: 'b' } })
    const v = sig()
    expect(v.kind).toBe('ok')
    if (v.kind === 'ok') {
      expect(v.value.items.map((x) => x.id)).toEqual(['a', 'b'])
    }
  })

  it('connection errors before first frame surface as error state', () => {
    FakeEventSource.instances = []
    const source = makeSseSource('http://test', FakeEventSource as any)
    const sig = live(() => source)
    FakeEventSource.instances[0]!.emitError()
    const v = sig()
    expect(v.kind).toBe('error')
  })

  it('git.diff passes args to Connect as camelCase', () => {
    const fakeConnect = createFakeConnectClient()
    const ship = createClient({ connectClient: fakeConnect.build() })
    const sig = live(() => ship.git.diff({ commit: 'abc1234' }))
    expect(sig().kind).toBe('loading')
    const calls = fakeConnect.getCalls()
    expect(calls).toHaveLength(1)
    expect(calls[0]!.domain).toBe('git')
    expect(calls[0]!.op).toBe('diffLive')
    // Args should be camelCased
    expect(calls[0]!.req).toEqual({ commit: 'abc1234' })
  })

  it('git.diff includes optional workspace_id when provided', () => {
    const fakeConnect = createFakeConnectClient()
    const ship = createClient({ connectClient: fakeConnect.build() })
    live(() => ship.git.diff({ commit: 'WORKING', workspace_id: 'ws-1' }))
    const calls = fakeConnect.getCalls()
    expect(calls).toHaveLength(1)
    // workspace_id should be camelCased to workspaceId
    expect(calls[0]!.req).toEqual({ commit: 'WORKING', workspaceId: 'ws-1' })
  })

  it('events.list serializes filter args via Connect with camelCase', () => {
    const fakeConnect = createFakeConnectClient()
    const ship = createClient({ connectClient: fakeConnect.build() })
    live(() => ship.events.list({ q: 'foo', event_type: 'job.completed' }))
    const calls = fakeConnect.getCalls()
    expect(calls).toHaveLength(1)
    // event_type should be camelCased to eventType
    expect(calls[0]!.req).toEqual({ q: 'foo', eventType: 'job.completed' })
  })

  it('canvas.doc serializes slug args into the expr', () => {
    FakeEventSource.instances = []
    const ship = createClient({ EventSource: FakeEventSource as any })
    live(() => ship.canvas.doc({ slug: 'roadmap' }))
    const url = FakeEventSource.instances[0]!.url
    expect(decodeURIComponent(url.split('expr=')[1]!)).toBe(
      'ship.canvas.doc({"slug":"roadmap"})',
    )
  })

  it('argless ops with no Connect service use SSE (mesh.snapshot)', () => {
    FakeEventSource.instances = []
    const ship = createClient({ EventSource: FakeEventSource as any })
    live(() => ship.mesh.snapshot())
    expect(FakeEventSource.instances).toHaveLength(1)
    const es = FakeEventSource.instances[0]!
    expect(decodeURIComponent(es.url.split('expr=')[1]!)).toBe('ship.mesh.snapshot()')
  })

  it('sessions.start POSTs JSON to /api/v1/sessions/spawn', async () => {
    const calls: Array<{ url: string; init?: RequestInit }> = []
    const orig = globalThis.fetch
    globalThis.fetch = async (url: any, init?: any) => {
      calls.push({ url: String(url), init })
      return new Response(JSON.stringify({ session_id: 'sess-1', attached: true }), {
        status: 200,
        headers: { 'content-type': 'application/json' },
      })
    }
    try {
      const ship = createClient({ EventSource: FakeEventSource as any })
      const resp = await ship.sessions.start({
        workspace_id: 'ws-1',
        kind: 'shell',
      })
      expect(resp).toEqual({ session_id: 'sess-1', attached: true })
      expect(calls).toHaveLength(1)
      expect(calls[0]!.url).toBe('/api/v1/sessions/spawn')
      expect(calls[0]!.init?.method).toBe('POST')
      expect(JSON.parse(calls[0]!.init?.body as string)).toEqual({
        workspace_id: 'ws-1',
        kind: 'shell',
      })
    } finally {
      globalThis.fetch = orig
    }
  })

  it('sessions.end DELETEs against the session id', async () => {
    const calls: Array<{ url: string; init?: RequestInit }> = []
    const orig = globalThis.fetch
    globalThis.fetch = async (url: any, init?: any) => {
      calls.push({ url: String(url), init })
      return new Response(null, { status: 204 })
    }
    try {
      const ship = createClient({ EventSource: FakeEventSource as any })
      await ship.sessions.end({ id: 'sess-2' })
      expect(calls[0]!.url).toBe('/api/v1/sessions/sess-2')
      expect(calls[0]!.init?.method).toBe('DELETE')
    } finally {
      globalThis.fetch = orig
    }
  })


  it('skills.invoke POSTs to /api/v1/skills/{id}/invoke', async () => {
    const calls: Array<{ url: string; init?: RequestInit }> = []
    const orig = globalThis.fetch
    globalThis.fetch = async (url: any, init?: any) => {
      calls.push({ url: String(url), init })
      return new Response(
        JSON.stringify({
          ok: true,
          run_id: 'run-1',
          skill_id: 'foo',
          workspace_id: null,
        }),
        { status: 200, headers: { 'content-type': 'application/json' } },
      )
    }
    try {
      const ship = createClient({ EventSource: FakeEventSource as any })
      const resp = await ship.skills.invoke({ id: 'foo', vars: { k: 'v' } })
      expect(resp.run_id).toBe('run-1')
      expect(resp.skill_id).toBe('foo')
      expect(calls[0]!.url).toBe('/api/v1/skills/foo/invoke')
      expect(JSON.parse(calls[0]!.init?.body as string)).toEqual({
        vars: { k: 'v' },
      })
    } finally {
      globalThis.fetch = orig
    }
  })

  it('mutation errors surface the daemon response body', async () => {
    const orig = globalThis.fetch
    globalThis.fetch = async () =>
      new Response('{"error":"agent_id required"}', {
        status: 400,
        statusText: 'Bad Request',
      })
    try {
      const ship = createClient({ EventSource: FakeEventSource as any })
      await expect(
        ship.sessions.start({ workspace_id: 'ws', kind: 'agent' }),
      ).rejects.toThrow(/agent_id required/)
    } finally {
      globalThis.fetch = orig
    }
  })

  it('agents.create calls Connect and returns {ok, id}', async () => {
    const fakeConnect = createFakeConnectClient()
    fakeConnect.setResponse('agents.create', { id: 'foo' })
    const ship = createClient({ connectClient: fakeConnect.build() })
    const doc = { agent: { id: 'foo', name: 'Foo' } }
    const resp = await ship.agents.create({ id: 'foo', doc })
    expect(resp).toEqual({ ok: true, id: 'foo' })
    const calls = fakeConnect.getCalls()
    expect(calls).toHaveLength(1)
    expect(calls[0]!.domain).toBe('agents')
    expect(calls[0]!.op).toBe('create')
    expect(calls[0]!.req).toEqual({
      id: 'foo',
      docJson: JSON.stringify(doc),
      scope: '',
    })
  })

  it('agents.update calls Connect and returns {ok, id}', async () => {
    const fakeConnect = createFakeConnectClient()
    fakeConnect.setResponse('agents.put', { id: 'foo' })
    const ship = createClient({ connectClient: fakeConnect.build() })
    const resp = await ship.agents.update({ id: 'foo', doc: { agent: { id: 'foo' } } })
    expect(resp).toEqual({ ok: true, id: 'foo' })
    const calls = fakeConnect.getCalls()
    expect(calls).toHaveLength(1)
    expect(calls[0]!.domain).toBe('agents')
    expect(calls[0]!.op).toBe('put')
  })

  it('collections.create_item calls Connect and returns {ok, id}', async () => {
    const fakeConnect = createFakeConnectClient()
    fakeConnect.setResponse('collections.createItem', { id: 'item-1' })
    const ship = createClient({ connectClient: fakeConnect.build() })
    const resp = await ship.collections.create_item({
      collection: 'systems',
      data: { name: 'X' },
    })
    expect(resp).toEqual({ ok: true, id: 'item-1' })
    const calls = fakeConnect.getCalls()
    expect(calls).toHaveLength(1)
    expect(calls[0]!.domain).toBe('collections')
    expect(calls[0]!.op).toBe('createItem')
    expect(calls[0]!.req).toEqual({
      collection: 'systems',
      dataJson: JSON.stringify({ name: 'X' }),
      id: '',
    })
  })

  it('collections.update_item and delete_item call Connect', async () => {
    const fakeConnect = createFakeConnectClient()
    fakeConnect.setResponse('collections.updateItem', { id: 'r-1' })
    fakeConnect.setResponse('collections.deleteItem', {})
    const ship = createClient({ connectClient: fakeConnect.build() })
    await ship.collections.update_item({
      collection: 'systems',
      item_id: 'r-1',
      data: { a: 1 },
    })
    await ship.collections.delete_item({ collection: 'systems', item_id: 'r-1' })
    const calls = fakeConnect.getCalls()
    expect(calls).toHaveLength(2)
    expect(calls[0]!.domain).toBe('collections')
    expect(calls[0]!.op).toBe('updateItem')
    expect(calls[1]!.domain).toBe('collections')
    expect(calls[1]!.op).toBe('deleteItem')
  })

  it('collections.create calls Connect and returns {ok, id}', async () => {
    const fakeConnect = createFakeConnectClient()
    fakeConnect.setResponse('collections.createSchema', {
      json: JSON.stringify({ ok: true, id: 'x' }),
    })
    const ship = createClient({ connectClient: fakeConnect.build() })
    const resp = await ship.collections.create({ id: 'x', name: 'X collection' })
    expect(resp).toEqual({ ok: true, id: 'x' })
    const calls = fakeConnect.getCalls()
    expect(calls).toHaveLength(1)
    expect(calls[0]!.domain).toBe('collections')
    expect(calls[0]!.op).toBe('createSchema')
  })

  it('artifacts.get returns text by default and json with as=json', async () => {
    const orig = globalThis.fetch
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
    try {
      const ship = createClient({ EventSource: FakeEventSource as any })
      const text = await ship.artifacts.get({ id: 'a1' })
      expect(text).toBe('hello world')
      const json = await ship.artifacts.get({ id: 'a2', as: 'json' })
      expect(json).toEqual({ ok: true, value: 42 })
    } finally {
      globalThis.fetch = orig
    }
  })

  it('artifacts.upload posts multipart form to /api/v1/artifacts', async () => {
    const calls: Array<{ url: string; init?: RequestInit }> = []
    const orig = globalThis.fetch
    globalThis.fetch = async (url: any, init?: any) => {
      calls.push({ url: String(url), init })
      return new Response(
        JSON.stringify({ ok: true, data: { id: 'a1', name: 'foo.txt', mime: 'text/plain' } }),
        { status: 200, headers: { 'content-type': 'application/json' } },
      )
    }
    try {
      const ship = createClient({ EventSource: FakeEventSource as any })
      // jsdom has File/Blob — pass a File.
      const file = new File(['hello'], 'foo.txt', { type: 'text/plain' })
      const resp = await ship.artifacts.upload({ file, collection_id: 'systems' })
      expect((resp as { ok: boolean }).ok).toBe(true)
      expect(calls[0]!.url).toBe('/api/v1/artifacts')
      expect(calls[0]!.init?.method).toBe('POST')
      expect(calls[0]!.init?.body).toBeInstanceOf(FormData)
      const fd = calls[0]!.init?.body as FormData
      expect(fd.get('file')).toBeInstanceOf(File)
      expect(fd.get('collection_id')).toBe('systems')
    } finally {
      globalThis.fetch = orig
    }
  })

  it('disposer closes the EventSource', () => {
    FakeEventSource.instances = []
    const source = makeSseSource('http://test', FakeEventSource as any)
    let captured: (() => void) | undefined
    source.subscribe = ((orig) => (sig) => {
      captured = orig.call(source, sig)
      return captured
    })(source.subscribe)
    live(() => source)
    expect(FakeEventSource.instances).toHaveLength(1)
    captured!()
    expect(FakeEventSource.instances[0]!.closed).toBe(true)
  })

  // Multi-subscriber fanout coverage (#198) lives in shared-source.test.ts —
  // exercises the dedup helper directly with a synthetic LiveSource so it
  // doesn't depend on FakeEventSource (which is upstream-broken — #214).
})
