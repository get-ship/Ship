import { describe, expect, it } from 'vitest'
import { live } from '@ship/ui-runtime'
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
    expect(es.url).toMatch(/_ship-set\/data\?expr=/)
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
      'http://test/_ship-set/data?expr=ship.agents.summaries%28%29',
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
    expect(url).toContain('/_ship-set/data?expr=')
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
