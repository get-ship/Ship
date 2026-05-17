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

describe('createClient', () => {
  it('agents.summaries opens an SSE channel and resolves on first patch', () => {
    FakeEventSource.instances = []
    const ship = createClient({ EventSource: FakeEventSource as any })
    const sig = live(() => ship.agents.summaries())
    expect(sig().kind).toBe('loading')
    expect(FakeEventSource.instances).toHaveLength(1)
    const es = FakeEventSource.instances[0]!
    expect(es.url).toMatch(/_sui\/data\?expr=/)
    es.emitPatch({
      op: 'set',
      value: { items: [{ id: 'a', name: 'A' }] },
    })
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
      'http://test/_sui/data?expr=ship.agents.summaries%28%29',
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

  it('git.diff serializes args as a JSON object literal in the URL', () => {
    FakeEventSource.instances = []
    const ship = createClient({ EventSource: FakeEventSource as any })
    const sig = live(() => ship.git.diff({ commit: 'abc1234' }))
    expect(sig().kind).toBe('loading')
    const url = FakeEventSource.instances[0]!.url
    expect(url).toContain('/_sui/data?expr=')
    // Args round-trip through encodeURIComponent — assert against the
    // decoded form for legibility.
    expect(decodeURIComponent(url.split('expr=')[1]!)).toBe(
      'ship.git.diff({"commit":"abc1234"})',
    )
  })

  it('git.diff includes optional workspace_id when provided', () => {
    FakeEventSource.instances = []
    const ship = createClient({ EventSource: FakeEventSource as any })
    live(() => ship.git.diff({ commit: 'WORKING', workspace_id: 'ws-1' }))
    const url = FakeEventSource.instances[0]!.url
    expect(decodeURIComponent(url.split('expr=')[1]!)).toBe(
      'ship.git.diff({"commit":"WORKING","workspace_id":"ws-1"})',
    )
  })

  it('events.list serializes filter args as a JSON object literal', () => {
    FakeEventSource.instances = []
    const ship = createClient({ EventSource: FakeEventSource as any })
    live(() => ship.events.list({ q: 'foo', event_type: 'job.completed' }))
    const url = FakeEventSource.instances[0]!.url
    expect(decodeURIComponent(url.split('expr=')[1]!)).toBe(
      'ship.events.list({"q":"foo","event_type":"pipeline.run.completed"})',
    )
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

  it('argless ops emit empty parens (chat.state, mesh.snapshot, settings.state)', () => {
    FakeEventSource.instances = []
    const ship = createClient({ EventSource: FakeEventSource as any })
    live(() => ship.mesh.snapshot())
    live(() => ship.settings.state())
    live(() => ship.dashboard.state())
    const exprs = FakeEventSource.instances.map((es) =>
      decodeURIComponent(es.url.split('expr=')[1]!),
    )
    expect(exprs).toEqual([
      'ship.mesh.snapshot()',
      'ship.settings.state()',
      'ship.dashboard.state()',
    ])
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

  it('workspaces.activate POSTs to /api/v1/projects/switch', async () => {
    const calls: Array<{ url: string; init?: RequestInit }> = []
    const orig = globalThis.fetch
    globalThis.fetch = async (url: any, init?: any) => {
      calls.push({ url: String(url), init })
      return new Response(JSON.stringify({ ok: true, active: 'proj-1' }), {
        status: 200,
        headers: { 'content-type': 'application/json' },
      })
    }
    try {
      const ship = createClient({ EventSource: FakeEventSource as any })
      const resp = await ship.workspaces.activate({ id: 'proj-1' })
      expect(resp).toEqual({ ok: true, active: 'proj-1' })
      expect(calls[0]!.url).toBe('/api/v1/projects/switch')
      expect(JSON.parse(calls[0]!.init?.body as string)).toEqual({ id: 'proj-1' })
    } finally {
      globalThis.fetch = orig
    }
  })

  it('workspaces.list opens an SSE channel', () => {
    FakeEventSource.instances = []
    const ship = createClient({ EventSource: FakeEventSource as any })
    const sig = live(() => ship.workspaces.list())
    expect(sig().kind).toBe('loading')
    expect(FakeEventSource.instances).toHaveLength(1)
    expect(FakeEventSource.instances[0]!.url).toMatch(/workspaces\.list/)
  })

  it('workspaces.active opens an SSE channel', () => {
    FakeEventSource.instances = []
    const ship = createClient({ EventSource: FakeEventSource as any })
    const sig = live(() => ship.workspaces.active())
    expect(sig().kind).toBe('loading')
    expect(FakeEventSource.instances).toHaveLength(1)
    expect(FakeEventSource.instances[0]!.url).toMatch(/workspaces\.active/)
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

  it('agents.create POSTs {id, doc} to /api/v1/agents', async () => {
    const calls: Array<{ url: string; init?: RequestInit }> = []
    const orig = globalThis.fetch
    globalThis.fetch = async (url: any, init?: any) => {
      calls.push({ url: String(url), init })
      return new Response(JSON.stringify({ ok: true, id: 'foo' }), {
        status: 200,
        headers: { 'content-type': 'application/json' },
      })
    }
    try {
      const ship = createClient({ EventSource: FakeEventSource as any })
      const doc = { agent: { id: 'foo', name: 'Foo' } }
      const resp = await ship.agents.create({ id: 'foo', doc })
      expect(resp).toEqual({ ok: true, id: 'foo' })
      expect(calls[0]!.url).toBe('/api/v1/agents')
      expect(calls[0]!.init?.method).toBe('POST')
      expect(JSON.parse(calls[0]!.init?.body as string)).toEqual({ id: 'foo', doc })
    } finally {
      globalThis.fetch = orig
    }
  })

  it('agents.update PUTs the doc body to /api/v1/agents/{id}', async () => {
    const calls: Array<{ url: string; init?: RequestInit }> = []
    const orig = globalThis.fetch
    globalThis.fetch = async (url: any, init?: any) => {
      calls.push({ url: String(url), init })
      return new Response(JSON.stringify({ ok: true, id: 'foo' }), {
        status: 200,
        headers: { 'content-type': 'application/json' },
      })
    }
    try {
      const ship = createClient({ EventSource: FakeEventSource as any })
      await ship.agents.update({ id: 'foo', doc: { agent: { id: 'foo' } } })
      expect(calls[0]!.url).toBe('/api/v1/agents/foo')
      expect(calls[0]!.init?.method).toBe('PUT')
    } finally {
      globalThis.fetch = orig
    }
  })

  it('collections.create_item POSTs to /api/v1/collections/{id}/items', async () => {
    const calls: Array<{ url: string; init?: RequestInit }> = []
    const orig = globalThis.fetch
    globalThis.fetch = async (url: any, init?: any) => {
      calls.push({ url: String(url), init })
      return new Response(JSON.stringify({ ok: true, id: 'item-1' }), {
        status: 200,
        headers: { 'content-type': 'application/json' },
      })
    }
    try {
      const ship = createClient({ EventSource: FakeEventSource as any })
      await ship.collections.create_item({
        collection: 'systems',
        data: { name: 'X' },
      })
      expect(calls[0]!.url).toBe('/api/v1/collections/systems/items')
      expect(JSON.parse(calls[0]!.init?.body as string)).toEqual({ data: { name: 'X' } })
    } finally {
      globalThis.fetch = orig
    }
  })

  it('collections.update_item PUTs and collections.delete_item DELETEs', async () => {
    const calls: Array<{ url: string; init?: RequestInit }> = []
    const orig = globalThis.fetch
    globalThis.fetch = async (url: any, init?: any) => {
      calls.push({ url: String(url), init })
      return new Response(JSON.stringify({ ok: true, id: 'r-1' }), {
        status: 200,
        headers: { 'content-type': 'application/json' },
      })
    }
    try {
      const ship = createClient({ EventSource: FakeEventSource as any })
      await ship.collections.update_item({
        collection: 'systems',
        item_id: 'r-1',
        data: { a: 1 },
      })
      await ship.collections.delete_item({ collection: 'systems', item_id: 'r-1' })
      expect(calls[0]!.url).toBe('/api/v1/collections/systems/items/r-1')
      expect(calls[0]!.init?.method).toBe('PUT')
      expect(calls[1]!.init?.method).toBe('DELETE')
    } finally {
      globalThis.fetch = orig
    }
  })

  it('collections.create POSTs to /api/v1/collections with id/name/fields', async () => {
    const calls: Array<{ url: string; init?: RequestInit }> = []
    const orig = globalThis.fetch
    globalThis.fetch = async (url: any, init?: any) => {
      calls.push({ url: String(url), init })
      return new Response(JSON.stringify({ ok: true, id: 'x' }), {
        status: 200,
        headers: { 'content-type': 'application/json' },
      })
    }
    try {
      const ship = createClient({ EventSource: FakeEventSource as any })
      await ship.collections.create({ id: 'x', name: 'X collection' })
      const body = JSON.parse(calls[0]!.init?.body as string)
      expect(body).toEqual({ id: 'x', name: 'X collection', fields: {} })
    } finally {
      globalThis.fetch = orig
    }
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
})
