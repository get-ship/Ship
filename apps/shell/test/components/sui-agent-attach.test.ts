// Unit tests for the WS attach bridge that powers <sui-agent-attach>.
// Stubs the WebSocket constructor + a fake <ship-terminal>-shaped host so
// the byte-pump can run in JSDOM.

import { describe, expect, it } from 'vitest'

import {
  attachAgentSession,
  type AgentSessionAttachOptions,
} from '../../src/components/sui-agent-attach/attach'

// ── Fakes ────────────────────────────────────────────────────────────────────

interface SentChunk { kind: 'text' | 'bytes'; bytes: Uint8Array; text: string }

class FakeSocket extends EventTarget {
  static OPEN = 1 as const
  static CLOSED = 3 as const
  static last: FakeSocket | null = null

  readyState: number = 0
  url: string
  binaryType: BinaryType = 'blob'
  sent: SentChunk[] = []
  closed = false

  constructor(url: string | URL) {
    super()
    this.url = String(url)
    FakeSocket.last = this
  }

  /// Simulate the server-side open handshake completing.
  fireOpen(): void {
    this.readyState = FakeSocket.OPEN
    this.dispatchEvent(new Event('open'))
  }

  /// Push a binary frame "from the server."
  fireBinary(bytes: Uint8Array): void {
    // Detach the buffer copy so the bridge's view doesn't share memory.
    const copy = new Uint8Array(bytes)
    const ev = new MessageEvent('message', { data: copy.buffer })
    this.dispatchEvent(ev)
  }

  fireClose(reason = ''): void {
    this.readyState = FakeSocket.CLOSED
    this.closed = true
    this.dispatchEvent(new CloseEvent('close', { code: 1000, reason }))
  }

  send(data: ArrayBufferLike | ArrayBufferView | string): void {
    if (typeof data === 'string') {
      this.sent.push({ kind: 'text', bytes: new TextEncoder().encode(data), text: data })
      return
    }
    const bytes = data instanceof ArrayBuffer
      ? new Uint8Array(data)
      : new Uint8Array((data as ArrayBufferView).buffer)
    this.sent.push({ kind: 'bytes', bytes, text: new TextDecoder().decode(bytes) })
  }

  close(_code?: number, _reason?: string): void {
    this.fireClose(_reason ?? '')
  }
}

/// Minimal `<ship-terminal>`-shaped host: tracks writes, supports the data
/// CustomEvent the bridge listens for, and exposes a `terminal` truthy
/// once `markReady()` runs so the buffer flushes.
function makeTermStub(): HTMLElement & {
  writes: string[]
  markReady: () => void
  emitData: (text: string) => void
} {
  const el = document.createElement('div') as HTMLElement & {
    writes: string[]
    write?: (data: string | Uint8Array) => void
    terminal?: { cols: number; rows: number } | null
    markReady: () => void
    emitData: (text: string) => void
  }
  el.writes = []
  el.write = (data: string | Uint8Array) => {
    const text = typeof data === 'string' ? data : new TextDecoder().decode(data)
    el.writes.push(text)
  }
  el.markReady = () => {
    el.terminal = { cols: 80, rows: 24 }
    el.dispatchEvent(new CustomEvent('ship-terminal-ready', { bubbles: true, composed: true }))
  }
  el.emitData = (text: string) => {
    el.dispatchEvent(new CustomEvent('ship-terminal-data', {
      detail: { data: text }, bubbles: true, composed: true,
    }))
  }
  return el
}

function defaultOpts(extras: Partial<AgentSessionAttachOptions> = {}): AgentSessionAttachOptions {
  return {
    scheme: 'ws:',
    host: 'localhost:1234',
    // Cast: tests use a fake constructor; the real WebSocket signature
    // includes the optional protocols arg the fake ignores.
    webSocketCtor: FakeSocket as unknown as typeof WebSocket,
    ...extras,
  }
}

// ── Tests ────────────────────────────────────────────────────────────────────

describe('attachAgentSession', () => {
  it('opens the WS at /api/v1/agent_sessions/{slug}/attach with binary type', () => {
    const term = makeTermStub()
    const attachment = attachAgentSession(term, 'my-job', defaultOpts())
    expect(FakeSocket.last).not.toBeNull()
    expect(FakeSocket.last!.url).toBe(
      'ws://localhost:1234/api/v1/agent_sessions/my-job/attach',
    )
    expect(FakeSocket.last!.binaryType).toBe('arraybuffer')
    attachment.dispose()
  })

  it('buffers server bytes until <ship-terminal> is ready, then flushes', () => {
    const term = makeTermStub()
    const attachment = attachAgentSession(term, 'echo', defaultOpts())
    FakeSocket.last!.fireOpen()
    FakeSocket.last!.fireBinary(new TextEncoder().encode('first '))
    FakeSocket.last!.fireBinary(new TextEncoder().encode('second'))
    // Engine not mounted yet — nothing should have been written.
    expect(term.writes).toEqual([])
    term.markReady()
    expect(term.writes.join('')).toBe('first second')
    attachment.dispose()
  })

  it('forwards xterm onData payloads as WS frames after open', () => {
    const term = makeTermStub()
    const attachment = attachAgentSession(term, 'echo', defaultOpts())
    FakeSocket.last!.fireOpen()
    term.emitData('hello\n')
    expect(FakeSocket.last!.sent).toHaveLength(1)
    expect(FakeSocket.last!.sent[0]!.text).toBe('hello\n')
    attachment.dispose()
  })

  it('drops input writes once the socket has closed', () => {
    const term = makeTermStub()
    const attachment = attachAgentSession(term, 'echo', defaultOpts())
    FakeSocket.last!.fireOpen()
    FakeSocket.last!.fireClose('peer gone')
    // After close, the bridge nulls its socket reference — onData is a
    // no-op (no exception, no queued frame).
    term.emitData('post-close\n')
    expect(FakeSocket.last!.sent).toHaveLength(0)
    expect(attachment.isConnected()).toBe(false)
    attachment.dispose()
  })

  it('invokes onDisconnect with the close reason', () => {
    const term = makeTermStub()
    const reasons: string[] = []
    const attachment = attachAgentSession(term, 'echo', defaultOpts({
      onDisconnect: (r) => reasons.push(r),
    }))
    FakeSocket.last!.fireOpen()
    FakeSocket.last!.fireClose('peer gone')
    expect(reasons).toEqual(['peer gone'])
    attachment.dispose()
  })

  it('dispose() removes listeners and closes the socket', () => {
    const term = makeTermStub()
    const attachment = attachAgentSession(term, 'echo', defaultOpts())
    FakeSocket.last!.fireOpen()
    attachment.dispose()
    expect(FakeSocket.last!.closed).toBe(true)
    // Post-dispose input must not be forwarded — the listener is detached.
    term.emitData('after-dispose\n')
    expect(FakeSocket.last!.sent).toHaveLength(0)
  })
})
