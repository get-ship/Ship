import { describe, it, expect, vi } from 'vitest'
import { ShipClient } from './client'
import type { Transport, TransportRequestInit, ShipEventEnvelope } from './transport'

/** Minimal in-memory transport, useful for asserting client behavior. */
function mockTransport(handler: (path: string, init?: TransportRequestInit) => unknown) {
  const listeners = new Set<(e: ShipEventEnvelope) => void>()
  const requestSpy = vi.fn((path: string, init?: TransportRequestInit) => {
    return Promise.resolve(handler(path, init))
  })
  const transport: Transport = {
    request: requestSpy as Transport['request'],
    subscribeEvents(fn) {
      listeners.add(fn)
      return () => listeners.delete(fn)
    },
    destroy() {
      listeners.clear()
    },
  }
  return { transport, requestSpy, listeners }
}

describe('ShipClient construction', () => {
  it('constructs without options and exposes defaults', () => {
    const c = new ShipClient()
    expect(c).toBeInstanceOf(ShipClient)
    expect(c.activeWorkspace).toBeUndefined()
  })

  it('accepts activeWorkspace', () => {
    const c = new ShipClient({ activeWorkspace: 'ws-1' })
    expect(c.activeWorkspace).toBe('ws-1')
  })

  it('accepts a custom transport', () => {
    const { transport } = mockTransport(() => ({}))
    const c = new ShipClient({ transport })
    expect(c).toBeInstanceOf(ShipClient)
  })
})

describe('ShipClient HTTP surface via transport', () => {
  it('getJobs unwraps the {jobs} payload', async () => {
    const { transport } = mockTransport(() => ({ jobs: [{ id: 'a' }, { id: 'b' }] }))
    const c = new ShipClient({ transport })
    const jobs = await c.getJobs()
    expect(jobs).toEqual([{ id: 'a' }, { id: 'b' }])
  })

  it('getJob requests /runtime/jobs/:id with raw envelope', async () => {
    const { transport, requestSpy } = mockTransport(() => ({ ok: true, data: { id: 'j1' } }))
    const c = new ShipClient({ transport })
    const res = await c.getJob('j1')
    expect(requestSpy).toHaveBeenCalledWith('/runtime/jobs/j1', {
      method: 'GET',
      raw: true,
    })
    expect(res).toEqual({ ok: true, data: { id: 'j1' } })
  })

  it('createJob posts params', async () => {
    const { transport, requestSpy } = mockTransport(() => ({ ok: true }))
    const c = new ShipClient({ transport })
    await c.createJob({ kind: 'test' })
    expect(requestSpy).toHaveBeenCalledWith('/runtime/jobs', {
      method: 'POST',
      body: { kind: 'test' },
      raw: true,
    })
  })

  it('emitEvent posts the canonical event envelope', async () => {
    const { transport, requestSpy } = mockTransport(() => ({ ok: true }))
    const c = new ShipClient({ transport })
    await c.emitEvent('job.created', 'j1', { foo: 'bar' })
    expect(requestSpy).toHaveBeenCalledWith('/events/emit', {
      method: 'POST',
      body: { event_type: 'job.created', entity_id: 'j1', payload: { foo: 'bar' } },
      raw: true,
    })
  })

  it('getSessionFiles and readFile hit workspace-scoped paths', async () => {
    const { transport, requestSpy } = mockTransport(() => ({ ok: true }))
    const c = new ShipClient({ transport })
    await c.getSessionFiles('ws-1')
    await c.readFile('ws-1', 'notes.md')
    expect(requestSpy.mock.calls[0]?.[0]).toBe('/workspaces/ws-1/session-files')
    expect(requestSpy.mock.calls[1]?.[0]).toBe('/workspaces/ws-1/session-files/notes.md')
  })
})

describe('ShipClient events', () => {
  it('onEvent returns an unsubscribe and routes events', () => {
    const { transport, listeners } = mockTransport(() => undefined)
    const c = new ShipClient({ transport })
    const received: ShipEventEnvelope[] = []
    const unsub = c.onEvent((e) => received.push(e))
    expect(listeners.size).toBe(1)
    const evt: ShipEventEnvelope = {
      id: '1',
      event_type: 'job.created',
      entity_id: 'j1',
      created_at: 'now',
    }
    listeners.forEach((l) => l(evt))
    expect(received).toHaveLength(1)
    unsub()
    // the transport listener is released once the last consumer goes
    expect(listeners.size).toBe(0)
  })
})
