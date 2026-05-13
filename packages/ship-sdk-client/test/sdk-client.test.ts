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
      'ship.events.list({"q":"foo","event_type":"job.completed"})',
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

  it('workspaces.list fetches /api/v1/projects', async () => {
    const orig = globalThis.fetch
    globalThis.fetch = async () =>
      new Response(JSON.stringify({ projects: [], active: null }), {
        status: 200,
        headers: { 'content-type': 'application/json' },
      })
    try {
      const ship = createClient({ EventSource: FakeEventSource as any })
      const resp = await ship.workspaces.list()
      expect(resp).toEqual({ projects: [], active: null })
    } finally {
      globalThis.fetch = orig
    }
  })

  it('skills.invoke rejects with a structured not_implemented error', async () => {
    const ship = createClient({ EventSource: FakeEventSource as any })
    await expect(
      ship.skills.invoke({ id: 'foo' }),
    ).rejects.toMatchObject({ code: 'not_implemented', op: 'skills.invoke' })
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
